[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][string]$DriverScript,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [int]$SeedPort = 9338,
    [int]$VerifyPort = 9339
)

$ErrorActionPreference = "Stop"

function Resolve-ExactQaDirectory {
    param(
        [Parameter(Mandatory)][string]$BaseDirectory,
        [Parameter(Mandatory)][string]$ExpectedLeaf
    )
    $base = [IO.Path]::GetFullPath($BaseDirectory).TrimEnd('\')
    $candidate = [IO.Path]::GetFullPath((Join-Path $base $ExpectedLeaf)).TrimEnd('\')
    $expected = "$base\$ExpectedLeaf"
    if (-not $candidate.Equals($expected, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing unexpected RC2 QA path: $candidate"
    }
    return $candidate
}

function Remove-ExactQaDirectory {
    param([Parameter(Mandatory)][string]$Path)
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Recurse -Force
    }
}

function Start-QaApplication {
    param(
        [Parameter(Mandatory)][string]$Application,
        [Parameter(Mandatory)][int]$DebugPort
    )
    $previousArguments = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
    try {
        $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort --remote-allow-origins=*"
        return Start-Process -FilePath $Application -WindowStyle Hidden -PassThru
    } finally {
        if ($null -eq $previousArguments) {
            Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
        } else {
            $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $previousArguments
        }
    }
}

function Stop-QaApplication {
    param([Diagnostics.Process]$Process)
    if ($null -eq $Process) { return }
    $Process.Refresh()
    if (-not $Process.HasExited) {
        Stop-Process -Id $Process.Id -Force
        $Process.WaitForExit(10000) | Out-Null
    }
}

$application = (Resolve-Path -LiteralPath $Executable).Path
$driver = (Resolve-Path -LiteralPath $DriverScript).Path
$version = (Get-Item -LiteralPath $application).VersionInfo.ProductVersion
if ($version -notlike "0.28.0-rc.2*") {
    throw "Expected a 0.28.0-rc.2 executable, got $version"
}

$output = [IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $output -Force | Out-Null
$screenshots = Join-Path $output "screenshots"
$seedReport = Join-Path $output "seed.json"
$verifyReport = Join-Path $output "verify.json"
$qaLeaf = "cn.treedeep.tradedesk.rc2qa"
$qaRoaming = Resolve-ExactQaDirectory -BaseDirectory $env:APPDATA -ExpectedLeaf $qaLeaf
$qaLocal = Resolve-ExactQaDirectory -BaseDirectory $env:LOCALAPPDATA -ExpectedLeaf $qaLeaf

Remove-ExactQaDirectory -Path $qaRoaming
Remove-ExactQaDirectory -Path $qaLocal

$process = $null
try {
    $process = Start-QaApplication -Application $application -DebugPort $SeedPort
    & node $driver seed $SeedPort $seedReport $screenshots
    if ($LASTEXITCODE -ne 0) { throw "RC2 UI seed interaction failed with exit code $LASTEXITCODE" }
    Stop-QaApplication -Process $process
    $process = $null

    $process = Start-QaApplication -Application $application -DebugPort $VerifyPort
    & node $driver verify $VerifyPort $verifyReport $screenshots
    if ($LASTEXITCODE -ne 0) { throw "RC2 UI persistence verification failed with exit code $LASTEXITCODE" }
    Stop-QaApplication -Process $process
    $process = $null

    $utf8 = [Text.UTF8Encoding]::new($false)
    $seed = [IO.File]::ReadAllText($seedReport, $utf8) | ConvertFrom-Json
    $verify = [IO.File]::ReadAllText($verifyReport, $utf8) | ConvertFrom-Json
    $aggregate = [ordered]@{
        result = if ($seed.result -eq "passed" -and $verify.result -eq "passed") { "passed" } else { "failed" }
        version = $version
        executable = $application
        executableSha256 = (Get-FileHash -LiteralPath $application -Algorithm SHA256).Hash.ToLowerInvariant()
        qaIdentifier = $qaLeaf
        seed = $seed
        verify = $verify
        completedAtUtc = [DateTime]::UtcNow.ToString("o")
    }
    $aggregate | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath (Join-Path $output "windows-ui-e2e-report.json") -Encoding utf8
    $aggregate | ConvertTo-Json -Depth 4
} finally {
    Stop-QaApplication -Process $process
    Remove-ExactQaDirectory -Path $qaRoaming
    Remove-ExactQaDirectory -Path $qaLocal
}
