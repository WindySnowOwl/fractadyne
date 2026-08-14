#!/usr/bin/env bash
#
# gpu-validate.sh - run the whole hardware-validation battery on one machine/GPU and leave a
# single bundle to send back. The Linux mirror of scripts/gpu-validate.ps1: same steps, same file
# names, same bundle layout, so results from a Windows box and a Linux box are directly
# comparable.
#
#   ./gpu-validate.sh --label rx6800xt-linux
#   ./gpu-validate.sh --label rtx3070-linux --quick        # skip the two long steps
#   ./gpu-validate.sh --label rx6800xt --backend vulkan    # pin one backend
#
# HERMETIC BY DESIGN: everything runs against a private config directory inside the bundle, so
# your own settings are untouched and every machine renders identically - the point of a
# cross-GPU comparison. (The F3 corpus check inherits the developer's live session and its
# baselines drifted into meaninglessness because of it; do not repeat that here.)
#
# Steps that open a window are wrapped in xvfb-run when no DISPLAY is present, so this works over
# a plain SSH session. --gputest needs no display at all.
#
# Produces  validate-<label>-<timestamp>/  with summary.txt, system.txt, adapter.txt,
# 01-gputest.txt .. 06-uitest.txt, app.log and crash/ - plus a .tar.gz beside it.

set -uo pipefail   # NOT -e: a failing step is data; the battery must continue.

LABEL=""
QUICK=0
BACKEND=""
OUT=""
BIN=""

die() { echo "error: $*" >&2; exit 1; }
say() { echo -e "\033[1;36m==>\033[0m $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --label)   shift; LABEL="${1:?--label needs a name}" ;;
    --quick)   QUICK=1 ;;
    --backend) shift; BACKEND="${1:?--backend needs vulkan|gl}" ;;
    --out)     shift; OUT="${1:?--out needs a path}" ;;
    --bin)     shift; BIN="${1:?--bin needs a path}" ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) die "unknown option: $1" ;;
  esac
  shift
done
[ -n "$LABEL" ] || die "--label is required (e.g. --label rx6800xt-linux)"

# --- locate the binary -------------------------------------------------------------------------
if [ -z "$BIN" ]; then
  here="$(cd "$(dirname "$0")" && pwd)"
  for c in "$here/fractadyne" "$here/../fractadyne" "$here/../target/release/fractadyne" \
           "$HOME/fractadyne/target/release/fractadyne"; do
    [ -x "$c" ] && { BIN="$c"; break; }
  done
fi
[ -n "$BIN" ] && [ -x "$BIN" ] || die "fractadyne binary not found (pass --bin PATH)"

# --- bundle location ---------------------------------------------------------------------------
if [ -z "$OUT" ]; then
  if [ -d /mnt/vger/Fractadyne ]; then OUT=/mnt/vger/Fractadyne; else OUT="$HOME"; fi
fi
STAMP="$(date +%Y%m%d-%H%M%S)"
DIR="$OUT/validate-$LABEL-$STAMP"
CFG="$DIR/config"
mkdir -p "$CFG" "$DIR/crash"

say "Fractadyne hardware validation"
echo "  binary : $BIN"
echo "  bundle : $DIR"
[ -n "$BACKEND" ] && echo "  backend: $BACKEND (pinned)"
echo

export FRACTADYNE_CONFIG_DIR="$CFG"
[ -n "$BACKEND" ] && export WGPU_BACKEND="$BACKEND"

# Steps that create a window need a display. Use xvfb-run only when there is not one already.
XVFB=""
if [ -z "${DISPLAY:-}" ] && command -v xvfb-run >/dev/null 2>&1; then
  XVFB="xvfb-run -a -s -screen 0 2560x1600x24"
fi

# --- system inventory --------------------------------------------------------------------------
{
  echo "Fractadyne validation bundle - $LABEL - $STAMP"
  echo
  echo "Kernel  : $(uname -srmo)"
  [ -r /etc/os-release ] && echo "Distro  : $(. /etc/os-release; echo "$PRETTY_NAME")"
  echo "CPU     : $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//')"
  echo "Cores   : $(nproc)"
  echo "RAM     : $(awk '/MemTotal/ {printf "%.1f GB", $2/1048576}' /proc/meminfo)"
  echo "GPU     :"
  lspci -nn 2>/dev/null | grep -Ei 'vga|3d|display' | sed 's/^/          /'
  if command -v glxinfo >/dev/null 2>&1; then
    echo "GL      : $(glxinfo -B 2>/dev/null | grep -E 'OpenGL renderer|OpenGL version' | tr '\n' ' ')"
  fi
  if command -v vulkaninfo >/dev/null 2>&1; then
    echo "Vulkan  : $(vulkaninfo --summary 2>/dev/null | grep -E 'deviceName|driverInfo' | head -4 | tr '\n' ' ')"
  fi
  echo "App     : $("$BIN" --version 2>/dev/null | tail -1)"
  [ -n "$BACKEND" ] && echo "Backend : pinned to $BACKEND"
} > "$DIR/system.txt" 2>&1

# --- the battery -------------------------------------------------------------------------------
SUMMARY_ROWS=""
step() {
  local name="$1" file="$2" why="$3"; shift 3
  echo -e "\033[1;33m-> $name\033[0m"
  [ -n "$why" ] && echo -e "   \033[2m$why\033[0m"
  local t0 t1 code
  t0=$(date +%s)
  "$@" > "$DIR/$file" 2>&1
  code=$?
  t1=$(date +%s)
  local secs=$((t1 - t0))
  if [ "$code" -eq 0 ]; then echo -e "   \033[1;32mexit $code in ${secs}s\033[0m"
  else echo -e "   \033[1;31mexit $code in ${secs}s\033[0m"; fi
  SUMMARY_ROWS="${SUMMARY_ROWS}$(printf '%-14s %6s %8s  %s\n' "$name" "$code" "${secs}s" "$file")"$'\n'
}

# --gputest is headless: no display, no xvfb, works over bare SSH.
step "gputest" "01-gputest.txt" \
  "df32/floatexp primitives vs CPU oracles, every backend" \
  "$BIN" --gputest
step "selftest" "02-selftest.txt" \
  "full suite + 17 goldens (goldens blessed on an RTX 3080; deltas elsewhere are expected)" \
  $XVFB "$BIN" --selftest
step "live-res" "03-live-res.txt" \
  "settled-resolution invariant - the B6 core, never yet run on hardware truly lacking TIMESTAMP_QUERY" \
  $XVFB "$BIN" --selftest --selftest-filter live-res
step "bench-matrix" "04-bench-matrix.txt" \
  "22-segment perf + determinism; exit 2 signals algorithmic drift, not merely slower" \
  $XVFB "$BIN" --bench-matrix

if [ "$QUICK" -eq 0 ]; then
  step "livetest" "05-livetest.txt" \
    "live view vs an offline render of the same view ON THIS MACHINE" \
    $XVFB "$BIN" --livetest tours/grand-tour.toml --size 480x270
  step "uitest" "06-uitest.txt" \
    "25-step UI + live-render walk with screenshots" \
    $XVFB "$BIN" --uitest "$DIR"
else
  echo -e "\033[2m-> skipping livetest + uitest (--quick)\033[0m"
fi

# --- harvest the app's own evidence --------------------------------------------------------------
[ -f "$CFG/logs/fractadyne.log" ] && cp "$CFG/logs/fractadyne.log" "$DIR/app.log"
cp "$CFG"/logs/crash-*.txt "$DIR/crash/" 2>/dev/null || true
if [ -f "$DIR/app.log" ]; then
  grep -E 'adapter:|capability:|TIMESTAMP_QUERY' "$DIR/app.log" | sort -u > "$DIR/adapter.txt" || true
fi

# --- summary --------------------------------------------------------------------------------------
{
  echo "Fractadyne hardware validation - $LABEL"
  echo "$STAMP"
  echo
  printf '%-14s %6s %8s  %s\n' "Step" "Exit" "Time" "File"
  printf '%s' "$SUMMARY_ROWS"
  cat <<'EOF'

How to read this
----------------
gputest      A failing two_sum/two_prod means this stack's shader compiler folds the error-free
             transforms, so every extended-precision path silently degrades to plain f32. Known:
             all NVIDIA backends fold them; AMD Vulkan/OpenGL do not; AMD DX12 fails differently
             (fma not fused).
selftest     The 17 goldens are compared EXACTLY and were blessed on an RTX 3080. Small deltas on
             other hardware are expected and are NOT automatically bugs - cross-vendor rounding
             legitimately differs. Judge by how many and how large (maxD/meanD per golden). The
             113 non-golden checks SHOULD pass everywhere; those failing is a real signal.
live-res     Must pass everywhere. This is the invariant that a GPU without TIMESTAMP_QUERY still
             settles at native resolution instead of being stuck at ~1/3 forever. On Linux this
             is the FIRST time it runs on hardware that genuinely lacks the feature (GL, and
             several Mesa/RADV/ANV combinations) rather than on an NVIDIA card faking it via
             FRACTADYNE_NO_TIMESTAMPS=1.
bench-matrix Timings vary by card and mean nothing across machines. Exit 2 = algorithmic drift.
livetest     Self-contained: live view vs an offline render on THIS machine, so its pass/fail is
             meaningful here. "drift" lines compare against an RTX 3080 baseline and can be
             ignored on other hardware; FAIL lines cannot.
uitest       Screenshots for eyeballing. The deep floatexp band is WARN-not-FAIL by design.

Send back the whole folder (or the .tar.gz beside it).
EOF
} > "$DIR/summary.txt"

tar -czf "$DIR.tar.gz" -C "$(dirname "$DIR")" "$(basename "$DIR")" 2>/dev/null || \
  echo "  (tar failed; send the folder itself)"

echo
say "Summary"
printf '%-14s %6s %8s\n' "Step" "Exit" "Time"
printf '%s' "$SUMMARY_ROWS" | awk '{printf "%-14s %6s %8s\n", $1, $2, $3}'
echo "bundle : $DIR"
[ -f "$DIR.tar.gz" ] && echo "tar    : $DIR.tar.gz"
echo
say "Read summary.txt first - it explains which failures are expected off the reference card."
