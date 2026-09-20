#!/bin/sh
# Run the real autopilot from a .kfr location in a WIPED scratch config and capture the live window
# by PrintWindow (no input is sent anywhere), so what was ON SCREEN can be scored — see README.md.
#
#   usage: capdive.sh NAME EXE KFR [SEED_TOML] [TIMEOUT_S] [ITER]
#     NAME       output directory (created under the current directory, wiped first)
#     EXE        the fractadyne build to run (a COPY, not the one the user is running)
#     KFR        the start location (dive-2p800.kfr reproduces the blank-motion-frame regime)
#     SEED_TOML  session.toml to seed the scratch config with (default: session-seed.toml here —
#                the reporting user's settings: prefer_detail, min_motion_res 0.83, zoom rate 4x)
#     TIMEOUT_S  how long the dive runs (default 26); capture starts 4 s in and lasts 18 s
#     ITER       --autodive-iter (default 10000 = the fixed count of the report; 0 = auto-iter)
HERE="$(cd "$(dirname "$0")" && pwd)"
NAME=$1; EXE=$2; KFR=$3; SEED=${4:-$HERE/session-seed.toml}; T=${5:-26}; ITER=${6:-10000}
[ -n "$NAME" ] && [ -f "$EXE" ] && [ -f "$KFR" ] || { echo "usage: capdive.sh NAME EXE KFR [SEED_TOML] [TIMEOUT_S] [ITER]" >&2; exit 2; }
OUT="$PWD/$NAME"; rm -rf "$OUT"; mkdir -p "$OUT/cfg" "$OUT/frames"
cp "$SEED" "$OUT/cfg/session.toml"
FRACTADYNE_CONFIG_DIR="$OUT/cfg" FRACTADYNE_NO_SOUND=1 FRACTADYNE_TRACE=autopilot,tile,gpu \
  "$EXE" --import-kfr "$KFR" --show-timestamp --autodive 300 --autodive-iter "$ITER" \
  --autodive-home 0 --autodive-timeout "$T" > "$OUT/stdout.txt" 2> "$OUT/stderr.txt" &
sleep 4
PID=$(powershell -NoProfile -Command "(Get-Process $(basename "$EXE" .exe) -ErrorAction SilentlyContinue | Select-Object -First 1).Id" | tr -d '\r')
echo "$NAME pid=$PID"
powershell -NoProfile -ExecutionPolicy Bypass -File "$HERE/grab.ps1" -ProcId "$PID" -OutDir "$OUT/frames" -Seconds 18 -IntervalMs 100 >/dev/null 2>&1
echo "$NAME captured $(ls "$OUT/frames" | wc -l) frames"
wait
