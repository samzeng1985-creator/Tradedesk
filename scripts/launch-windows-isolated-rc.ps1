[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Executable,
    [Parameter(Mandatory)][int]$DebugPort,
    [Parameter(Mandatory)][string]$PidReportPath
)

$ErrorActionPreference = "Stop"
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$profileKey = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$sid"
$profile = (Get-ItemProperty -LiteralPath $profileKey -Name ProfileImagePath).ProfileImagePath
$profile = [Environment]::ExpandEnvironmentVariables($profile)
$env:USERPROFILE = $profile
$env:APPDATA = Join-Path $profile "AppData\Roaming"
$env:LOCALAPPDATA = Join-Path $profile "AppData\Local"
$env:HOMEDRIVE = [IO.Path]::GetPathRoot($profile).TrimEnd('\')
$env:HOMEPATH = $profile.Substring($env:HOMEDRIVE.Length)
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$DebugPort --remote-allow-origins=*"

$process = Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
[ordered]@{
    user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
    processId = $process.Id
    debugPort = $DebugPort
    startedAtUtc = [DateTime]::UtcNow.ToString("o")
} | ConvertTo-Json | Set-Content -LiteralPath $PidReportPath -Encoding utf8
$process.WaitForExit()

