[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$ArtifactPath,
    [Parameter(Mandatory)][ValidateSet("windows-x64", "macos-arm64", "macos-x64")][string]$Platform,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path $PSScriptRoot -Parent
$artifactMatches = @(Resolve-Path -Path $ArtifactPath)
if ($artifactMatches.Count -ne 1) {
    throw "ArtifactPath must resolve to exactly one file; found $($artifactMatches.Count)"
}
$resolvedArtifact = $artifactMatches[0].Path
$package = Get-Content -LiteralPath (Join-Path $projectRoot "package.json") -Raw | ConvertFrom-Json
$cargo = Get-Content -LiteralPath (Join-Path $projectRoot "src-tauri\Cargo.toml") -Raw
$cargoVersion = [regex]::Match($cargo, '(?m)^version = "([^"]+)"$').Groups[1].Value

if ($package.version -ne $cargoVersion) {
    throw "Frontend and Rust versions do not match: $($package.version) / $cargoVersion"
}

if (-not $OutputPath) {
    $OutputPath = Join-Path (Split-Path $resolvedArtifact -Parent) "release-manifest.json"
}

$manifest = [ordered]@{
    product = "TradeDesk Local"
    version = $package.version
    platform = $Platform
    artifact = Split-Path $resolvedArtifact -Leaf
    bytes = (Get-Item -LiteralPath $resolvedArtifact).Length
    sha256 = (Get-FileHash -LiteralPath $resolvedArtifact -Algorithm SHA256).Hash.ToLowerInvariant()
    signed = $false
    notarized = $false
    sqlcipherSchema = 15
    typstVersion = "0.15.1"
    generatedAtUtc = [DateTime]::UtcNow.ToString("o")
}

$manifest | ConvertTo-Json | Set-Content -LiteralPath $OutputPath -Encoding utf8
Write-Output $OutputPath
