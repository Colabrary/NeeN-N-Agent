# NeeN Desktop Agent - One-line installer for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/Colabrary/NeeN-N-Agent/main/install.ps1 | iex

param([switch]$Uninstall)

$ErrorActionPreference = "Stop"

$RepoRaw     = "https://raw.githubusercontent.com/Colabrary/NeeN-N-Agent/main"
$ReleaseBase = "https://github.com/Colabrary/NeeN-N-Agent/releases/latest/download"
$AppName    = "NeeN Desktop Agent"
$ExeName    = "neen-desktop-agent.exe"
$InstallDir = "$env:LOCALAPPDATA\NeeN\Desktop Agent"
$TaskName   = "NeeN Desktop Agent"
$LogDir     = "$env:LOCALAPPDATA\NeeN\Logs"

function Write-Info    { Write-Host "[NeeN] $args" -ForegroundColor Cyan }
function Write-Success { Write-Host "[NeeN] $args" -ForegroundColor Green }
function Write-Fail    { Write-Host "[NeeN] ERROR: $args" -ForegroundColor Red; exit 1 }

# ─── Uninstall ────────────────────────────────────────────────────────────────
if ($Uninstall) {
    Write-Info "Uninstalling NeeN Desktop Agent..."
    Stop-Process -Name "neen-desktop-agent" -Force -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue
    Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name $AppName -ErrorAction SilentlyContinue
    if (Test-Path $InstallDir) { Remove-Item $InstallDir -Recurse -Force }
    Write-Success "NeeN Desktop Agent uninstalled."
    exit 0
}

# ─── Download ─────────────────────────────────────────────────────────────────
$InstallerUrl  = "$ReleaseBase/NeeN-Desktop-Agent-Setup.exe"
$InstallerPath = "$env:TEMP\NeeN-Desktop-Agent-Setup.exe"

Write-Info "Downloading NeeN Desktop Agent for Windows..."
try {
    $ProgressPreference = "SilentlyContinue"
    Invoke-WebRequest -Uri $InstallerUrl -OutFile $InstallerPath -UseBasicParsing
} catch {
    Write-Fail "Download failed: $_`nMake sure you have internet access."
}

# ─── Silent install ───────────────────────────────────────────────────────────
Write-Info "Installing silently (no popups)..."
$proc = Start-Process -FilePath $InstallerPath -ArgumentList "/S /D=$InstallDir" -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Fail "Installer exited with code $($proc.ExitCode)"
}
Remove-Item $InstallerPath -Force -ErrorAction SilentlyContinue

$ExePath = "$InstallDir\$ExeName"
if (-not (Test-Path $ExePath)) {
    Write-Fail "Install finished but exe not found at: $ExePath"
}

# ─── Register as startup via Task Scheduler ───────────────────────────────────
Write-Info "Registering as startup service..."
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false -ErrorAction SilentlyContinue

$Action    = New-ScheduledTaskAction -Execute $ExePath
$Trigger   = New-ScheduledTaskTrigger -AtLogOn
$Settings  = New-ScheduledTaskSettingsSet `
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
    -Description "NeeN AI Desktop Agent — auto-start at login" `
    -Force | Out-Null

# Registry fallback
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" `
    -Name $AppName -Value "`"$ExePath`"" -Force

Write-Info "Starting NeeN Desktop Agent..."
Start-Process -FilePath $ExePath -WindowStyle Hidden

Write-Success "$AppName installed and running!"
Write-Info "Starts automatically at every Windows login."
Write-Info "To uninstall: iwr -useb $RepoRaw/install.ps1 | iex -Uninstall"
