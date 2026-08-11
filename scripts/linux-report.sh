#!/usr/bin/env bash
#
# linux-report.sh — gather a Fractadyne diagnostics bundle on the Linux test rig into a staging
# folder, ready to be pulled to the Windows dev box with scripts/pull-linux-reports.ps1.
#
# This side only GATHERS — it does not transfer. The Windows box pulls (scp) whenever needed, so
# you can re-pull without re-gathering, and nothing on the rig needs to reach Windows.
#
#   ./linux-report.sh "black screen at e87 after loading my dive script"
#   ./linux-report.sh "selftest deltas on the RX 7900" --selftest
#
# Each run writes  <out>/<YYYYmmdd-HHMMSS>/  containing:
#   note.txt        your description + when/where
#   system.txt      uname, os-release, CPU/mem, GPU + driver, rustc/cargo versions
#   gpu-kernel.txt  kernel GPU messages (journalctl -k / dmesg) — the Linux analogue of the
#                   Windows "Event 153" GPU-reset evidence
#   git.txt         the working copy's commit / branch / version / dirty state
#   crash/          any crash-*.txt from the app's config dir (device-loss / OOM field evidence)
#   config/         session.toml (the exact view) + other *.toml from the config dir
#   selftest.txt    (with --selftest) the pass/fail table, captured under xvfb
#
# Options:
#   --selftest         run `--selftest` (under xvfb if headless) and capture its output.
#   --uitest           run the scripted UI + live-render walk (screenshots + checks) into uitest/.
#   --out DIR          staging base directory. Default: /mnt/vger/Fractadyne/reports when the
#                      \\vger\share mount is present (dev box reads it directly, no scp), else
#                      ~/fractadyne-reports (then scp-pull).
#   --config DIR       app config dir to harvest (default: $FRACTADYNE_CONFIG_DIR, else
#                      ${XDG_CONFIG_HOME:-~/.config}/fractadyne).
#   --repo DIR         Fractadyne working copy, for git info + the built binary (default: ~/fractadyne).
#   --tar              also produce <out>/<timestamp>.tar.gz of the folder.
#   -h | --help        show this help.

set -euo pipefail

NOTE=""
# Default staging base is resolved AFTER arg parsing (see below) so an explicit --out wins. When
# the Windows share //vger/share is mounted at /mnt/vger, its Fractadyne folder is the natural
# home — reports land at \\vger\share\Fractadyne\reports\<ts>\ and the Windows box reads them
# directly, no scp. Off the share, fall back to a home-dir folder (then scp-pull).
SHARE_BASE="/mnt/vger/Fractadyne/reports"
OUT_BASE=""   # empty ⇒ auto-detect
CONFIG_DIR="${FRACTADYNE_CONFIG_DIR:-${XDG_CONFIG_HOME:-${HOME}/.config}/fractadyne}"
REPO_DIR="${HOME}/fractadyne"
DO_SELFTEST=0
DO_UITEST=0
DO_TAR=0

die() { echo "error: $*" >&2; exit 1; }
say() { echo -e "\033[1;36m==>\033[0m $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --selftest) DO_SELFTEST=1 ;;
    --uitest)   DO_UITEST=1 ;;
    --tar)      DO_TAR=1 ;;
    --out)      shift; OUT_BASE="${1:?--out needs a path}" ;;
    --config)   shift; CONFIG_DIR="${1:?--config needs a path}" ;;
    --repo)     shift; REPO_DIR="${1:?--repo needs a path}" ;;
    -h|--help)  sed -n '2,32p' "$0"; exit 0 ;;
    --*)        die "unknown option: $1 (try --help)" ;;
    *)          NOTE="${NOTE:+$NOTE }$1" ;;   # accumulate free-text description
  esac
  shift
done

# Resolve the default staging base if --out wasn't given: the mounted share when present (so the
# Windows box sees the report with no transfer), else a home-dir folder for the scp-pull path.
if [ -z "$OUT_BASE" ]; then
  if [ -d "/mnt/vger/Fractadyne" ]; then
    OUT_BASE="$SHARE_BASE"
  else
    OUT_BASE="${HOME}/fractadyne-reports"
  fi
fi

# Timestamped run folder. `date` is fine here (this runs on the rig, not in a resumable harness).
TS="$(date +%Y%m%d-%H%M%S)"
RUN="${OUT_BASE}/${TS}"
mkdir -p "${RUN}/crash" "${RUN}/config"
say "Gathering into ${RUN}"

# ---------------------------------------------------------------- note
{
  echo "Fractadyne diagnostics bundle"
  echo "when:  ${TS} ($(date))"
  echo "host:  $(hostname)"
  echo "user:  $(whoami)"
  echo
  echo "note:  ${NOTE:-<none given>}"
} > "${RUN}/note.txt"

# ---------------------------------------------------------------- system / hardware
{
  echo "### uname";           uname -a 2>/dev/null || true
  echo; echo "### os-release"; cat /etc/os-release 2>/dev/null || true
  echo; echo "### cpu";        lscpu 2>/dev/null | sed -n '1,20p' || true
  echo; echo "### memory";     free -h 2>/dev/null || true
  echo; echo "### nvidia-smi"; command -v nvidia-smi >/dev/null && nvidia-smi 2>&1 || echo "(no nvidia-smi)"
  echo; echo "### GL renderer"
  if command -v glxinfo >/dev/null; then glxinfo 2>/dev/null | grep -iE 'OpenGL (vendor|renderer|version)' || true
  else echo "(no glxinfo — apt-get install mesa-utils for it)"; fi
  echo; echo "### Vulkan"
  if command -v vulkaninfo >/dev/null; then vulkaninfo --summary 2>/dev/null | sed -n '1,40p' || true
  else echo "(no vulkaninfo)"; fi
  echo; echo "### toolchain"
  command -v rustc >/dev/null && rustc --version || echo "(no rustc)"
  command -v cargo >/dev/null && cargo --version || true
} > "${RUN}/system.txt" 2>&1

# ---------------------------------------------------------------- kernel GPU messages
# The Linux stand-in for the Windows nvlddmkm "Event 153": a driver reset / Xid error shows up
# here. Prefer the current boot's kernel log; fall back to dmesg (may need sudo for full output).
{
  echo "### journalctl -k -b (GPU-related lines)"
  if command -v journalctl >/dev/null; then
    journalctl -k -b 2>/dev/null | grep -iE 'nvrm|nvidia|amdgpu|radeon|i915|drm|gpu hang|reset|xid|timeout' || echo "(no matching kernel lines — or journal not readable without sudo)"
  else
    echo "(no journalctl)"
  fi
  echo; echo "### dmesg (GPU-related lines)"
  dmesg 2>/dev/null | grep -iE 'nvrm|nvidia|amdgpu|radeon|i915|drm|gpu hang|reset|xid|timeout' || echo "(dmesg empty or needs sudo: try 'sudo dmesg')"
} > "${RUN}/gpu-kernel.txt" 2>&1

# ---------------------------------------------------------------- git / build provenance
if [ -d "${REPO_DIR}/.git" ]; then
  {
    echo "### repo: ${REPO_DIR}"
    git -C "${REPO_DIR}" log -1 --format='commit %h  %ci%n%s' 2>/dev/null || true
    echo; echo "branch: $(git -C "${REPO_DIR}" rev-parse --abbrev-ref HEAD 2>/dev/null)"
    echo "version: $(grep -m1 '^version = ' "${REPO_DIR}/Cargo.toml" 2>/dev/null | sed -E 's/version = "(.*)"/\1/')"
    echo; echo "### working-tree status (should be clean for a test build)"
    git -C "${REPO_DIR}" status --short 2>/dev/null || true
  } > "${RUN}/git.txt" 2>&1
else
  echo "(no git repo at ${REPO_DIR} — pass --repo)" > "${RUN}/git.txt"
fi

# ---------------------------------------------------------------- app config: crashes + session
if [ -d "${CONFIG_DIR}" ]; then
  say "Config dir: ${CONFIG_DIR}"
  # Crash reports (diag::write_crash_report) — the primary field evidence for device-loss/OOM.
  if compgen -G "${CONFIG_DIR}/logs/crash-*.txt" >/dev/null; then
    cp -f "${CONFIG_DIR}"/logs/crash-*.txt "${RUN}/crash/" 2>/dev/null || true
  fi
  # Any plain logs alongside them.
  compgen -G "${CONFIG_DIR}/logs/*.log" >/dev/null && cp -f "${CONFIG_DIR}"/logs/*.log "${RUN}/crash/" 2>/dev/null || true
  # Session state (the exact view that misbehaved) + any other config TOML.
  compgen -G "${CONFIG_DIR}/*.toml" >/dev/null && cp -f "${CONFIG_DIR}"/*.toml "${RUN}/config/" 2>/dev/null || true
  CRASHN="$(find "${RUN}/crash" -type f 2>/dev/null | wc -l | tr -d ' ')"
  say "Copied ${CRASHN} crash/log file(s) and $(find "${RUN}/config" -type f | wc -l | tr -d ' ') config file(s)"
else
  echo "(config dir not found: ${CONFIG_DIR})" > "${RUN}/config/MISSING.txt"
  say "No config dir at ${CONFIG_DIR} (set FRACTADYNE_CONFIG_DIR or pass --config)"
fi

# ---------------------------------------------------------------- optional self-test
if [ "$DO_SELFTEST" = "1" ]; then
  BIN="${REPO_DIR}/target/release/fractadyne"
  [ -x "$BIN" ] || BIN="${REPO_DIR}/target/debug/fractadyne"
  if [ -x "$BIN" ]; then
    say "Running --selftest (output → selftest.txt)…"
    if command -v xvfb-run >/dev/null; then
      xvfb-run -a "$BIN" --selftest > "${RUN}/selftest.txt" 2>&1 || true
    else
      "$BIN" --selftest > "${RUN}/selftest.txt" 2>&1 || true
    fi
    tail -1 "${RUN}/selftest.txt" | sed 's/^/    /'
  else
    echo "(no built binary under ${REPO_DIR}/target — build first with linux-build.sh)" > "${RUN}/selftest.txt"
  fi
fi

# ---------------------------------------------------------------- optional UI walk
# Runs the scripted UI + live-render walk (screenshots every screen, checks each) and drops its
# review bundle inside this report. Needs a display, so wrap it in xvfb-run when headless.
if [ "$DO_UITEST" = "1" ]; then
  BIN="${REPO_DIR}/target/release/fractadyne"
  [ -x "$BIN" ] || BIN="${REPO_DIR}/target/debug/fractadyne"
  if [ -x "$BIN" ]; then
    say "Running --uitest (screenshots + checks → uitest/)…"
    if command -v xvfb-run >/dev/null; then
      # A generous virtual screen so the wide-window step isn't clamped by the display size.
      xvfb-run -a -s "-screen 0 2560x1600x24" "$BIN" --uitest "${RUN}/uitest" > "${RUN}/uitest.log" 2>&1 || true
    else
      "$BIN" --uitest "${RUN}/uitest" > "${RUN}/uitest.log" 2>&1 || true
    fi
    grep -m1 "complete:" "${RUN}/uitest.log" | sed 's/^/    /' || true
  else
    echo "(no built binary under ${REPO_DIR}/target — build first with linux-build.sh)" > "${RUN}/uitest.log"
  fi
fi

# ---------------------------------------------------------------- optional tarball
if [ "$DO_TAR" = "1" ]; then
  ( cd "${OUT_BASE}" && tar czf "${TS}.tar.gz" "${TS}" )
  say "Tarball: ${OUT_BASE}/${TS}.tar.gz"
fi

say "Done. Bundle ready at: ${RUN}"
echo
case "${RUN}" in
  /mnt/vger/*)
    # Landed on the mounted Windows share — the dev box already sees it, no transfer needed.
    win_sub="${RUN#/mnt/vger/}"                       # path under the share root
    win_sub="${win_sub//\//\\}"                       # forward slashes → backslashes
    echo "On the share — the Windows box can read it directly at:"
    echo "    \\\\vger\\share\\${win_sub}"
    ;;
  *)
    echo "Not on the share. Pull it to the Windows box with (run there):"
    echo "    scripts\\pull-linux-reports.ps1 -From ${USER:-user}@$(hostname) -Latest"
    ;;
esac
