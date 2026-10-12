#!/bin/sh
# Cross-OS smoke test of the resident engine (no dictionary needed): start it in server mode, then check
# health, romaji -> hiragana, symbol candidates (hyphen / slash / plus), the shared web screen, and a second
# instance exiting quietly. Usage: tools/e2e-smoke.sh <path-to-aruaru-ime-daemon>
set -eu
D="${1:?daemon path}"
T="$(mktemp -d)"
export ARUARU_IME_SERVER=1 ARUARU_DATA_DIR="$T" ARUARU_IME_DICT="$T/none.bin"
"$D" >"$T/log.txt" 2>&1 &
PID=$!
trap 'kill $PID 2>/dev/null || true' EXIT
i=0
until curl -fs http://127.0.0.1:4660/health >/dev/null 2>&1; do
  i=$((i+1)); [ $i -gt 50 ] && { echo "engine did not start"; cat "$T/log.txt"; exit 1; }
  sleep 0.3
done
fail() { echo "FAIL: $1"; exit 1; }
H="$(curl -fs http://127.0.0.1:4660/health)"; echo "health: $H"
echo "$H" | grep -q "proto=1" || fail "health"
post() { curl -fs -X POST "http://127.0.0.1:4660$1" -d "$2"; }
post /session '{"sid":"s","op":"lang","lang":"ja"}' >/dev/null
R=""
for c in k o n n i c h i h a; do R="$(post /key "{\"sid\":\"s\",\"key\":{\"Char\":\"$c\"}}")"; done
echo "typed konnichiha: $R" | cut -c1-160
echo "$R" | grep -q 'こんにちは' || fail "romaji to hiragana"
post /session '{"sid":"s","op":"cancel"}' >/dev/null
R="$(post /key '{"sid":"s","key":{"Char":"-"},"before":""}')"
echo "$R" | grep -q '－' || fail "hyphen candidates"
post /session '{"sid":"s","op":"cancel"}' >/dev/null
R="$(post /key '{"sid":"s","key":{"Char":"/"},"before":""}')"
echo "$R" | grep -q '・' || fail "slash -> nakaten"
post /session '{"sid":"s","op":"cancel"}' >/dev/null
R="$(post /key '{"sid":"s","key":{"Char":"+"},"before":""}')"
echo "$R" | grep -q 'プラス' || fail "plus candidates"
curl -fs http://127.0.0.1:4660/ui | grep -q 'ダウンロードはこちら' || fail "web screen"
# a second instance must exit quietly (same version already running)
if command -v timeout >/dev/null 2>&1; then
  timeout 15 "$D" >/dev/null 2>&1 && echo "second instance exited 0" || fail "second instance"
fi
echo "SMOKE OK"
