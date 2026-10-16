#!/bin/sh
# aruaru-IME installer for macOS and Linux (user-level, no root needed).
# Installs the resident engine + CLI, downloads the dictionary, sets up autostart, and opens the screen.
set -eu
APP="$HOME/.local/share/aruaru-ime"
BIN="$APP/bin"
DATA="${ARUARU_DATA_DIR:-}"
OS="$(uname -s)"
HERE="$(cd "$(dirname "$0")" && pwd)"
DICT_URL="https://easy-web.tokyo/ime/download/dict.bin"

echo "== aruaru-IME installer ($OS) =="
mkdir -p "$BIN"
cp "$HERE/aruaru-ime-daemon" "$HERE/aruaru-ime" "$BIN/"
chmod +x "$BIN/aruaru-ime-daemon" "$BIN/aruaru-ime"
# Linux: IBus engine (system-wide IME for GTK/Qt/browsers). Needs IBus; skipped if the binary is not in the package.
# Linux: Fcitx5 addon (needs a root copy of the .so into the system fcitx5 addon folder; the config files go to the user folder).
if [ "$OS" = "Linux" ] && [ -f "$HERE/fcitx5/aruaru-ime.so" ]; then
  FD="${XDG_DATA_HOME:-$HOME/.local/share}/fcitx5"
  mkdir -p "$FD/addon" "$FD/inputmethod"
  cp "$HERE/fcitx5/aruaru-ime-addon.conf" "$FD/addon/aruaru-ime.conf"; cp "$HERE/fcitx5/aruaru-ime-im.conf" "$FD/inputmethod/aruaru-ime.conf"
  FL="$(ls -d /usr/lib/x86_64-linux-gnu/fcitx5 /usr/lib64/fcitx5 /usr/lib/fcitx5 2>/dev/null | head -1)"
  if [ -n "$FL" ] && command -v fcitx5 >/dev/null 2>&1; then
    if [ -w "$FL" ]; then cp "$HERE/fcitx5/aruaru-ime.so" "$FL/"; elif command -v sudo >/dev/null 2>&1 && sudo -n true 2>/dev/null; then sudo cp "$HERE/fcitx5/aruaru-ime.so" "$FL/"; else echo "Fcitx5: run once:  sudo cp \"$HERE/fcitx5/aruaru-ime.so\" \"$FL/\""; fi
    echo "Fcitx5: add aruaru-IME in fcitx5-configtool, then restart fcitx5 (fcitx5 -r)."
  fi
fi
if [ "$OS" = "Linux" ] && [ -f "$HERE/aruaru-ime-ibus" ]; then
  cp "$HERE/aruaru-ime-ibus" "$BIN/"; chmod +x "$BIN/aruaru-ime-ibus"
  CD="${XDG_DATA_HOME:-$HOME/.local/share}/ibus/component"
  mkdir -p "$CD"
  cat > "$CD/aruaru-ime.xml" <<XML
<component>
  <name>org.freedesktop.IBus.AruaruIME</name>
  <description>aruaru-IME</description>
  <exec>$BIN/aruaru-ime-ibus --ibus</exec>
  <version>0.2</version><author>石塚正浩（MASAHIRO ISHIZUKA）</author><license>MIT</license>
  <homepage>https://github.com/aon-co-jp/aruaru-IME</homepage><textdomain>aruaru-ime</textdomain>
  <engines><engine><name>aruaru-ime</name><language>ja</language><license>MIT</license><author>石塚正浩（MASAHIRO ISHIZUKA）</author><icon></icon><layout>default</layout><longname>aruaru-IME</longname><description>AI IME</description><rank>80</rank></engine></engines>
</component>
XML
  if command -v ibus >/dev/null 2>&1; then ibus write-cache >/dev/null 2>&1 || true; ibus restart >/dev/null 2>&1 || true; fi
  echo "IBus: registered. Add it in Settings > Keyboard > Input Sources > Japanese > aruaru-IME (or: ibus engine aruaru-ime)."
fi

# macOS: input method (InputMethodKit). Log out and in, then add it in System Settings > Keyboard > Input Sources > Japanese.
if [ "$OS" = "Darwin" ] && [ -d "$HERE/aruaru-IME.app" ]; then
  mkdir -p "$HOME/Library/Input Methods"
  rm -rf "$HOME/Library/Input Methods/aruaru-IME.app"
  cp -R "$HERE/aruaru-IME.app" "$HOME/Library/Input Methods/"
  echo "macOS input method installed. Log out and in, then add aruaru-IME in System Settings > Keyboard > Input Sources > Japanese."
fi

# data dir (same rule as the engine)
if [ -z "$DATA" ]; then
  if [ "$OS" = "Darwin" ]; then DATA="$HOME/Library/Application Support/aruaru"; else DATA="${XDG_DATA_HOME:-$HOME/.local/share}/aruaru"; fi
fi
mkdir -p "$DATA"
if [ ! -s "$DATA/dict.bin" ]; then
  echo "Downloading the dictionary (about 64 MB)..."
  if command -v curl >/dev/null 2>&1; then curl -fL --progress-bar -o "$DATA/dict.bin.tmp" "$DICT_URL"; else wget -O "$DATA/dict.bin.tmp" "$DICT_URL"; fi
  mv "$DATA/dict.bin.tmp" "$DATA/dict.bin"
fi

# autostart
if [ "$OS" = "Darwin" ]; then
  PL="$HOME/Library/LaunchAgents/jp.aon.aruaru-ime.plist"
  mkdir -p "$HOME/Library/LaunchAgents"
  cat > "$PL" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>jp.aon.aruaru-ime</string>
<key>ProgramArguments</key><array><string>$BIN/aruaru-ime-daemon</string></array>
<key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
</dict></plist>
PLIST
  launchctl unload "$PL" 2>/dev/null || true
  launchctl load "$PL"
else
  UD="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
  if command -v systemctl >/dev/null 2>&1 && systemctl --user show-environment >/dev/null 2>&1; then
    mkdir -p "$UD"
    cat > "$UD/aruaru-ime.service" <<UNIT
[Unit]
Description=aruaru-IME engine
[Service]
ExecStart=$BIN/aruaru-ime-daemon
Restart=always
[Install]
WantedBy=default.target
UNIT
    systemctl --user daemon-reload
    systemctl --user enable --now aruaru-ime.service
  else
    AD="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
    mkdir -p "$AD"
    printf '[Desktop Entry]\nType=Application\nName=aruaru-IME\nExec=%s/aruaru-ime-daemon\nX-GNOME-Autostart-enabled=true\n' "$BIN" > "$AD/aruaru-ime.desktop"
    (nohup "$BIN/aruaru-ime-daemon" >/dev/null 2>&1 &)
  fi
fi

sleep 2
if command -v curl >/dev/null 2>&1 && curl -fs http://127.0.0.1:4660/health >/dev/null; then
  echo "OK: the engine is running."
else
  echo "The engine did not answer yet; it will start at the next login."
fi
URL="http://127.0.0.1:4660/ui"
echo "Open $URL  (you can type there; this build has no system-wide IME yet — see README)."
if [ "$OS" = "Darwin" ]; then open "$URL" || true; else (xdg-open "$URL" >/dev/null 2>&1 || true); fi
