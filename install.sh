#!/bin/bash
set -e

# NeeN Desktop Agent - One-line installer for macOS & Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/Colabrary/NeeN-N-Agent/main/install.sh | bash

REPO_RAW="https://raw.githubusercontent.com/Colabrary/NeeN-N-Agent/main"
RELEASE_BASE="https://github.com/Colabrary/NeeN-N-Agent/releases/latest/download"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()    { echo -e "${BLUE}[NeeN]${NC} $1"; }
success() { echo -e "${GREEN}[NeeN]${NC} $1"; }
error()   { echo -e "${RED}[NeeN]${NC} $1"; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"

# ─── macOS ────────────────────────────────────────────────────────────────────
if [ "$OS" = "Darwin" ]; then
    DMG_URL="${RELEASE_BASE}/NeeN-Desktop-Agent.dmg"
    DMG_PATH="/tmp/NeeN-Desktop-Agent.dmg"
    INSTALL_PATH="/Applications/NeeN Desktop Agent.app"
    MOUNT_PATH="/Volumes/NeeN Desktop Agent"
    PLIST_DIR="$HOME/Library/LaunchAgents"
    PLIST_PATH="$PLIST_DIR/com.neen.desktop-agent.plist"
    BUNDLE_ID="com.neen.desktop-agent"

    info "Detected macOS ($ARCH)"
    info "Downloading NeeN Desktop Agent..."
    curl -fsSL --progress-bar "$DMG_URL" -o "$DMG_PATH" || error "Download failed. Check your internet."

    info "Installing..."
    hdiutil attach "$DMG_PATH" -quiet -nobrowse

    [ -d "$INSTALL_PATH" ] && rm -rf "$INSTALL_PATH"
    cp -R "/Volumes/NeeN Desktop Agent/NeeN Desktop Agent.app" /Applications/ 2>/dev/null \
        || cp -R "$MOUNT_PATH/"*.app /Applications/ 2>/dev/null \
        || error "Could not copy app from DMG"

    hdiutil detach "$MOUNT_PATH" -quiet 2>/dev/null || true
    rm -f "$DMG_PATH"

    # Remove Gatekeeper quarantine — no verification popup
    info "Removing quarantine restrictions..."
    xattr -rd com.apple.quarantine "$INSTALL_PATH" 2>/dev/null || true
    xattr -cr "$INSTALL_PATH" 2>/dev/null || true

    # Register as LaunchAgent — auto-starts at login
    info "Registering as login service..."
    mkdir -p "$PLIST_DIR"
    launchctl unload "$PLIST_PATH" 2>/dev/null || true

    cat > "$PLIST_PATH" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${BUNDLE_ID}</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Applications/NeeN Desktop Agent.app/Contents/MacOS/neen-desktop-agent</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>Crashed</key>
        <true/>
    </dict>
    <key>ProcessType</key>
    <string>Interactive</string>
    <key>StandardOutPath</key>
    <string>${HOME}/Library/Logs/neen-desktop-agent.log</string>
    <key>StandardErrorPath</key>
    <string>${HOME}/Library/Logs/neen-desktop-agent-error.log</string>
</dict>
</plist>
PLIST

    launchctl load -w "$PLIST_PATH"
    launchctl start "$BUNDLE_ID" 2>/dev/null || true

    success "NeeN Desktop Agent installed and running!"
    info "Starts automatically at every login."
    info "Logs: ~/Library/Logs/neen-desktop-agent.log"

# ─── Linux ────────────────────────────────────────────────────────────────────
elif [ "$OS" = "Linux" ]; then
    APPIMAGE_URL="${RELEASE_BASE}/NeeN-Desktop-Agent.AppImage"
    INSTALL_DIR="$HOME/.local/bin"
    APPIMAGE_PATH="$INSTALL_DIR/neen-desktop-agent"
    AUTOSTART_DIR="$HOME/.config/autostart"
    DESKTOP_PATH="$AUTOSTART_DIR/neen-desktop-agent.desktop"

    info "Detected Linux ($ARCH)"
    mkdir -p "$INSTALL_DIR" "$AUTOSTART_DIR"

    info "Downloading NeeN Desktop Agent..."
    curl -fsSL --progress-bar "$APPIMAGE_URL" -o "$APPIMAGE_PATH" || error "Download failed. Check your internet."
    chmod +x "$APPIMAGE_PATH"

    # XDG autostart — starts at login for GNOME/KDE/XFCE etc.
    cat > "$DESKTOP_PATH" <<DESKTOP
[Desktop Entry]
Type=Application
Name=NeeN Desktop Agent
Exec=${APPIMAGE_PATH}
Comment=NeeN AI Desktop Agent
Categories=Utility;
X-GNOME-Autostart-enabled=true
Hidden=false
NoDisplay=false
DESKTOP

    info "Starting NeeN Desktop Agent..."
    nohup "$APPIMAGE_PATH" > "$HOME/.local/share/neen-agent.log" 2>&1 &

    success "NeeN Desktop Agent installed and running!"
    info "Starts automatically at every login."
    info "Log: ~/.local/share/neen-agent.log"

else
    echo "Unsupported OS: $OS"
    echo "For Windows, run this in PowerShell:"
    echo '  iwr -useb https://raw.githubusercontent.com/Colabrary/NeeN-N-Agent/main/install.ps1 | iex'
    exit 1
fi
