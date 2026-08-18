[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$CurrentInstaller,
    [string]$PreviousInstaller,
    [string]$ReportPath
)

$ErrorActionPreference = "Stop"

function Resolve-OneFile {
    param([Parameter(Mandatory)][string]$Path)
    $matches = @(Resolve-Path -Path $Path)
    if ($matches.Count -ne 1 -or -not (Test-Path -LiteralPath $matches[0].Path -PathType Leaf)) {
        throw "Installer path must resolve to exactly one file: $Path"
    }
    return $matches[0].Path
}

function Get-TradeDeskInstallRecords {
    $roots = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"
    )
    return @($roots | ForEach-Object {
        if (Test-Path -LiteralPath $_) {
            Get-ChildItem -LiteralPath $_ | ForEach-Object {
                $record = Get-ItemProperty -LiteralPath $_.PSPath -ErrorAction SilentlyContinue
                if ($record.DisplayName -like "*TradeDesk*") { $record }
            }
        }
    })
}

function Get-TradeDeskShortcuts {
    $roots = @(
        (Join-Path $env:APPDATA "Microsoft\Windows\Start Menu"),
        (Join-Path $env:USERPROFILE "Desktop"),
        (Join-Path $env:PUBLIC "Desktop")
    )
    return @($roots | ForEach-Object {
        if (Test-Path -LiteralPath $_) {
            Get-ChildItem -LiteralPath $_ -Filter "*TradeDesk*.lnk" -File -Recurse -ErrorAction SilentlyContinue
        }
    })
}

function Invoke-SilentInstall {
    param([Parameter(Mandatory)][string]$Installer, [Parameter(Mandatory)][string]$Destination)
    $process = Start-Process -FilePath $Installer -ArgumentList "/S /D=$Destination" -Wait -PassThru
    if ($process.ExitCode -ne 0) {
        throw "Installer exited with code $($process.ExitCode): $Installer"
    }
}

function Get-VersionFromInstallerName {
    param([Parameter(Mandatory)][string]$Installer)
    $match = [regex]::Match(
        (Split-Path $Installer -Leaf),
        '\d+\.\d+\.\d+(?:-[0-9A-Za-z]+(?:\.[0-9A-Za-z]+)*)?'
    )
    if (-not $match.Success) { throw "Cannot determine version from installer filename: $Installer" }
    return $match.Value
}

function Get-InstalledLayout {
    param([Parameter(Mandatory)][string]$InstallDirectory)
    $mainExe = Get-ChildItem -LiteralPath $InstallDirectory -Filter "*.exe" -File |
        Where-Object { $_.Name -notmatch "^(uninstall|typst)" } |
        Select-Object -First 1
    $uninstaller = Get-ChildItem -LiteralPath $InstallDirectory -Filter "uninstall*.exe" -File |
        Select-Object -First 1
    $required = @(
        $mainExe,
        $uninstaller,
        (Get-Item -LiteralPath (Join-Path $InstallDirectory "typst.exe") -ErrorAction SilentlyContinue),
        (Get-Item -LiteralPath (Join-Path $InstallDirectory "TYPST-LICENSE.txt") -ErrorAction SilentlyContinue),
        (Get-Item -LiteralPath (Join-Path $InstallDirectory "TYPST-NOTICE.txt") -ErrorAction SilentlyContinue)
    )
    if ($required.Where({ $null -eq $_ }).Count -gt 0) {
        throw "Installed application is missing the executable, uninstaller or Typst resources"
    }
    return [ordered]@{
        mainExe = $mainExe.FullName
        uninstaller = $uninstaller.FullName
        productVersion = $mainExe.VersionInfo.ProductVersion
        typstBytes = (Get-Item -LiteralPath (Join-Path $InstallDirectory "typst.exe")).Length
    }
}

$currentInstallerPath = Resolve-OneFile $CurrentInstaller
$previousInstallerPath = if ($PreviousInstaller) { Resolve-OneFile $PreviousInstaller } else { $null }
$projectRoot = Split-Path $PSScriptRoot -Parent
$runRoot = Join-Path $projectRoot "tmp\installer-smoke\$([guid]::NewGuid().ToString('N'))"
$installDirectory = Join-Path $runRoot "app"
$workspaceDirectory = Join-Path $env:APPDATA "cn.treedeep.tradedesk"
$sentinelPath = Join-Path $workspaceDirectory "installer-smoke-workspace.sentinel"
$workspaceCreatedByTest = $false
$installedLayout = $null
$startedAt = [DateTime]::UtcNow

if (Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "*trade-desk*" }) {
    throw "Close all TradeDesk processes before running the installer smoke test"
}
if ((Get-TradeDeskInstallRecords).Count -gt 0) {
    throw "A TradeDesk installation already exists; use an isolated Windows test account"
}
if ((Get-TradeDeskShortcuts).Count -gt 0) {
    throw "A TradeDesk shortcut already exists; use an isolated Windows test account"
}
if (Test-Path -LiteralPath $workspaceDirectory) {
    throw "A TradeDesk workspace already exists at $workspaceDirectory; use an isolated Windows test account"
}

New-Item -ItemType Directory -Path $installDirectory -Force | Out-Null

try {
    $phases = [System.Collections.Generic.List[object]]::new()
    if ($previousInstallerPath) {
        Invoke-SilentInstall -Installer $previousInstallerPath -Destination $installDirectory
        $previousLayout = Get-InstalledLayout -InstallDirectory $installDirectory
        $expectedPreviousVersion = Get-VersionFromInstallerName $previousInstallerPath
        if ($previousLayout.productVersion -ne $expectedPreviousVersion) {
            throw "Previous install version mismatch: $($previousLayout.productVersion) / $expectedPreviousVersion"
        }
        $phases.Add([ordered]@{ phase = "previous-install"; version = $previousLayout.productVersion })
    }

    if (-not (Test-Path -LiteralPath $workspaceDirectory)) {
        New-Item -ItemType Directory -Path $workspaceDirectory | Out-Null
        $workspaceCreatedByTest = $true
    }
    "TradeDesk installer data-preservation probe" | Set-Content -LiteralPath $sentinelPath -Encoding utf8

    Invoke-SilentInstall -Installer $currentInstallerPath -Destination $installDirectory
    $installedLayout = Get-InstalledLayout -InstallDirectory $installDirectory
    $expectedCurrentVersion = Get-VersionFromInstallerName $currentInstallerPath
    if ($installedLayout.productVersion -ne $expectedCurrentVersion) {
        throw "Current install version mismatch: $($installedLayout.productVersion) / $expectedCurrentVersion"
    }
    $installedShortcuts = Get-TradeDeskShortcuts
    if ($installedShortcuts.Count -eq 0) {
        throw "Installer did not create a TradeDesk Start Menu or desktop shortcut"
    }
    if (-not (Test-Path -LiteralPath $sentinelPath)) {
        throw "Workspace sentinel was removed during installation or upgrade"
    }
    $phases.Add([ordered]@{
        phase = if ($previousInstallerPath) { "upgrade" } else { "clean-install" }
        version = $installedLayout.productVersion
    })

    $uninstall = Start-Process -FilePath $installedLayout.uninstaller -ArgumentList "/S" -Wait -PassThru
    if ($uninstall.ExitCode -ne 0) { throw "Uninstaller exited with code $($uninstall.ExitCode)" }
    Start-Sleep -Milliseconds 500
    if (Test-Path -LiteralPath $installedLayout.mainExe) {
        throw "Application executable remains after uninstall"
    }
    if (-not (Test-Path -LiteralPath $sentinelPath)) {
        throw "Workspace sentinel was removed by uninstall"
    }
    if ((Get-TradeDeskInstallRecords).Count -gt 0) {
        throw "TradeDesk uninstall registry record remains after uninstall"
    }
    if ((Get-TradeDeskShortcuts).Count -gt 0) {
        throw "TradeDesk shortcut remains after uninstall"
    }
    $phases.Add([ordered]@{ phase = "uninstall"; workspacePreserved = $true })

    if (-not $ReportPath) { $ReportPath = Join-Path $runRoot "installer-smoke-report.json" }
    $reportParent = Split-Path ([IO.Path]::GetFullPath($ReportPath)) -Parent
    New-Item -ItemType Directory -Path $reportParent -Force | Out-Null
    [ordered]@{
        result = "passed"
        currentInstaller = Split-Path $currentInstallerPath -Leaf
        previousInstaller = if ($previousInstallerPath) { Split-Path $previousInstallerPath -Leaf } else { $null }
        currentSha256 = (Get-FileHash -LiteralPath $currentInstallerPath -Algorithm SHA256).Hash.ToLowerInvariant()
        typstBytes = $installedLayout.typstBytes
        installedShortcutCount = $installedShortcuts.Count
        phases = $phases
        durationSeconds = [Math]::Round(([DateTime]::UtcNow - $startedAt).TotalSeconds, 2)
        testedAtUtc = [DateTime]::UtcNow.ToString("o")
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $ReportPath -Encoding utf8
    Write-Output ([IO.Path]::GetFullPath($ReportPath))
}
finally {
    if ($installedLayout -and (Test-Path -LiteralPath $installedLayout.uninstaller)) {
        Start-Process -FilePath $installedLayout.uninstaller -ArgumentList "/S" -Wait | Out-Null
    }
    if ($workspaceCreatedByTest -and (Test-Path -LiteralPath $workspaceDirectory)) {
        $appDataRoot = [IO.Path]::GetFullPath($env:APPDATA).TrimEnd('\') + '\'
        $workspaceFullPath = [IO.Path]::GetFullPath($workspaceDirectory)
        if (-not $workspaceFullPath.StartsWith($appDataRoot, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to clean a workspace outside APPDATA: $workspaceFullPath"
        }
        Remove-Item -LiteralPath $workspaceFullPath -Recurse -Force
    }
}
