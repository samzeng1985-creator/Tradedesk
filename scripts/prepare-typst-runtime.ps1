[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$projectRoot = Split-Path $PSScriptRoot -Parent
$typstRoot = Join-Path $projectRoot "tools\typst"
$runtimePath = Join-Path $typstRoot "typst.exe"
$version = "0.15.1"
$expectedHash = "19CE3551153C2FE7EE9FA2F95208310C8F4D3209FEDB699E0333FAF8913F6736"

New-Item -ItemType Directory -Path $typstRoot -Force | Out-Null

if (-not (Test-Path -LiteralPath $runtimePath)) {
    $installedRuntime = Get-ChildItem -Path $typstRoot -Recurse -Filter "typst.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -ne $runtimePath } |
        Select-Object -First 1

    if (-not $installedRuntime) {
        $archivePath = Join-Path $env:TEMP "tradedesk-typst-$version-$([guid]::NewGuid().ToString('N')).zip"
        try {
            Invoke-WebRequest -Uri "https://github.com/typst/typst/releases/download/v$version/typst-x86_64-pc-windows-msvc.zip" -OutFile $archivePath -UseBasicParsing
            if ((Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash -ne $expectedHash) {
                throw "Typst archive checksum validation failed"
            }
            Expand-Archive -LiteralPath $archivePath -DestinationPath $typstRoot -Force
        }
        finally {
            if (Test-Path -LiteralPath $archivePath) {
                Remove-Item -LiteralPath $archivePath -Force
            }
        }
        $installedRuntime = Get-ChildItem -Path $typstRoot -Recurse -Filter "typst.exe" |
            Where-Object { $_.FullName -ne $runtimePath } |
            Select-Object -First 1
    }

    if (-not $installedRuntime) { throw "Typst runtime was not found after preparation" }
    Copy-Item -LiteralPath $installedRuntime.FullName -Destination $runtimePath
    foreach ($asset in @(@("LICENSE", "TYPST-LICENSE.txt"), @("NOTICE", "TYPST-NOTICE.txt"))) {
        $source = Join-Path $installedRuntime.DirectoryName $asset[0]
        if (-not (Test-Path -LiteralPath $source)) { throw "Missing Typst $($asset[0]) file" }
        Copy-Item -LiteralPath $source -Destination (Join-Path $typstRoot $asset[1])
    }
}

foreach ($required in @("TYPST-LICENSE.txt", "TYPST-NOTICE.txt")) {
    if (-not (Test-Path -LiteralPath (Join-Path $typstRoot $required))) {
        throw "Missing bundled Typst notice: $required"
    }
}

& $runtimePath --version
