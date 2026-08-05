[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$missing = [System.Collections.Generic.List[string]]::new()

foreach ($command in @("git", "node", "pnpm", "rustup", "rustc", "cargo")) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        $missing.Add($command)
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

if ($missing.Count -gt 0) {
    Write-Error ("Missing prerequisites: " + ($missing -join ", "))
}

Write-Host "Development environment is ready."
rustc --version
cargo --version
node --version
pnpm --version
