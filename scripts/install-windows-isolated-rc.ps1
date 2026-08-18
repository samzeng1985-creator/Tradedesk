[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Installer,
    [Parameter(Mandatory)][string]$InstallDirectory,
    [Parameter(Mandatory)][string]$ReportPath
)

$ErrorActionPreference = "Stop"

function Set-IsolatedProfileEnvironment {
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $profileKey = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$sid"
    $profile = (Get-ItemProperty -LiteralPath $profileKey -Name ProfileImagePath).ProfileImagePath
    $profile = [Environment]::ExpandEnvironmentVariables($profile)
    $env:USERPROFILE = $profile
    $env:APPDATA = Join-Path $profile "AppData\Roaming"
    $env:LOCALAPPDATA = Join-Path $profile "AppData\Local"
    $env:HOMEDRIVE = [IO.Path]::GetPathRoot($profile).TrimEnd('\')
    $env:HOMEPATH = $profile.Substring($env:HOMEDRIVE.Length)
    foreach ($directory in @($env:APPDATA, $env:LOCALAPPDATA, (Join-Path $profile "Desktop"))) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
}

Set-IsolatedProfileEnvironment
New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
$process = Start-Process -FilePath $Installer -ArgumentList "/S /D=$InstallDirectory" -Wait -PassThru
if ($process.ExitCode -ne 0) {
    throw "Installer exited with code $($process.ExitCode)"
}

$main = Get-ChildItem -LiteralPath $InstallDirectory -Filter "*.exe" -File |
    Where-Object { $_.Name -notmatch "^(uninstall|typst)" } |
    Select-Object -First 1
if (-not $main) {
    throw "Installed TradeDesk executable was not found"
}

$reportParent = Split-Path ([IO.Path]::GetFullPath($ReportPath)) -Parent
New-Item -ItemType Directory -Path $reportParent -Force | Out-Null
[ordered]@{
    result = "passed"
    user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
    version = $main.VersionInfo.ProductVersion
    executable = $main.FullName
    workspaceDirectory = Join-Path $env:APPDATA "cn.treedeep.tradedesk"
} | ConvertTo-Json | Set-Content -LiteralPath $ReportPath -Encoding utf8

