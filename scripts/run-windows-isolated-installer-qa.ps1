[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$InstallerTestScript,
    [Parameter(Mandatory)][string]$CurrentInstaller,
    [string]$PreviousInstaller,
    [Parameter(Mandatory)][string]$ReportPath
)

$ErrorActionPreference = "Stop"

# Start-Process -Credential can load the requested user's registry profile while
# retaining the caller's environment variables. Resolve the profile from the
# active token SID; Known Folder APIs can return empty paths for a background
# first logon that has not opened Explorer yet.
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$profileKey = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\$sid"
$userProfile = (Get-ItemProperty -LiteralPath $profileKey -Name ProfileImagePath).ProfileImagePath
$userProfile = [Environment]::ExpandEnvironmentVariables($userProfile)
$appData = Join-Path $userProfile "AppData\Roaming"
$localAppData = Join-Path $userProfile "AppData\Local"
$desktop = Join-Path $userProfile "Desktop"

foreach ($directory in @($appData, $localAppData, $desktop)) {
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
}

$env:USERPROFILE = $userProfile
$env:APPDATA = $appData
$env:LOCALAPPDATA = $localAppData
$env:HOMEDRIVE = [IO.Path]::GetPathRoot($userProfile).TrimEnd('\')
$env:HOMEPATH = $userProfile.Substring($env:HOMEDRIVE.Length)

$arguments = @{
    CurrentInstaller = $CurrentInstaller
    ReportPath = $ReportPath
}
if ($PreviousInstaller) {
    $arguments.PreviousInstaller = $PreviousInstaller
}

& $InstallerTestScript @arguments
