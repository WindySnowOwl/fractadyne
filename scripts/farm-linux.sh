#!/usr/bin/env bash
# farm-linux.sh - build Fractadyne on a Linux test machine (PLUTO booted into Linux) from a git bundle
# on the share, and run render-farm work there that a Windows machine asks for.
#
#   bash farm-linux.sh check      # what is missing, and the exact commands to install it
#   bash farm-linux.sh build      # clone the commit published on the share, build it, record it
#   bash farm-linux.sh farmtest   # --farmtest on this machine: a whole farm on Linux, results to the share
#   bash farm-linux.sh discover   # --discover: which controllers answer on this network
#   bash farm-linux.sh watch      # leave running: serve the requests scripts/farm-pluto.ps1 -Linux files
#                                 #   (a render client, --farmtest, --discover, or a rebuild of the
#                                 #   published commit: farm-pluto.ps1 -Linux -LinuxBuild)
#
# WHY A BUNDLE, NOT A TARBALL. The farm admits only an exact version-and-commit match, and a build
# from a `git archive` tarball is stamped "<sha>-archive" (its tree cannot be checked), which no
# Windows build matches. A git bundle clones into a real checkout, so the build is stamped with the
# commit itself. The Windows side writes it: scripts/publish-share.ps1 -Bundle.
#
# WATCH is a small stand-in for the Windows field agent, which does not run under Linux: it polls
# <share>/field/linux/requests/ for a request, runs it from the build made here, writes
# <share>/field/linux/results/<id>/, and deletes the farm key it was handed. It runs only what a
# request may name (a render client, --farmtest, --discover), every field checked, each run with a
# throw-away config folder - never this machine's own Fractadyne settings. Start it from a terminal
# in the desktop session: a render needs a display, even with its window hidden.
#
# Nothing here installs system packages: `check` prints the commands for you to run.

set -u
# Where this machine mounts the share: FRACTADYNE_SHARE, set where the mount is (e.g. ~/.bashrc).
SHARE="${FRACTADYNE_SHARE:-/mnt/share/fractadyne}"
WORK="${FARM_LINUX_WORK:-$HOME/fractadyne-farm}"
HOST="$(hostname -s 2>/dev/null || hostname)"
LINUX_DIR="$SHARE/field/linux"
SRC="$WORK/src"
EXE="$SRC/target/release/fractadyne"

say() { printf '%s\n' "$*"; }
ok() { printf '  ok    %s\n' "$*"; }
bad() { printf '  FIX   %s\n' "$1"; [ -n "${2:-}" ] && printf '        -> %s\n' "$2"; PROBLEMS=$((PROBLEMS + 1)); }
utc() { date -u +%Y-%m-%dT%H:%M:%SZ; }

# The commit the share publishes: the newest builds/<tag>/BUILD-ID.txt that has a bundle beside it.
published() {
    local best="" f
    for f in "$SHARE"/builds/*/BUILD-ID.txt; do
        [ -f "$f" ] || continue
        [ -n "$(ls "$(dirname "$f")"/*.bundle 2>/dev/null)" ] || continue
        if [ -z "$best" ] || [ "$f" -nt "$best" ]; then best="$f"; fi
    done
    [ -n "$best" ] || return 1
    BUILD_DIR="$(dirname "$best")"
    COMMIT="$(sed -n 's/^commit: //p' "$best" | head -1 | tr -d '\r ')"
    BUNDLE="$(ls "$BUILD_DIR"/*.bundle | head -1)"
    [ -n "$COMMIT" ]
}

# Is a version line a clean build of $COMMIT? Its stamp is ", g<sha>)" - a "-dirty" or "-archive"
# stamp, or another commit, is not.
stamp_ok() {
    local s; s="$(printf '%s' "$1" | sed -n 's/.*, g\([0-9a-f]\{7,40\}\))$/\1/p')"
    [ -n "$s" ] && [ -n "${COMMIT:-}" ] && [ "${COMMIT#"$s"}" != "$COMMIT" ]
}

# What the build here is: "fractadyne <ver> (build N, g<sha>)", read with a throw-away config.
version_line() {
    [ -x "$EXE" ] || return 1
    local cfg; cfg="$(mktemp -d)"
    FRACTADYNE_CONFIG_DIR="$cfg" FRACTADYNE_NO_SOUND=1 "$EXE" --version 2>/dev/null | grep '^fractadyne ' | tail -1
    rm -rf "$cfg"
}

cmd_check() {
    PROBLEMS=0
    say "Fractadyne render farm on Linux ($HOST) - preconditions"
    if [ -d "$SHARE" ] && [ -w "$SHARE" ]; then ok "the share is mounted and writable at $SHARE"
    else bad "the share is not mounted (or not writable) at $SHARE" "mount it, or set FRACTADYNE_SHARE to where it is"; fi
    local missing_apt=""
    command -v git >/dev/null || missing_apt="$missing_apt git"
    command -v cc >/dev/null || missing_apt="$missing_apt build-essential"
    command -v pkg-config >/dev/null || missing_apt="$missing_apt pkg-config"
    command -v python3 >/dev/null || missing_apt="$missing_apt python3"
    # The libraries the app links (as the CI's Linux job installs them).
    local m pkg
    for m in "gtk+-3.0:libgtk-3-dev" "xkbcommon:libxkbcommon-dev" "wayland-client:libwayland-dev" "x11:libx11-dev" "xcursor:libxcursor-dev" "xrandr:libxrandr-dev" "xi:libxi-dev"; do
        pkg="${m#*:}"
        if command -v pkg-config >/dev/null && pkg-config --exists "${m%%:*}" 2>/dev/null; then :; else missing_apt="$missing_apt $pkg"; fi
    done
    # With `apt-get update` first: on a machine whose package lists are old, the install asks the
    # mirror for versions it has since replaced and fails with 404 (seen on the test machine).
    if [ -n "$missing_apt" ]; then bad "missing packages:$missing_apt" "sudo apt-get update && sudo apt-get install -y$missing_apt"
    else ok "git, a C toolchain, pkg-config, python3 and the -dev libraries are installed"; fi
    if command -v cargo >/dev/null; then ok "Rust: $(rustc --version 2>/dev/null)"
    else bad "Rust is not installed (or not on PATH)" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && . \"\$HOME/.cargo/env\""; fi
    if command -v vulkaninfo >/dev/null; then ok "Vulkan: $(vulkaninfo --summary 2>/dev/null | grep -m1 -E 'deviceName' | sed 's/^ *//')"
    else say "  -     vulkaninfo not installed (optional: sudo apt-get install -y vulkan-tools)"; fi
    if [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then ok "a display session ($([ -n "${WAYLAND_DISPLAY:-}" ] && echo Wayland || echo X11))"
    else bad "no display in this shell" "run this from a terminal in the desktop session: a render needs a display"; fi
    if published; then ok "the share publishes commit ${COMMIT:0:12} ($BUNDLE)"
    else bad "no published bundle on the share (builds/<tag>/*.bundle beside BUILD-ID.txt)" "on Windows: pwsh -File scripts\\publish-share.ps1 -SkipSource -Bundle"; fi
    local v; v="$(version_line)"
    if [ -n "$v" ]; then
        if stamp_ok "$v"; then ok "built here: $v (the published commit)"
        else bad "built here: $v - not the published commit" "bash $0 build"; fi
    else bad "nothing built here yet" "bash $0 build"; fi
    say ""
    if [ "$PROBLEMS" -gt 0 ]; then say "Not ready: $PROBLEMS thing(s) to fix."; return 1; fi
    say "Ready."
}

cmd_build() {
    published || { say "no published bundle on the share - on Windows: pwsh -File scripts\\publish-share.ps1 -SkipSource -Bundle"; return 1; }
    command -v cargo >/dev/null || { say "Rust is not installed - run: bash $0 check"; return 1; }
    mkdir -p "$WORK"
    say "Building commit ${COMMIT:0:12} from $BUNDLE"
    if [ ! -d "$SRC/.git" ]; then git init -q "$SRC" || return 1; fi
    # Fetch the bundle's HEAD and check out exactly the commit BUILD-ID.txt names: a real checkout,
    # so the build is stamped with that commit (and would say -dirty if anything here differed).
    git -C "$SRC" fetch -q "$BUNDLE" HEAD || { say "could not fetch from the bundle"; return 1; }
    local got; got="$(git -C "$SRC" rev-parse FETCH_HEAD)"
    [ "$got" = "$COMMIT" ] || { say "the bundle's HEAD is $got, but BUILD-ID.txt says $COMMIT - republish"; return 1; }
    git -C "$SRC" checkout -q --detach --force "$COMMIT" || return 1
    local t0; t0=$(date +%s)
    ( cd "$SRC" && cargo build --release -p fractadyne-app ) || { say "the build failed"; return 1; }
    local v; v="$(version_line)"
    say "built in $(( $(date +%s) - t0 )) s: $v"
    stamp_ok "$v" || { say "the build reports '$v', not the published commit (a -dirty or unknown stamp?)"; return 1; }
    mkdir -p "$LINUX_DIR"
    python3 - "$LINUX_DIR/built-$HOST.json" "$COMMIT" "$v" "$EXE" <<'EOF'
import json, sys, datetime
p, commit, version, exe = sys.argv[1:5]
json.dump({"host": p.rsplit("built-", 1)[1][:-5], "commit": commit, "version": version, "exe": exe,
           "built_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}, open(p, "w"), indent=1)
EOF
    say "recorded on the share: $LINUX_DIR/built-$HOST.json"
}

# Run one action into <share>/field/linux/results/<id>/: output.txt, the run's logs, status.json.
run_action() {
    local id="$1" action="$2" controller="${3:-}" key="${4:-}" share_mode="${5:-false}" discover="${6:-false}"
    local res="$LINUX_DIR/results/$id" local_dir="$WORK/runs/$id"
    mkdir -p "$res" "$local_dir/config"
    local out="$res/output.txt" code=0 t0; t0=$(date +%s)
    write_status() {
        python3 - "$res/status.json" "$id" "$1" "$2" "$3" "$(version_line)" <<'EOF'
import json, sys
p, rid, state, code, detail, version = sys.argv[1:7]
json.dump({"id": rid, "state": state, "exit": int(code) if code.lstrip("-").isdigit() else None,
           "detail": detail, "build": version}, open(p, "w"), indent=1)
EOF
    }
    write_status running "" "$action"
    export FRACTADYNE_CONFIG_DIR="$local_dir/config" FRACTADYNE_NO_SOUND=1
    case "$action" in
        build)
            # The commit the share publishes, built here: what a code change on the Windows side
            # needs before both ends can run one farm again. Nothing from the request is used.
            cmd_build >"$out" 2>&1; code=$? ;;
        discover)
            "$EXE" --discover >"$out" 2>&1; code=$? ;;
        farmtest)
            "$EXE" --farmtest "$local_dir/farmtest" >"$out" 2>&1; code=$? ;;
        client)
            local kf="$local_dir/farm-key.txt"
            ( umask 077; printf '%s\n' "$key" >"$kf" )
            if [ "$discover" = "true" ]; then "$EXE" --discover >"$res/discover.txt" 2>&1; fi
            local extra=()
            [ "$share_mode" = "true" ] && extra+=(--share-root "$SHARE")
            "$EXE" --render-client "$controller" --farm-key-file "$kf" --name "$HOST-linux" --one-job "${extra[@]}" >"$out" 2>&1; code=$?
            rm -f "$kf" ;;
    esac
    unset FRACTADYNE_CONFIG_DIR
    [ -d "$local_dir/config/logs" ] && cp -r "$local_dir/config/logs" "$res/logs" 2>/dev/null
    write_status "$([ "$code" -eq 0 ] && echo done || echo failed)" "$code" "$action ($(( $(date +%s) - t0 )) s)"
    say "  $id: $action exit $code -> $res"
}

cmd_once() { case "${1:-}" in discover|farmtest) run_action "manual-$(date +%Y%m%d-%H%M%S)" "$1" ;; esac; }

cmd_watch() {
    [ -x "$EXE" ] || { say "nothing built here yet - run: bash $0 build"; return 1; }
    [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] || { say "no display in this shell - start this from a terminal in the desktop session"; return 1; }
    mkdir -p "$LINUX_DIR/requests" "$LINUX_DIR/claimed" "$LINUX_DIR/results"
    # A request's run reads `version_line` afresh, so a rebuild takes effect at the next request.
    say "Watching $LINUX_DIR/requests for render-farm requests (build: $(version_line)). Ctrl+C to stop."
    while true; do
        python3 - "$LINUX_DIR/watch-$HOST.json" "$(version_line)" <<'EOF'
import json, sys, datetime
json.dump({"state": "watching", "build": sys.argv[2],
           "last_poll_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), "poll_seconds": 5},
          open(sys.argv[1], "w"), indent=1)
EOF
        local req
        for req in "$LINUX_DIR"/requests/*.json; do
            [ -f "$req" ] || continue
            local base; base="$(basename "$req" .json)"
            local claimed="$LINUX_DIR/claimed/$base.json"
            mv "$req" "$claimed" 2>/dev/null || continue
            # Every field checked before anything runs; the key is read here and removed from the
            # claimed copy at once.
            local fields
            fields="$(python3 - "$claimed" <<'EOF'
import json, re, sys
r = json.load(open(sys.argv[1]))
rid, action = str(r.get("id", "")), str(r.get("action", ""))
ok = re.fullmatch(r"[0-9]{8}-[0-9]{6}-[a-z0-9]{4}", rid) and action in ("client", "farmtest", "discover", "build")
ctl, key = str(r.get("controller", "")), str(r.get("farm_key", ""))
if action == "client":
    ok = ok and re.fullmatch(r"[A-Za-z0-9.-]{1,253}:[0-9]{1,5}", ctl) and re.fullmatch(r"fdn1-[a-z2-7-]{50,90}", key)
r["farm_key"] = "(removed)"
json.dump(r, open(sys.argv[1], "w"), indent=1)
print("\t".join([rid, action, ctl, key, str(bool(r.get("share", False))).lower(), str(bool(r.get("discover", False))).lower()]) if ok else "REFUSED")
EOF
)"
            if [ "$fields" = "REFUSED" ] || [ -z "$fields" ]; then say "  refused $base (a field failed its check)"; continue; fi
            local rid action ctl key sm dc
            IFS=$'\t' read -r rid action ctl key sm dc <<<"$fields"
            say "  $(utc) running $rid: $action ${ctl}"
            run_action "$rid" "$action" "$ctl" "$key" "$sm" "$dc"
            key=""
        done
        sleep 5
    done
}

case "${1:-check}" in
    check) cmd_check ;;
    build) cmd_build ;;
    farmtest|discover) [ -x "$EXE" ] || { say "nothing built here yet - run: bash $0 build"; exit 1; }; cmd_once "$1" ;;
    watch) cmd_watch ;;
    *) say "usage: bash $0 {check|build|farmtest|discover|watch}"; exit 2 ;;
esac
