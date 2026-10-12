#!/bin/sh
# aruaru-IME uninstaller (macOS / Linux). Keeps your data (dictionaries, learning) unless you pass --purge.
set -u
OS="$(uname -s)"
if [ "$OS" = "Darwin" ]; then
  PL="$HOME/Library/LaunchAgents/jp.aon.aruaru-ime.plist"
  launchctl unload "$PL" 2>/dev/null || true
  rm -f "$PL"
  DATA="$HOME/Library/Application Support/aruaru"
else
  systemctl --user disable --now aruaru-ime.service 2>/dev/null || true
  rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/aruaru-ime.service" "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/aruaru-ime.desktop"
  DATA="${XDG_DATA_HOME:-$HOME/.local/share}/aruaru"
fi
pkill -f aruaru-ime-daemon 2>/dev/null || true
rm -f "${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component/aruaru-ime.xml"; command -v ibus >/dev/null 2>&1 && ibus write-cache >/dev/null 2>&1 && ibus restart >/dev/null 2>&1 || true
rm -rf "$HOME/.local/share/aruaru-ime"
if [ "${1:-}" = "--purge" ]; then rm -rf "$DATA"; echo "Removed engine and data."; else echo "Removed engine. Data kept in: $DATA (use --purge to delete)."; fi
