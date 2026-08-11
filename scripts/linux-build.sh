#!/usr/bin/env bash
#
# linux-build.sh — pull the latest Fractadyne source from GitHub and build it locally,
# so you can test a fresh commit on a Linux box WITHOUT triggering a GitHub release build.
#
# First run:   ./linux-build.sh --deps            # clone + install system deps + build
# Every after: ./linux-build.sh                   # fetch latest main + rebuild
# With checks: ./linux-build.sh --selftest        # build, then run --selftest under xvfb
# And launch:  ./linux-build.sh --run             # build, then start the app
#
# Options:
#   --deps            apt-get the system libraries the GUI needs (uses sudo; Debian/Ubuntu).
#   --selftest        after building, run the headless self-test under xvfb-run.
#   --run             after building, launch the app (needs a real display).
#   --branch NAME     build a branch/tag other than main.
#   --dir PATH        where the working copy lives (default: ~/fractadyne).
#   --release         build the optimized release profile (default; matches the shipped binary).
#   --debug           build the debug profile instead (faster compile, slower binary).
#   --clean           `cargo clean` first (use when a build is wedged; costs a full rebuild).
#   -h | --help       show this help.
#
# The repo is public, so no credentials are needed for a read-only clone/pull.

set -euo pipefail

REPO_URL="https://github.com/WindySnowOwl/fractadyne.git"
DIR="${HOME}/fractadyne"
BRANCH="main"
PROFILE="release"
DO_DEPS=0
DO_SELFTEST=0
DO_RUN=0
DO_CLEAN=0

die() { echo "error: $*" >&2; exit 1; }
say() { echo -e "\033[1;36m==>\033[0m $*"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --deps)     DO_DEPS=1 ;;
    --selftest) DO_SELFTEST=1 ;;
    --run)      DO_RUN=1 ;;
    --clean)    DO_CLEAN=1 ;;
    --release)  PROFILE="release" ;;
    --debug)    PROFILE="debug" ;;
    --branch)   shift; BRANCH="${1:?--branch needs a name}" ;;
    --dir)      shift; DIR="${1:?--dir needs a path}" ;;
    -h|--help)  sed -n '2,30p' "$0"; exit 0 ;;
    *)          die "unknown option: $1 (try --help)" ;;
  esac
  shift
done

# ---------------------------------------------------------------- system dependencies
# Mirrors the release/CI Linux jobs: winit/eframe need X11 + Wayland + xkbcommon; rfd (the
# native file dialogs this build now remembers directories for) needs GTK3.
if [ "$DO_DEPS" = "1" ]; then
  command -v apt-get >/dev/null 2>&1 || die "--deps assumes Debian/Ubuntu (apt-get not found); install the GTK3 + X11/Wayland dev packages manually"
  say "Installing system dependencies (sudo)…"
  sudo apt-get update
  sudo apt-get install -y --no-install-recommends \
    git curl build-essential pkg-config \
    libgtk-3-dev libxkbcommon-dev libwayland-dev \
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev
  # xvfb only needed for --selftest on a headless box; harmless to have.
  sudo apt-get install -y --no-install-recommends xvfb || true
fi

# ---------------------------------------------------------------- toolchain
if ! command -v cargo >/dev/null 2>&1; then
  # Pick up a rustup install that isn't on PATH yet in this shell.
  [ -f "${HOME}/.cargo/env" ] && . "${HOME}/.cargo/env"
fi
command -v cargo >/dev/null 2>&1 || die "cargo not found — install Rust: https://rustup.rs (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh)"
command -v git   >/dev/null 2>&1 || die "git not found — run with --deps, or apt-get install git"

# ---------------------------------------------------------------- fetch / update source
if [ -d "${DIR}/.git" ]; then
  say "Updating ${DIR} (branch ${BRANCH})…"
  git -C "${DIR}" fetch --prune origin
  # Hard-sync to origin so a local build never diverges from what you pushed. This DISCARDS
  # local edits in the working copy — it is a test checkout, not a dev tree.
  git -C "${DIR}" checkout "${BRANCH}"
  git -C "${DIR}" reset --hard "origin/${BRANCH}"
else
  say "Cloning ${REPO_URL} → ${DIR}…"
  git clone --branch "${BRANCH}" "${REPO_URL}" "${DIR}"
fi

cd "${DIR}"
COMMIT="$(git rev-parse --short HEAD)"
VERSION="$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
say "At ${COMMIT} — version ${VERSION}"

# ---------------------------------------------------------------- build
[ "$DO_CLEAN" = "1" ] && { say "cargo clean…"; cargo clean; }

if [ "$PROFILE" = "release" ]; then
  say "Building release binary (this can take a few minutes)…"
  cargo build --release --bin fractadyne
  BIN="${DIR}/target/release/fractadyne"
else
  say "Building debug binary…"
  cargo build --bin fractadyne
  BIN="${DIR}/target/debug/fractadyne"
fi
say "Built: ${BIN}"

# ---------------------------------------------------------------- optional self-test
# The self-test opens a GPU device, so it needs a display. On a headless box, xvfb-run gives
# it a virtual one. On a desktop with a real GPU you can also run "${BIN} --selftest" directly
# (a real GPU exercises the actual driver; the goldens are blessed on an RTX 3080, so exact
# matches are expected only there — small deltas elsewhere are FP32 variance, not failures).
if [ "$DO_SELFTEST" = "1" ]; then
  if command -v xvfb-run >/dev/null 2>&1; then
    say "Running --selftest under xvfb…"
    xvfb-run -a "${BIN}" --selftest || die "selftest reported failures (see output above)"
  else
    say "xvfb-run not found; running --selftest on the current display…"
    "${BIN}" --selftest || die "selftest reported failures (see output above)"
  fi
fi

# ---------------------------------------------------------------- optional launch
if [ "$DO_RUN" = "1" ]; then
  say "Launching…"
  exec "${BIN}"
fi

say "Done. Run it with:  ${BIN}"
