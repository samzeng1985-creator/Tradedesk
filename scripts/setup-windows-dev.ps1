[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Test-Command {
    param([Parameter(Mandatory)][string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Get-VerifiedInstaller {
    param(
        [Parameter(Mandatory)][string]$Uri,
        [Parameter(Mandatory)][string]$Destination,
        [string]$RequiredSigner
    )

    Invoke-WebRequest -Uri $Uri -OutFile $Destination -UseBasicParsing
    if (-not (Test-Path $Destination)) {
        throw "Installer download failed: $Uri"
    }

    if ($RequiredSigner) {
        $signature = Get-AuthenticodeSignature -FilePath $Destination
        if ($signature.Status -ne "Valid" -or $signature.SignerCertificate.Subject -notmatch $RequiredSigner) {
            throw "Installer signature validation failed: $Destination"
        }
    }
}

$setupRoot = Join-Path $env:TEMP "tradedesk-dev-setup"
New-Item -ItemType Directory -Path $setupRoot -Force | Out-Null

try {
    if (-not (Test-Command "node")) {
        $nodeVersion = "24.19.0"
        $nodeInstaller = Join-Path $setupRoot "node-lts-x64.msi"
        Get-VerifiedInstaller -Uri "https://nodejs.org/dist/v$nodeVersion/node-v$nodeVersion-x64.msi" -Destination $nodeInstaller -RequiredSigner "OpenJS|Node.js"
        $nodeProcess = Start-Process -FilePath "msiexec.exe" -ArgumentList @("/i", $nodeInstaller, "/qn", "/norestart") -Verb RunAs -Wait -PassThru
        if ($nodeProcess.ExitCode -notin @(0, 3010)) {
            throw "Node.js installation failed with exit code $($nodeProcess.ExitCode)"
        }
    }

    if (-not (Test-Command "pnpm")) {
        $npm = Join-Path $env:ProgramFiles "nodejs\npm.cmd"
        if (-not (Test-Path $npm)) { throw "npm was not found after the Node.js installation" }
        & $npm install --global pnpm@11.9.0
        if ($LASTEXITCODE -ne 0) { throw "pnpm installation failed with exit code $LASTEXITCODE" }
    }

    if (-not (Test-Command "rustup")) {
        $rustupInstaller = Join-Path $setupRoot "rustup-init.exe"
        Get-VerifiedInstaller -Uri "https://win.rustup.rs/x86_64" -Destination $rustupInstaller
        & $rustupInstaller -y --profile minimal --default-toolchain stable
        if ($LASTEXITCODE -ne 0) { throw "Rust installation failed with exit code $LASTEXITCODE" }
    }

    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    $hasCppTools = Test-Path $vswhere
    if ($hasCppTools) {
        $installation = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        $hasCppTools = -not [string]::IsNullOrWhiteSpace($installation)
    }

    if (-not $hasCppTools) {
        $vsInstaller = Join-Path $setupRoot "vs_BuildTools.exe"
        Get-VerifiedInstaller -Uri "https://aka.ms/vs/17/release/vs_BuildTools.exe" -Destination $vsInstaller -RequiredSigner "Microsoft"
        $arguments = @(
            "--quiet",
            "--wait",
            "--norestart",
            "--add", "Microsoft.VisualStudio.Workload.VCTools",
            "--includeRecommended"
        )
        $process = Start-Process -FilePath $vsInstaller -ArgumentList $arguments -Verb RunAs -Wait -PassThru
        if ($process.ExitCode -notin @(0, 3010)) {
            throw "Visual Studio Build Tools installation failed with exit code $($process.ExitCode)"
        }
    }

    Write-Host "Development prerequisites installed. Open a new terminal, then run scripts/verify-dev.ps1."
}
finally {
    if (Test-Path $setupRoot) {
        Remove-Item -LiteralPath $setupRoot -Recurse -Force
    }
}
