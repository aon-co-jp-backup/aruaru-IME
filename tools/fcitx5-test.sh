#!/bin/bash
# Fcitx5 addon end-to-end test (Linux). Needs: fcitx5, python3-dbus, python3-gi, dbus, the built addon and the daemon.
# Usage: tools/fcitx5-test.sh <fcitx5 addon build dir containing aruaru-ime.so> <dir containing aruaru-ime-daemon>
set -u
ADDON="${1:?build dir with aruaru-ime.so}"
BIN="${2:?directory with aruaru-ime-daemon}"
T="$(mktemp -d)"
mkdir -p "$T/lib/fcitx5" "$T/share/fcitx5/addon" "$T/share/fcitx5/inputmethod" "$T/config/fcitx5"
cp "$ADDON/aruaru-ime.so" "$T/lib/fcitx5/"
cp "$(dirname "$0")/../fcitx5/aruaru-ime-addon.conf" "$T/share/fcitx5/addon/aruaru-ime.conf"
cp "$(dirname "$0")/../fcitx5/aruaru-ime-im.conf" "$T/share/fcitx5/inputmethod/aruaru-ime.conf"
cat > "$T/config/fcitx5/profile" <<EOF
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=aruaru-ime

[Groups/0/Items/0]
Name=aruaru-ime
Layout=

[GroupOrder]
0=Default
EOF
export XDG_CONFIG_HOME="$T/config" XDG_DATA_DIRS="$T/share:/usr/local/share:/usr/share" FCITX_ADDON_DIRS="$T/lib/fcitx5:/usr/lib/x86_64-linux-gnu/fcitx5"
unset DISPLAY WAYLAND_DISPLAY
mkdir -p "$T/run"; chmod 700 "$T/run"; export XDG_RUNTIME_DIR="$T/run"
export ARUARU_IME_PORT=4660
export PATH="$BIN:$PATH"
exec dbus-run-session -- bash -c '
  "'"$BIN"'/aruaru-ime-daemon" >"'"$T"'/daemon.log" 2>&1 &
  for i in $(seq 1 30); do curl -s http://127.0.0.1:4660/health >/dev/null && break; sleep 0.5; done
  fcitx5 --disable=wayland,waylandim,xcb,xim,classicui,kimpanel,notificationitem,notifications,virtualkeyboard,fcitx4frontend --verbose "*=5" >"'"$T"'/fcitx.log" 2>&1 &
  for i in $(seq 1 30); do dbus-send --session --print-reply --dest=org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus.NameHasOwner string:org.fcitx.Fcitx5 2>/dev/null | grep -q true && break; sleep 0.5; done
  sleep 1
  python3 - <<PY
import sys, time, dbus, dbus.mainloop.glib
from gi.repository import GLib
dbus.mainloop.glib.DBusGMainLoop(set_as_default=True)
bus = dbus.SessionBus()
im = dbus.Interface(bus.get_object("org.fcitx.Fcitx5", "/org/freedesktop/portal/inputmethod"), "org.fcitx.Fcitx.InputMethod1")
path, _ = im.CreateInputContext([("program", "aruaru-test")])
obj = bus.get_object("org.fcitx.Fcitx5", path)
ic = dbus.Interface(obj, "org.fcitx.Fcitx.InputContext1")
got = {"commit": [], "pre": ""}
ic.connect_to_signal("CommitString", lambda s: got["commit"].append(str(s)))
ic.connect_to_signal("UpdateFormattedPreedit", lambda t, c: got.__setitem__("pre", "".join(str(x[0]) for x in t)))
ic.SetCapability(dbus.UInt64((1 << 1) | (1 << 4)))
ic.FocusIn()
loop = GLib.MainLoop()
def pump(ms=300):
    ctx = loop.get_context(); end = time.time() + ms / 1000
    while time.time() < end:
        ctx.iteration(False); time.sleep(0.01)
pump(500)
t = 0
def key(sym):
    global t
    t += 1
    ok = ic.ProcessKeyEvent(dbus.UInt32(sym), dbus.UInt32(0), dbus.UInt32(0), False, dbus.UInt32(t))
    pump(250)
    return ok
for ch in "konnichiha":
    key(ord(ch))
print("preedit:", got["pre"])
ok_pre = ("こんにちは" in got["pre"]) or ("今日は" in got["pre"])
key(0xff0d); pump(500)
print("commit:", got["commit"])
ok_commit = any(("今日は" in c or "こんにちは" in c) for c in got["commit"])
sys.exit(0 if (ok_pre and ok_commit) else 1)
PY
  rc=$?
  echo "--- fcitx.log"; grep -iE "aruaru|error|fail" "'"$T"'/fcitx.log" | tail -20
  pkill -x fcitx5; pkill -f aruaru-ime-daemon
  exit $rc
'
