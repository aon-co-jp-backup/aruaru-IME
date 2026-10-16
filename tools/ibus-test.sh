#!/bin/bash
# IBus engine end-to-end test (Linux). Needs: ibus-daemon, python3-gi + gir1.2-ibus-1.0, dbus-launch, and the built binaries.
# Usage: tools/ibus-test.sh <dir containing aruaru-ime-ibus and aruaru-ime-daemon>
# Env (optional, for a non-system IBus): IBUS_ROOT=<extracted root>
set -u
BIN="${1:?directory with aruaru-ime-ibus and aruaru-ime-daemon}"
T="$(mktemp -d)"
if [ -n "${IBUS_ROOT:-}" ]; then
  export PATH="$IBUS_ROOT/usr/bin:$PATH"
  export LD_LIBRARY_PATH="$IBUS_ROOT/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"
  export GI_TYPELIB_PATH="$IBUS_ROOT/usr/lib/x86_64-linux-gnu/girepository-1.0:${GI_TYPELIB_PATH:-}"
  export XDG_DATA_DIRS="$IBUS_ROOT/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
  mkdir -p "$T/schemas"; cp "$IBUS_ROOT"/usr/share/glib-2.0/schemas/*.xml "$T/schemas/"; glib-compile-schemas "$T/schemas"
  export GSETTINGS_SCHEMA_DIR="$T/schemas"
fi
mkdir -p "$T/share/ibus/component"
cat > "$T/share/ibus/component/aruaru-ime.xml" <<XML
<component>
  <name>org.freedesktop.IBus.AruaruIME</name>
  <description>aruaru-IME</description>
  <exec>$BIN/aruaru-ime-ibus --ibus</exec>
  <version>0.2</version><author>石 塚 正 浩（MASAHIRO ISHIZUKA）</author><license>MIT</license><homepage>https://github.com/aon-co-jp/aruaru-IME</homepage><textdomain>aruaru-ime</textdomain>
  <engines><engine><name>aruaru-ime</name><language>ja</language><license>MIT</license><author>石 塚 正 浩（MASAHIRO ISHIZUKA）</author><icon></icon><layout>default</layout><longname>aruaru-IME</longname><description>AI IME</description><rank>80</rank></engine></engines>
</component>
XML
export XDG_DATA_DIRS="$T/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export IBUS_COMPONENT_PATH="$T/share/ibus/component${IBUS_ROOT:+:$IBUS_ROOT/usr/share/ibus/component}"
export ARUARU_IME_PORT=4660 ARUARU_IME_IBUS_DEBUG="$T/engine.log"
exec dbus-run-session -- bash -c '
  G_MESSAGES_DEBUG=all ibus-daemon --daemonize --replace --address "unix:path='"$T"'/ibus.sock" --panel disable --config disable --emoji-extension disable --cache none --verbose >"'"$T"'/ibus.log" 2>&1
  for i in $(seq 1 30); do [ -S "'"$T"'/ibus.sock" ] && break; sleep 0.5; done
  export IBUS_ADDRESS="unix:path='"$T"'/ibus.sock"
  echo "ibus address: $IBUS_ADDRESS"
  ibus engine aruaru-ime 2>&1 | head -3; sleep 2
  python3 - <<PY
import sys, time, gi
gi.require_version("IBus", "1.0")
from gi.repository import IBus, GLib
IBus.init()
bus = IBus.Bus()
ic = bus.create_input_context("aruaru-test")
ic.set_capabilities(IBus.Capabilite.PREEDIT_TEXT | IBus.Capabilite.LOOKUP_TABLE | IBus.Capabilite.FOCUS)
got = {"commit": [], "pre": "", "cands": []}
ic.connect("commit-text", lambda c, t: got["commit"].append(t.get_text()))
ic.connect("update-preedit-text", lambda c, t, *a: got.__setitem__("pre", t.get_text()))
ic.connect("update-lookup-table", lambda c, tbl, *a: got.__setitem__("cands", [tbl.get_candidate(i).get_text() for i in range(tbl.get_number_of_candidates())]))
print("engines:", [e.get_name() for e in bus.list_engines() if "aruaru" in e.get_name()])
ic.set_engine("aruaru-ime")
ic.focus_in()
loop = GLib.MainLoop()
def pump(ms=300):
    ctx = loop.get_context(); end = time.time() + ms / 1000
    while time.time() < end:
        ctx.iteration(False); time.sleep(0.01)
pump(1500)
print("engine after set:", ic.get_engine().get_name() if ic.get_engine() else None)
def key(ch):
    ok = ic.process_key_event(ord(ch), 0, 0); pump(250); return ok
for ch in "konnichiha":
    key(ch)
print("preedit:", got["pre"], "| cands:", got["cands"][:4])
ok_pre = "こんにちは" in got["pre"] or any("こんにちは" in c for c in got["cands"])
ic.process_key_event(0xff0d, 0, 0); pump(500)
print("commit:", got["commit"])
ok_commit = any(("今日は" in c or "こんにちは" in c) for c in got["commit"])
sys.exit(0 if (ok_pre and ok_commit) else 1)
PY
  rc=$?
  echo "--- ibus.log"; tail -15 "'"$T"'/ibus.log"; echo "--- engine.log"; tail -15 "'"$T"'/engine.log"
  ps -eo pid,args | grep -E "aruaru-ime-(ibus|daemon)" | grep -v grep
  ibus exit 2>/dev/null
  exit $rc
'
