[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$missing = [System.Collections.Generic.List[string]]::new()
$projectRoot = Split-Path $PSScriptRoot -Parent

foreach ($command in @("git", "node", "pnpm.cmd", "rustup", "rustc", "cargo", "perl")) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        $missing.Add($command)
    }
}

$nasm = Join-Path $env:LOCALAPPDATA "bin\NASM\nasm.exe"
if (-not (Test-Path $nasm)) {
    $nasmCommand = Get-Command "nasm" -ErrorAction SilentlyContinue
    $nasm = if ($nasmCommand) { $nasmCommand.Source } else { $null }
}
if (-not $nasm) {
    $missing.Add("NASM")
}

$perl = Get-Command "perl" -ErrorAction SilentlyContinue
if ($perl) {
    & $perl.Source -MLocale::Maketext::Simple -e "exit 0"
    if ($LASTEXITCODE -ne 0) {
        $missing.Add("Full Strawberry Perl modules")
    }
}

$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
$cppPath = $null
if (Test-Path $vswhere) {
    $cppPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
}
if ([string]::IsNullOrWhiteSpace($cppPath)) {
    $missing.Add("Microsoft C++ Build Tools")
}

$webViewRoots = @(
    "C:\Program Files (x86)\Microsoft\EdgeWebView\Application",
    "C:\Program Files\Microsoft\EdgeWebView\Application"
)
if (-not ($webViewRoots | Where-Object { Test-Path $_ })) {
    $missing.Add("Microsoft Edge WebView2 Runtime")
}

$typst = Get-ChildItem -Path (Join-Path $projectRoot "tools\typst") -Recurse -Filter "typst.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $typst) {
    $missing.Add("Typst 0.15.1 PDF renderer")
}

if ($missing.Count -gt 0) {
    Write-Error ("Missing prerequisites: " + ($missing -join ", "))
}

Write-Host "Development environment is ready."
rustc --version
cargo --version
node --version
pnpm.cmd --version
perl --version
if ($nasm) { & $nasm -v }
if ($typst) { & $typst.FullName --version }
