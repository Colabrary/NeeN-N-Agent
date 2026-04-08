# NeeN Desktop Agent - One-line installer for Windows
# Usage: iwr -useb https://releases.neen.ai/install.ps1 | iex
# Or:    curl.exe -fsSL https://releases.neen.ai/install.ps1 | powershell -

param(
    [string]$ReleaseUrl = "https://github.com/Colabrary/NeeN-N-Agent/releases/latest/download",
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$AppName    = "NeeN Desktop Agent"
$ExeName    = "neen-desktop-agent.exe"
$InstallDir = "$env:LOCALAPPDATA\NeeN\Desktop Agent"
$StartupKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$TaskName   = "NeeN Desktop Agent"
$LogDir     = "$env:LOCALAPPDATA\NeeN\Logs"

function Write-Info    { Write-Host "[NeeN] $args" -ForegroundColor Cyan }
function Write-Success { Write-Host "[NeeN] $args" -ForegroundColor Green }
function Write-Warn    { Write-Host "[NeeN] $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "[NeeN] $args" -ForegroundColor Red; exit 1 }

# ─── Uninstall mode ──────────────────────────────────────────────────────────
if ($Uninstall) {
    Write-Info "Uninstalling NeeN Desktop Agent..."
    Stop-Process -Name "neen-desktop-agent" -Force -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path $StartupKey -Name $AppName -ErrorAction SilentlyContinue
    if (Test-Path $InstallDir) { Remove-Item $InstallDir -Recurse -Force }
    Write-Success "NeeN Desktop Agent uninstalled."
    exit 0
}

# ─── Detect arch ─────────────────────────────────────────────────────────────
$arch = if ([System.Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "windows-aarch64" } else { "windows-x86_64" }
} else {
    Write-Fail "32-bit Windows is not supported."
}

Write-Info "Installing $AppName for $arch..."

# ─── Download ─────────────────────────────────────────────────────────────────
$InstallerUrl  = "$ReleaseUrl/$arch/NeeN-Desktop-Agent-Setup.exe"
$InstallerPath = "$env:TEMP\NeeN-Desktop-Agent-Setup.exe"

Write-Info "Downloading installer..."
try {
    $ProgressPreference = "SilentlyContinue"  # Makes Invoke-WebRequest much faster
    Invoke-WebRequest -Uri $InstallerUrl -OutFile $InstallerPath -UseBasicParsing
} catch {
    Write-Fail "Download failed: $_"
}

# ─── Silent install (NSIS/WiX installer with /S flag) ────────────────────────
Write-Info "Running silent install..."
$proc = Start-Process -FilePath $InstallerPath -ArgumentList "/S /D=$InstallDir" -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Fail "Installer failed with exit code $($proc.ExitCode)"
}
Remove-Item $InstallerPath -Force -ErrorAction SilentlyContinue

$ExePath = "$InstallDir\$ExeName"
if (-not (Test-Path $ExePath)) {
    Write-Fail "Install completed but executable not found at: $ExePath"
}

# ─── Register as startup service via Task Scheduler ──────────────────────────
Write-Info "Registering as startup service..."
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

# Remove old task if exists
Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue

# Create scheduled task that runs at login, restarts on crash
$Action  = New-ScheduledTaskAction -Execute $ExePath
$Trigger = New-ScheduledTaskTrigger -AtLogOn

$Settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit 0 `
    -RestartCount 10 `
    -RestartInterval (New-TimeSpan -Minutes 1)

$Principal = New-ScheduledTaskPrincipal `
    -UserId "$env:USERDOMAIN\$env:USERNAME" `
    -LogonType Interactive `
    -RunLevel Limited

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $Action `
    -Trigger $Trigger `
    -Settings $Settings `
    -Principal $Principal `
    -Description "NeeN AI Desktop Agent - Auto-start at login" `
    -Force | Out-Null

# Also add registry fallback for reliability
Set-ItemProperty -Path $StartupKey -Name $AppName -Value "`"$ExePath`"" -Force

Write-Info "Starting NeeN Desktop Agent..."
Start-Process -FilePath $ExePath -WindowStyle Hidden

Write-Success "$AppName installed and started!"
Write-Info "It will automatically launch at every Windows login."
Write-Info "Log folder: $LogDir"
Write-Info ""
Write-Info "To stop:      Stop-Process -Name neen-desktop-agent"
Write-Info "To uninstall: iwr -useb https://releases.neen.ai/install.ps1 | iex -Uninstall"
