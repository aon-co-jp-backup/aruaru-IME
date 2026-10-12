#!/bin/sh
# Android smoke test (runs inside the emulator job): install the APK, start it, check that the online screen
# loads (its text appears in the accessibility tree), then rotate and check again. Saves screenshots.
# Usage: tools/android-smoke.sh <apk> <name>
set -u
APK="$1"; NAME="$2"
adb install -r "$APK" || exit 1
adb shell am start -n jp.aon.aruaruime/.MainActivity || exit 1
ok=0
for i in $(seq 1 40); do
  sleep 3
  adb shell uiautomator dump /sdcard/ui.xml >/dev/null 2>&1
  adb pull /sdcard/ui.xml "android-test-$NAME-ui.xml" >/dev/null 2>&1
  # キーボードの有効化を案内するダイアログが出ていたら、閉じる
  if grep -q "キーボード / keyboard" "android-test-$NAME-ui.xml" 2>/dev/null; then adb shell input keyevent KEYCODE_BACK; sleep 2; continue; fi
  if grep -q "ダウンロードはこちら\|aruaru-IME" "android-test-$NAME-ui.xml" 2>/dev/null && grep -q "Download the installer\|ダウンロード" "android-test-$NAME-ui.xml" 2>/dev/null; then ok=1; break; fi
done
adb exec-out screencap -p > "android-test-$NAME-portrait.png"
# landscape
adb shell settings put system accelerometer_rotation 0
adb shell settings put system user_rotation 1
sleep 4
adb exec-out screencap -p > "android-test-$NAME-landscape.png"
adb shell uiautomator dump /sdcard/ui2.xml >/dev/null 2>&1
adb pull /sdcard/ui2.xml "android-test-$NAME-ui-landscape.xml" >/dev/null 2>&1
grep -q "ダウンロード" "android-test-$NAME-ui-landscape.xml" 2>/dev/null || ok=0
adb logcat -d | grep -i "chromium\|aruaru" | tail -30 > "android-test-$NAME-log.txt"
if [ $ok = 1 ]; then echo "ANDROID SMOKE OK ($NAME)"; exit 0; else echo "ANDROID SMOKE FAILED ($NAME)"; exit 1; fi
