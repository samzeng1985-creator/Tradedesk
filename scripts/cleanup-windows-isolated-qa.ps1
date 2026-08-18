[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$StagingDirectory,
    [Parameter(Mandatory)][string]$CredentialDirectory,
    [Parameter(Mandatory)][string]$ProjectRoot
)

$ErrorActionPreference = "Stop"
$staging = [IO.Path]::GetFullPath($StagingDirectory).TrimEnd('\')
$expectedStaging = [IO.Path]::GetFullPath("C:\Users\Public\Documents\TradeDesk-RC1-QA").TrimEnd('\')
if (-not $staging.Equals($expectedStaging, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to clean unexpected QA staging path: $staging"
}

$project = [IO.Path]::GetFullPath($ProjectRoot).TrimEnd('\')
$credential = [IO.Path]::GetFullPath($CredentialDirectory).TrimEnd('\')
$expectedCredential = Join-Path $project "tmp\windows-rc1-qa"
if (-not $credential.Equals($expectedCredential, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to clean unexpected QA credential path: $credential"
}

if (Test-Path -LiteralPath $staging) {
    Remove-Item -LiteralPath $staging -Recurse -Force
}
if (Test-Path -LiteralPath $credential) {
    Remove-Item -LiteralPath $credential -Recurse -Force
}

[ordered]@{
    stagingRemoved = -not (Test-Path -LiteralPath $staging)
    credentialRemoved = -not (Test-Path -LiteralPath $credential)
} | ConvertTo-Json
