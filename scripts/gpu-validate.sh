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
# LOCAL BY DESIGN: the battery runs under ${TMPDIR:-/tmp}/fractadyne-validate/ and the finished
# bundle is copied to --out at the end. Every step logs into the bundle's config folder and the
# app writes frames.bin on the UI thread every frame, so a bundle built ON a share measures the
# share (the RX 6800 XT's beta.113 recordtest failed on exactly that). See gpu-validate.ps1.
#
# Steps that open a window are wrapped in xvfb-run when no DISPLAY is present, so this works over
# a plain SSH session. --gputest needs no display at all.
#
# Produces  validate-<label>-<timestamp>/  with summary.txt, system.txt, adapter.txt,
# 00-build-id.txt, 01-gputest.txt .. 06-uitest.txt, app.log and crash/ - plus a .tar.gz beside it.
# Exits 0 when every step did what gpu-validate-expected.json declares for this card, 2 when one
# did not (named in summary.txt), 3 when nothing could be judged - as gpu-validate.ps1.

set -uo pipefail   # NOT -e: a failing step is data; the battery must continue.

LABEL=""
QUICK=0
BACKEND=""
OUT=""
BIN=""
BUILD_ID=""
HERE="$(cd "$(dirname "$0")" && pwd)"

die() { echo "error: $*" >&2; exit 1; }
say() { echo -e "\033[1;36m==>\033[0m $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --label)   shift; LABEL="${1:?--label needs a name}" ;;
    --quick)   QUICK=1 ;;
    --backend) shift; BACKEND="${1:?--backend needs vulkan|gl}" ;;
    --out)     shift; OUT="${1:?--out needs a path}" ;;
    --bin)     shift; BIN="${1:?--bin needs a path}" ;;
    --build-id) shift; BUILD_ID="${1:?--build-id needs a path to BUILD-ID.txt}" ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
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
  # The team share when FRACTADYNE_SHARE names a mounted directory; else the home directory.
  if [ -n "${FRACTADYNE_SHARE:-}" ] && [ -d "$FRACTADYNE_SHARE" ]; then OUT="$FRACTADYNE_SHARE"; else OUT="$HOME"; fi
fi
STAMP="$(date +%Y%m%d-%H%M%S)"
NAME="validate-$LABEL-$STAMP"
# Built locally, delivered at the end (LOCAL BY DESIGN, above).
WORK="${TMPDIR:-/tmp}/fractadyne-validate"
DIR="$WORK/$NAME"
FINAL="$OUT/$NAME"
CFG="$DIR/config"
mkdir -p "$CFG" "$DIR/crash"
WORK_NOTE="this machine's disk"
case "$(stat -f -c %T "$DIR" 2>/dev/null)" in
  nfs*|cifs|smb*|fuse.sshfs|9p) WORK_NOTE="WARNING: this is a network filesystem - timings will measure it" ;;
esac

say "Fractadyne hardware validation"
echo "  binary : $BIN"
echo "  runs in: $DIR ($WORK_NOTE)"
echo "  bundle : $FINAL (copied there when the battery is done)"
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
APP_VERSION="$("$BIN" --version 2>/dev/null | grep '^fractadyne ' | tail -1)"
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
  echo "App     : $APP_VERSION"
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
  touch "$DIR/.stepstamp"
  "$@" > "$DIR/$file" 2>&1
  code=$?
  # The step's own frame record, harvested per step: every launch truncates frames.bin and
  # rotates frames.jsonl, so collecting once at the end would keep only the last step's. Only a
  # file written during THIS step (newer than the stamp) - see gpu-validate.ps1.
  mkdir -p "$DIR/frames"
  for f in frames.bin frames.jsonl; do
    if [ -f "$CFG/logs/$f" ] && [ "$CFG/logs/$f" -nt "$DIR/.stepstamp" ]; then
      cp "$CFG/logs/$f" "$DIR/frames/${file%.*}.$f"
    fi
  done
  t1=$(date +%s)
  local secs=$((t1 - t0))
  if [ "$code" -eq 0 ]; then echo -e "   \033[1;32mexit $code in ${secs}s\033[0m"
  else echo -e "   \033[1;31mexit $code in ${secs}s\033[0m"; fi
  SUMMARY_ROWS="${SUMMARY_ROWS}$(printf '%-14s %6s %8s  %s\n' "$name" "$code" "${secs}s" "$file")"$'\n'
  printf '%s\t%s\t%s\n' "$name" "$code" "$file" >> "$DIR/.steps.tsv"   # what the verdict judges
}

# --- step 00: WHICH binary is this? ----------------------------------------------------------------
# The mirror of gpu-validate.ps1 step 00 (see there for the why). 0 = the share published this binary,
# or it is a build of the published source (g<commit>-archive - the tarball route this box uses);
# 1 = BUILD-ID.txt found and this binary is not in it; 2 = no BUILD-ID.txt, cannot verify - NOT a pass.
build_id_step() {
  local t0 t1 ver tag="" id="" c commit code=2 hit="" out
  local re_tag='^fractadyne ([^ ]+) ' re_arch='\(build [0-9]+, g([0-9a-f]{7,40})-archive\)$'
  t0=$(date +%s)
  ver="${APP_VERSION#fractadyne }"
  [[ "$APP_VERSION" =~ $re_tag ]] && tag="v${BASH_REMATCH[1]}"
  local cands=()
  [ -n "$BUILD_ID" ] && cands+=("$BUILD_ID")
  cands+=("$HERE/BUILD-ID.txt" "$HERE/../BUILD-ID.txt" "$HERE/../../BUILD-ID.txt")
  [ -n "$tag" ] && cands+=("$OUT/builds/$tag/BUILD-ID.txt")
  [ -n "$tag" ] && [ -n "${FRACTADYNE_SHARE:-}" ] && cands+=("$FRACTADYNE_SHARE/builds/$tag/BUILD-ID.txt")
  for c in "${cands[@]}"; do [ -f "$c" ] && { id="$c"; break; }; done
  out="binary  : $BIN"$'\n'"version : $APP_VERSION"$'\n'
  if [ -z "$id" ]; then
    out+="verdict : CANNOT VERIFY - no BUILD-ID.txt found. Searched:"$'\n'
    for c in "${cands[@]}"; do out+="            $c"$'\n'; done
  else
    out+="against : $id"$'\n'
    # BUILD-ID.txt is written on Windows (CRLF): strip the CR before comparing.
    commit="$(tr -d '\r' < "$id" | sed -n 's/^commit: //p' | head -1)"
    hit="$(tr -d '\r' < "$id" | grep '^exe: ' | awk -F ' \\| ' -v v="$ver" '$2 == v { sub(/^exe: /, "", $1); print $1; exit }')"
    if [ -n "$hit" ]; then
      code=0; out+="verdict : OK - this is the published $hit"$'\n'
    elif [ -n "$commit" ] && [[ "$ver" =~ $re_arch ]] && [[ "$commit" == "${BASH_REMATCH[1]}"* ]]; then
      code=0; out+="verdict : OK - built from the published source of commit $commit"$'\n'
    else
      code=1; out+="verdict : MISMATCH - this binary is not one the share published. Published:"$'\n'
      out+="$(tr -d '\r' < "$id" | grep -E '^(tag|commit|exe): ' | sed 's/^/            /')"$'\n'
    fi
  fi
  printf '%s' "$out" > "$DIR/00-build-id.txt"
  t1=$(date +%s)
  echo -e "\033[1;33m-> build-id\033[0m"
  if [ "$code" -eq 0 ]; then echo -e "   \033[1;32m$(grep '^verdict' "$DIR/00-build-id.txt" | sed 's/^verdict : //')\033[0m"
  else echo -e "   \033[1;31m$(grep '^verdict' "$DIR/00-build-id.txt" | sed 's/^verdict : //')\033[0m"; fi
  SUMMARY_ROWS="${SUMMARY_ROWS}$(printf '%-14s %6s %8s  %s\n' "build-id" "$code" "$((t1 - t0))s" "00-build-id.txt")"$'\n'
  printf '%s\t%s\t%s\n' "build-id" "$code" "00-build-id.txt" >> "$DIR/.steps.tsv"
}
build_id_step

# --gputest is headless: no display, no xvfb, works over bare SSH.
step "gputest" "01-gputest.txt" \
  "df32/floatexp primitives vs CPU oracles, every backend" \
  "$BIN" --gputest
# --out puts the report and this run's golden renders (current/*.png) in the bundle: this card's
# would-be golden set (as gpu-validate.ps1).
step "selftest" "02-selftest.txt" \
  "full suite + goldens: strict against this card's own set if it has one, else cross-GPU against the RTX 3080's" \
  $XVFB "$BIN" --selftest --out "$DIR/02-selftest/report.md"
step "live-res" "03-live-res.txt" \
  "settled-resolution invariant - the B6 core, never yet run on hardware truly lacking TIMESTAMP_QUERY" \
  $XVFB "$BIN" --selftest --selftest-filter live-res
step "bench-matrix" "04-bench-matrix.txt" \
  "perf + determinism vs THIS card's baseline; 2 = algorithmic drift, 3 = no baseline for this card" \
  $XVFB "$BIN" --bench-matrix
# The run's own would-be baseline (as gpu-validate.ps1): review it, then commit it as
# benchmarks/bench-matrix-baseline-<card slug>.json.
[ -f "$CFG/logs/bench-matrix-run.json" ] && cp "$CFG/logs/bench-matrix-run.json" "$DIR/04-bench-matrix-run.json"

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
# ~20 s, so it runs even under --quick - see gpu-validate.ps1.
step "recordtest" "07-recordtest.txt" \
  "the per-frame record: every frame recorded, survives an abort, watchdog + blind tripwire fire" \
  $XVFB "$BIN" --recordtest

# --- harvest the app's own evidence --------------------------------------------------------------
[ -f "$CFG/logs/fractadyne.log" ] && cp "$CFG/logs/fractadyne.log" "$DIR/app.log"
cp "$CFG"/logs/crash-*.txt "$DIR/crash/" 2>/dev/null || true
cp "$CFG"/logs/crash-*-frames.jsonl "$CFG"/logs/crash-view-*.fdn "$DIR/crash/" 2>/dev/null || true
if [ -f "$DIR/app.log" ]; then
  grep -E 'adapter:|capability:|TIMESTAMP_QUERY' "$DIR/app.log" | sort -u > "$DIR/adapter.txt" || true
fi

# --- summary --------------------------------------------------------------------------------------
{
  echo "Fractadyne hardware validation - $LABEL"
  echo "$STAMP"
  echo "ran in $DIR ($WORK_NOTE)"
  echo
  printf '%-14s %6s %8s  %s\n' "Step" "Exit" "Time" "File"
  printf '%s' "$SUMMARY_ROWS"
  cat <<'EOF'

How to read this
----------------
build-id     Read this line FIRST. 0 = the binary tested is the one the share published, or a build
             of its published source. 1 = it is not: every other result is about an unknown binary.
             2 = no BUILD-ID.txt was found, so nobody can say which build this is - pass
             --build-id <path> to the one in the share's builds/<tag>/ folder.
gputest      A failing two_sum/two_prod means this stack's shader compiler folds the error-free
             transforms, so every extended-precision path silently degrades to plain f32. Every
             stack tested so far fails here: all NVIDIA backends, and the RX 6800 XT on every
             backend under both Windows and Linux (Mesa). Exit 1 is expected; the file is the data.
selftest     A card with its OWN golden set (validation/golden/<card>/) and bench-matrix baseline
             is held to them exactly. Any other card is compared with the RTX 3080's goldens at a
             wider, measured tolerance, and its path signatures are reported rather than failed.
             So a FAILURE here is a real signal. Known, under investigation: the RX 6800 XT fails
             one check, which depends on the system - "a wider canvas contains the narrower one"
             under Windows, "floatexp vs df32" under Linux (Mesa RADV). Any other failure is new.
             02-selftest/current/ holds this run's golden renders: this card's would-be set.
live-res     Must pass everywhere. This is the invariant that a GPU without TIMESTAMP_QUERY still
             settles at native resolution instead of being stuck at ~1/3 forever. On Linux this
             is the FIRST time it runs on hardware that genuinely lacks the feature (GL, and
             several Mesa/RADV/ANV combinations) rather than on an NVIDIA card faking it via
             FRACTADYNE_NO_TIMESTAMPS=1.
bench-matrix Timings vary by card and mean nothing across machines. The path signatures are
             compared EXACTLY against this card's own baseline: exit 2 = drift. A card with no
             baseline of its own exits 3 (nothing could be judged). 04-bench-matrix-run.json is
             this run as a would-be baseline for this card.
livetest     Self-contained: live view vs an offline render on THIS machine, so its pass/fail is
             meaningful here. "drift" lines compare against an RTX 3080 baseline and can be
             ignored on other hardware; FAIL and STARVED lines cannot (STARVED: a render at 4x
             the iteration ask resolves what the screen left black, and the app did not say why).
             Exit 2 = drift, 3 = nothing judged.
uitest       Screenshots for eyeballing. The deep floatexp band is WARN-not-FAIL by design.
recordtest   Must pass everywhere: 0 pass, 2 RED, 3 VACUOUS (nothing exercised - not a pass). The
             per-frame record is what diagnoses the next crash. frames/ holds each step's record.
verdict      The battery's own judgement, below: each step's exit code (and, for a step known to
             fail, which checks failed) against what gpu-validate-expected.json declares for this
             card. Exit 0 = all as declared, 2 = something UNEXPECTED (named), 3 = not judged. A
             card under Linux has its own name (RADV adds the chip: "...-radv-navi21"), so it is
             judged by the defaults until its own entry is declared.

Send back the whole folder (or the .tar.gz beside it).
EOF
} > "$DIR/summary.txt"



# --- verdict: every step against gpu-validate-expected.json ----------------------------------------
# The mirror of gpu-validate.ps1's verdict (see there): each card's expected exits, and where a step is
# known to fail WHICH checks, are declared in gpu-validate-expected.json beside this script; anything
# else is named here and makes the battery exit 2. A missing or unreadable file, or no Python to read
# it, judges nothing and exits 3. Lines are indented, as there.
VERDICT_CODE=3
VERDICT_TEXT="  NOT JUDGED: no python3 to read gpu-validate-expected.json, so no exit code was checked.
  VERDICT: NOT JUDGED"
# The first interpreter that RUNS: a `python3` on PATH can be a stub that only prints how to install one.
PY=""
for c in python3 python; do
  if command -v "$c" >/dev/null 2>&1 && "$c" -c 'import json' >/dev/null 2>&1; then PY="$c"; break; fi
done
if [ -n "$PY" ]; then
  VERDICT_TEXT="$("$PY" - "$HERE/gpu-validate-expected.json" "$DIR" <<'PYJUDGE'
import json, re, sys
exp_path, bundle = sys.argv[1], sys.argv[2]
slug = ""
try:
    for l in open(bundle + "/adapter.txt", encoding="utf-8", errors="replace"):
        m = re.search(r"adapter:\s*([^\u00b7]+?)\s*\u00b7", l)
        if m:
            slug = re.sub(r"[^a-z0-9]+", "-", m.group(1).lower()).strip("-")
            break
except OSError:
    pass
lines, code = [], 0
try:
    exp = json.load(open(exp_path, encoding="utf-8-sig"))
except (OSError, ValueError):
    exp = None
if not exp:
    code = 3
    lines.append("  NOT JUDGED: %s is missing or unreadable, so no step's exit code was checked." % exp_path)
else:
    card = exp.get("adapters", {}).get(slug) if slug else None
    note = "" if card else " (not in the file: judged by its defaults)"
    lines.append("  card: %s%s" % (slug or "(no adapter line in the log)", note))
    for row in open(bundle + "/.steps.tsv", encoding="utf-8"):
        step, ex, f = row.rstrip("\n").split("\t")
        ex = int(ex)
        rule = (card or {}).get(step) or exp.get("default", {}).get(step)
        allowed = [int(a) for a in rule["exit"]] if rule else [0]
        problem = ""
        if ex not in allowed:
            problem = "exit %d, expected %s" % (ex, " or ".join(str(a) for a in allowed))
            if ex == 0:
                problem += " - it now PASSES: update gpu-validate-expected.json"
        elif ex != 0 and rule and rule.get("known_fails"):
            try:
                text = open(bundle + "/" + f, encoding="utf-8", errors="replace").read().splitlines()
            except OSError:
                text = []
            fails = [m.group(1).strip() for m in (re.match(r"^\s*\[FAIL\]\s+(.+)$", l) for l in text) if m]
            new = [x for x in fails if not any(x.startswith(k) for k in rule["known_fails"])]
            if not fails:
                problem = "exit %d with no [FAIL] line to hold against the known failures" % ex
            elif new:
                problem = "NEW failure(s): " + "; ".join(new)
        if problem:
            code = 2
            lines.append("  UNEXPECTED   %-13s %s" % (step, problem))
        else:
            lines.append("  as expected  %-13s exit %d" % (step, ex))
word = {0: "PASS - every step did what gpu-validate-expected.json declares for this card",
        2: "FAIL - a step did not do what gpu-validate-expected.json declares (UNEXPECTED above)"}.get(code, "NOT JUDGED")
print("\n".join(lines + ["  VERDICT: " + word]))
sys.exit(code)
PYJUDGE
)"
  VERDICT_CODE=$?
fi
{
  echo
  echo "Verdict (gpu-validate-expected.json)"
  echo "------------------------------------"
  echo "$VERDICT_TEXT"
} >> "$DIR/summary.txt"
rm -f "$DIR/.steps.tsv"

tar -czf "$DIR.tar.gz" -C "$(dirname "$DIR")" "$(basename "$DIR")" 2>/dev/null || \
  echo "  (tar failed; send the folder itself)"

# --- deliver: copy the finished bundle (and its tarball) to --out ----------------------------------
# The local copy is removed only once the copy has succeeded; otherwise it stays, and says where.
BUNDLE="$DIR"
if [ "$(mkdir -p "$OUT" 2>/dev/null; cd "$OUT" 2>/dev/null && pwd -P)" != "$(cd "$WORK" && pwd -P)" ]; then
  if cp -r "$DIR" "$OUT/" && { [ ! -f "$DIR.tar.gz" ] || cp "$DIR.tar.gz" "$OUT/"; }; then
    rm -rf "$DIR" "$DIR.tar.gz"
    BUNDLE="$FINAL"
  else
    echo "  (copy to $OUT failed - the bundle is still at $DIR)"
  fi
fi

echo
say "Summary"
printf '%-14s %6s %8s\n' "Step" "Exit" "Time"
printf '%s' "$SUMMARY_ROWS" | awk '{printf "%-14s %6s %8s\n", $1, $2, $3}'
echo "bundle : $BUNDLE"
[ -f "$BUNDLE.tar.gz" ] && echo "tar    : $BUNDLE.tar.gz"
echo
echo "$VERDICT_TEXT"
say "Read summary.txt first - it explains which failures are expected off the reference card."
exit "$VERDICT_CODE"
