#!/bin/sh
# fs-server-probe.sh - measure what FractalShark 0.543's client/server CLI actually buys, and
# whether its output is a picture.
#
#   tools/fs-server-probe.sh CLI_EXE SCENE_KFR ITERATIONS [ALGO] [REPS]
#
# WHY THIS EXISTS. 0.543 was published in answer to our timing writeup: a server pays the
# initialization cost once so per-frame cost becomes visible. Before wiring that into the bench
# lane we need two numbers from THIS machine, not from the release notes -- how much the server
# actually amortizes, and whether the frames it produces are real. The second matters more than
# the first: 0.532's headless CLI wrote a FLAT IMAGE and exited 0 for every GPU algorithm, which
# is how this kit once published "144x faster than Fraktaler-3" for a blank frame.
#
# Prints one line per measurement and a summary; writes PNGs beside the scene for inspection.
set -u
CLI=$1; SCENE=$2; ITERS=$3; ALGO=${4:-Gpu1x32PerturbedLAv2}; REPS=${5:-3}
[ -x "$CLI" ] || { echo "usage: fs-server-probe.sh CLI_EXE SCENE_KFR ITERATIONS [ALGO] [REPS]" >&2; exit 2; }

CX=$(grep -m1 '^Re:' "$SCENE" | sed 's/^Re: *//')
CY=$(grep -m1 '^Im:' "$SCENE" | sed 's/^Im: *//')
ZOOM=$(grep -m1 '^Zoom:' "$SCENE" | sed 's/^Zoom: *//')
NAME=$(basename "$SCENE" .kfr)
OUT=$(dirname "$SCENE")/fs-probe-$NAME
mkdir -p "$OUT"
W=1280; H=720

ms_now() { date +%s%N; }

render_args() {
  echo "--render-algorithm $ALGO --center-x $CX --center-y $CY --zoom $ZOOM \
        --iterations $ITERS --width $W --height $H --antialiasing 1 --quiet"
}

echo "scene   : $NAME  zoom=$ZOOM iters=$ITERS algo=$ALGO"

# ---- cold: one process per frame, initialization paid every time -------------------------
i=1
while [ "$i" -le "$REPS" ]; do
  s=$(ms_now)
  # shellcheck disable=SC2046
  "$CLI" $(render_args) --out "$OUT/cold-$i.png" >"$OUT/cold-$i.log" 2>&1
  e=$(ms_now)
  echo "cold rep $i: $(( (e - s) / 1000000 )) ms"
  echo $(( (e - s) / 1000000 )) >> "$OUT/cold.txt"
  i=$((i + 1))
done

# ---- server: initialization once, then one client call per frame --------------------------
EP="FractalSharkCli-probe-$$"
s=$(ms_now)
("$CLI" --server --endpoint "$EP" --width $W --height $H > "$OUT/server.log" 2>&1 &)
# Wait for the pipe rather than sleeping a guess: a fixed sleep either wastes time or races.
tries=0
until grep -q "listening" "$OUT/server.log" 2>/dev/null || [ "$tries" -ge 60 ]; do
  sleep 1; tries=$((tries + 1))
done
e=$(ms_now)
echo "server startup: $(( (e - s) / 1000000 )) ms"

i=1
while [ "$i" -le "$REPS" ]; do
  s=$(ms_now)
  # shellcheck disable=SC2046
  "$CLI" --connect --endpoint "$EP" $(render_args) --out "$OUT/warm-$i.png" >"$OUT/warm-$i.log" 2>&1
  e=$(ms_now)
  echo "warm rep $i: $(( (e - s) / 1000000 )) ms  (server reports: $(grep -h -o 'Frame time: .*' "$OUT/warm-$i.log" | tail -1))"
  echo $(( (e - s) / 1000000 )) >> "$OUT/warm.txt"
  i=$((i + 1))
done

# The shutdown is what FLUSHES pending PNG encoding -- the client returns before the image is on
# disk, so a lane that checks the file straight after a render finds nothing and scores a DNF.
"$CLI" --connect --endpoint "$EP" --shutdown >>"$OUT/server.log" 2>&1
sleep 2

med() { sort -n "$1" 2>/dev/null | awk '{v[NR]=$1} END{ if(!NR) print "-"; else if(NR%2) print v[(NR+1)/2]; else print (v[NR/2]+v[NR/2+1])/2 }'; }
echo
echo "median cold (process per frame): $(med "$OUT/cold.txt") ms"
echo "median warm (server amortized) : $(med "$OUT/warm.txt") ms"
echo
echo "structure check (a flat image that exits 0 is the expensive kind of wrong):"
python - "$OUT" <<'PY'
import glob, sys, os
import numpy as np
from PIL import Image
for f in sorted(glob.glob(os.path.join(sys.argv[1], "*.png"))):
    a = np.asarray(Image.open(f).convert("RGB")).reshape(-1, 3)
    cols, counts = np.unique(a, axis=0, return_counts=True)
    modal = counts.max() / counts.sum()
    ok = len(cols) > 16 and modal < 0.98
    print(f"  {os.path.basename(f):16s} colours={len(cols):6d} modal={modal:.3f}  {'REAL' if ok else 'BLANK'}")
PY
