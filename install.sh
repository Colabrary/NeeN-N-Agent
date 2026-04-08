#!/bin/bash
set -e

# NeeN Desktop Agent - One-line installer for macOS & Linux
# Usage: curl -fsSL https://releases.neen.ai/install.sh | bash

RELEASE_URL="https://releases.neen.ai/latest"
APP_NAME="NeeN Desktop Agent"
BUNDLE_ID="com.neen.desktop-agent"
LAUNCH_AGENT_LABEL="com.neen.desktop-agent"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()    { echo -e "${BLUE}[NeeN]${NC} $1"; }
success() { echo -e "${GREEN}[NeeN]${NC} $1"; }
warn()    { echo -e "${YELLOW}[NeeN]${NC} $1"; }
error()   { echo -e "${RED}[NeeN]${NC} $1"; exit 1; }

# ─── Detect OS ───────────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      arm64)  PLATFORM="macos-aarch64" ;;
      x86_64) PLATFORM="macos-x86_64"  ;;
      *)      error "Unsupported Mac architecture: $ARCH" ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64) PLATFORM="linux-x86_64" ;;
      aarch64) PLATFORM="linux-aarch64" ;;
      *)      error "Unsupported Linux architecture: $ARCH" ;;
    esac
    ;;
  *)
    error "Unsupported OS: $OS. For Windows run install.ps1"
    ;;
esac

info "Installing NeeN Desktop Agent for $PLATFORM..."

# ─── macOS Install ───────────────────────────────────────────────────────────
if [ "$OS" = "Darwin" ]; then
  DMG_URL="${RELEASE_URL}/${PLATFORM}/NeeN-Desktop-Agent.dmg"
  DMG_PATH="/tmp/NeeN-Desktop-Agent.dmg"
  MOUNT_PATH="/Volumes/NeeN Desktop Agent"
  INSTALL_PATH="/Applications/NeeN Desktop Agent.app"
  LAUNCH_AGENT_DIR="$HOME/Library/LaunchAgents"
  PLIST_PATH="$LAUNCH_AGENT_DIR/${LAUNCH_AGENT_LABEL}.plist"

  info "Downloading NeeN Desktop Agent..."
  curl -fsSL --progress-bar "$DMG_URL" -o "$DMG_PATH" || error "Download failed. Check your internet connection."

  info "Mounting disk image..."
  hdiutil attach "$DMG_PATH" -quiet -nobrowse

  info "Installing to /Applications..."
  if [ -d "$INSTALL_PATH" ]; then
    rm -rf "$INSTALL_PATH"
  fi
  cp -R "$MOUNT_PATH/NeeN Desktop Agent.app" /Applications/

  info "Unmounting disk image..."
  hdiutil detach "$MOUNT_PATH" -quiet
  rm -f "$DMG_PATH"

  # Remove quarantine flag so macOS doesn't block the app (no Gatekeeper prompt)
  info "Removing quarantine restrictions..."
  xattr -rd com.apple.quarantine "$INSTALL_PATH" 2>/dev/null || true
  xattr -cr "$INSTALL_PATH" 2>/dev/null || true

  # ─── Register as LaunchAgent (auto-start at login) ────────────────────────
  info "Registering as login service..."
  mkdir -p "$LAUNCH_AGENT_DIR"

  cat > "$PLIST_PATH" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${LAUNCH_AGENT_LABEL}</string>
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
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
    </dict>
    <key>StandardOutPath</key>
    <string>${HOME}/Library/Logs/neen-desktop-agent.log</string>
    <key>StandardErrorPath</key>
    <string>${HOME}/Library/Logs/neen-desktop-agent-error.log</string>
</dict>
</plist>
PLIST

  # Unload previous version if running
  launchctl unload "$PLIST_PATH" 2>/dev/null || true

  # Load and start the agent
  launchctl load -w "$PLIST_PATH"
  launchctl start "$LAUNCH_AGENT_LABEL" 2>/dev/null || true

  success "NeeN Desktop Agent installed and started!"
  info "It will automatically launch at every login."
  info "Logs: ~/Library/Logs/neen-desktop-agent.log"
  info ""
  info "To stop:    launchctl stop ${LAUNCH_AGENT_LABEL}"
  info "To disable: launchctl unload -w ${LAUNCH_AGENT_LABEL}.plist"

# ─── Linux Install ───────────────────────────────────────────────────────────
elif [ "$OS" = "Linux" ]; then
  APPIMAGE_URL="${RELEASE_URL}/${PLATFORM}/NeeN-Desktop-Agent.AppImage"
  INSTALL_DIR="$HOME/.local/bin"
  APPIMAGE_PATH="$INSTALL_DIR/neen-desktop-agent"
  AUTOSTART_DIR="$HOME/.config/autostart"
  DESKTOP_PATH="$AUTOSTART_DIR/neen-desktop-agent.desktop"

  mkdir -p "$INSTALL_DIR" "$AUTOSTART_DIR"

  info "Downloading NeeN Desktop Agent..."
  curl -fsSL --progress-bar "$APPIMAGE_URL" -o "$APPIMAGE_PATH" || error "Download failed."
  chmod +x "$APPIMAGE_PATH"

  # Register as autostart via XDG
  cat > "$DESKTOP_PATH" <<DESKTOP
[Desktop Entry]
Type=Application
Name=NeeN Desktop Agent
Exec=${APPIMAGE_PATH}
Icon=neen-desktop-agent
Comment=NeeN AI Desktop Agent
Categories=Utility;
X-GNOME-Autostart-enabled=true
Hidden=false
NoDisplay=false
DESKTOP

  # Start immediately
  nohup "$APPIMAGE_PATH" > "$HOME/.local/share/neen-desktop-agent.log" 2>&1 &

  success "NeeN Desktop Agent installed and started!"
  info "It will automatically launch at every login."
fi
