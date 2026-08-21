#!/usr/bin/env bash
# bench-latest.sh - Linux counterpart of bench-latest.ps1: download the LATEST releases of the
# benchmarked renderers, run the benchmark sequentially, and produce the summary report.
#
#   ./bench-latest.sh [--apps-dir DIR] [--required-gb N] [--reps N] [--timeout S]
#                     [--scenes slug,slug|id,...] [--skip lane,lane] [--skip-download]
#                     [--fractadyne PATH] [--fraktaler3 PATH]
#
# Lanes on Linux, honestly:
#   fractadyne    automated. Newest GitHub release with a linux-x64 tarball (prereleases
#                 included - that is where the Linux builds live); sha256 verified when
#                 published. --fractadyne PATH benchmarks a local build instead.
#   fraktaler3    automated, but mathr publishes no Linux binary (download/latest has Windows
#                 exes + source only) - pass --fraktaler3 PATH to a binary you built from
#                 https://fraktaler.mathr.co.uk/ or the lane records NA-no-linux-binary.
#   imagina       Windows-only upstream: recorded NA-windows-only.
#   fractalshark  CUDA + GUI; upstream ships FractalShark-Linux-<v>.tar.gz. Fetched only when
#                 an NVIDIA GPU is present (nvidia-smi) and a display is available, then run
#                 operator-assisted exactly like the Windows kit: you transcribe the render
#                 time the app reports.
#
# Results land in results/<host>-<stamp>/: sysinfo.txt, results.csv, summary.md,
# apps-manifest.txt - the same schema the PowerShell kit writes, so reports from either OS
# read the same. Timing columns per the README: wall_s (process start to exit, automated
# lanes) and reported_s (the renderer's own figure) are never mixed.

set -u
KIT="$(cd "$(dirname "$0")" && pwd)"

APPS="$KIT/apps"
REQUIRED_GB=2
REPS=1
TIMEOUT_S=7200
SCENES_FILTER=""
SKIP=""
SKIP_DOWNLOAD=0
FRACTADYNE_EXE=""
FRAKTALER3_EXE=""

while [ $# -gt 0 ]; do
    case "$1" in
        --apps-dir) APPS="$2"; shift 2 ;;
        --required-gb) REQUIRED_GB="$2"; shift 2 ;;
        --reps) REPS="$2"; shift 2 ;;
        --timeout) TIMEOUT_S="$2"; shift 2 ;;
        --scenes) SCENES_FILTER="$2"; shift 2 ;;
        --skip) SKIP="$2"; shift 2 ;;
        --skip-download) SKIP_DOWNLOAD=1; shift ;;
        --fractadyne) FRACTADYNE_EXE="$2"; shift 2 ;;
        --fraktaler3) FRAKTALER3_EXE="$2"; shift 2 ;;
        -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
        *) echo "unknown option: $1 (see --help)"; exit 1 ;;
    esac
done

step() { echo "== $*"; }
skipped() { case ",$SKIP," in *",$1,"*) return 0 ;; *) return 1 ;; esac }

for tool in curl tar awk df; do
    command -v "$tool" >/dev/null || { echo "ERROR: $tool is required"; exit 1; }
done

mkdir -p "$APPS" || { echo "ERROR: cannot create $APPS"; exit 1; }
APPS="$(cd "$APPS" && pwd)"
MANIFEST="$APPS/apps-manifest.txt"

# ---- free-space check on the apps dir's filesystem ----
if [ "$SKIP_DOWNLOAD" -eq 0 ]; then
    avail_kb=$(df -Pk "$APPS" | awk 'NR==2 {print $4}')
    need_kb=$(awk -v g="$REQUIRED_GB" 'BEGIN {printf "%d", g * 1024 * 1024}')
    if [ "$avail_kb" -lt "$need_kb" ]; then
        awk -v a="$avail_kb" -v g="$REQUIRED_GB" 'BEGIN {
            printf "ERROR: %.1f GB free at the apps dir; %s GB required (downloads + extraction + results).\n", a/1048576, g }'
        echo "Free space, choose another --apps-dir, or lower --required-gb if you know better."
        exit 1
    fi
    awk -v a="$avail_kb" -v g="$REQUIRED_GB" 'BEGIN {
        printf "== Space: %.1f GB free (need %s GB) - ok\n", a/1048576, g }'
fi

# ---- download helpers ----
UA="fractadyne-bench-kit"

# Newest GitHub release (prereleases included) whose asset name matches the regex; prints
# "tag<TAB>url<TAB>sha_url". python3 when present, else a sed fallback over the raw JSON.
gh_asset() { # repo asset_regex
    local json
    json=$(curl -sfL --max-time 60 -H "User-Agent: $UA" \
        "https://api.github.com/repos/$1/releases?per_page=15") || return 1
    if command -v python3 >/dev/null; then
        printf '%s' "$json" | python3 -c '
import json, re, sys
pat = re.compile(sys.argv[1])
for r in json.load(sys.stdin):
    hit = next((a for a in r["assets"] if pat.search(a["name"])), None)
    if hit:
        sha = next((a["browser_download_url"] for a in r["assets"]
                    if a["name"] == hit["name"] + ".sha256"), "")
        print(r["tag_name"] + "\t" + hit["browser_download_url"] + "\t" + sha)
        break
' "$2"
    else
        # Crude but serviceable: first download URL matching the regex; its release tag is the
        # nearest tag_name above it. sha side-asset lookup is skipped on this path.
        printf '%s' "$json" | tr ',' '\n' | sed -n 's/.*"browser_download_url": *"\([^"]*\)".*/\1/p' |
            grep -E "$2" | head -1 | awk '{print "unknown\t" $0 "\t"}'
    fi
}

fetch() { # url dest
    echo "   fetching $1"
    curl -sfL --max-time 1800 -H "User-Agent: $UA" -o "$2" "$1" || return 1
    echo "   $(du -m "$2" | cut -f1) MB"
}

check_sha() { # file sha_url
    [ -n "$2" ] || return 0
    local expected actual
    expected=$(curl -sfL --max-time 60 -H "User-Agent: $UA" "$2" | awk '{print tolower($1)}')
    actual=$(sha256sum "$1" | awk '{print $1}')
    [ "$expected" = "$actual" ] || { echo "ERROR: sha256 mismatch for $1"; return 1; }
    echo "   sha256 verified"
}

note_app() { # name version source exe
    local hash="-"
    [ -n "$4" ] && [ -f "$4" ] && hash=$(sha256sum "$4" | awk '{print $1}')
    echo "$1: $2  source=$3  exe=$4  sha256=$hash" >>"$MANIFEST"
}

# ---- lane preconditions ----
HAS_NVIDIA=0; command -v nvidia-smi >/dev/null && nvidia-smi -L >/dev/null 2>&1 && HAS_NVIDIA=1
HAS_DISPLAY=0; [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] && HAS_DISPLAY=1
FRACTALSHARK_EXE=""

echo "Fractadyne bench-kit app manifest - $(date '+%Y-%m-%d %H:%M:%S')" >"$MANIFEST"

if [ "$SKIP_DOWNLOAD" -eq 1 ]; then
    step "Offline mode: using whatever $APPS already holds."
    [ -z "$FRACTADYNE_EXE" ] && FRACTADYNE_EXE=$(find "$APPS" -name fractadyne -type f 2>/dev/null | head -1)
    FRACTALSHARK_EXE=$(find "$APPS" -iname 'FractalShark*' -type f ! -name '*.tar.gz' 2>/dev/null | head -1)
else
    # Fractadyne
    if [ -n "$FRACTADYNE_EXE" ]; then
        step "Fractadyne: using local build $FRACTADYNE_EXE"
        note_app fractadyne local-build "$FRACTADYNE_EXE" "$FRACTADYNE_EXE"
    else
        step "Fractadyne: newest release with a linux-x64 tarball"
        line=$(gh_asset WindySnowOwl/fractadyne 'fractadyne-.*-linux-x64\.tar\.gz$')
        if [ -n "$line" ]; then
            tag=$(printf '%s' "$line" | cut -f1); url=$(printf '%s' "$line" | cut -f2); sha=$(printf '%s' "$line" | cut -f3)
            tarball="$APPS/$(basename "$url")"
            if fetch "$url" "$tarball" && check_sha "$tarball" "$sha"; then
                rm -rf "$APPS/fractadyne"; mkdir -p "$APPS/fractadyne"
                tar -xzf "$tarball" -C "$APPS/fractadyne" &&
                    FRACTADYNE_EXE=$(find "$APPS/fractadyne" -name fractadyne -type f | head -1)
                [ -n "$FRACTADYNE_EXE" ] && chmod +x "$FRACTADYNE_EXE"
            fi
        fi
        if [ -n "$FRACTADYNE_EXE" ]; then
            note_app fractadyne "$tag" "$url" "$FRACTADYNE_EXE"
        else
            echo "   FAILED - no release tarball reachable; lane records DNF unless --fractadyne is given."
            note_app fractadyne FETCH-FAILED - ""
        fi
    fi

    # Fraktaler-3: no Linux binary upstream - operator-built only.
    if [ -n "$FRAKTALER3_EXE" ]; then
        step "Fraktaler-3: using $FRAKTALER3_EXE"
        note_app fraktaler3 operator-built local "$FRAKTALER3_EXE"
    else
        step "Fraktaler-3: mathr publishes no Linux binary; pass --fraktaler3 PATH to a build of"
        echo "   https://fraktaler.mathr.co.uk/ (download/latest has the source). Lane: NA."
        note_app fraktaler3 NA-no-linux-binary https://fraktaler.mathr.co.uk/ ""
    fi

    # FractalShark: CUDA + GUI, big asset - fetch only when the lane can actually run.
    if ! skipped fractalshark && [ "$HAS_NVIDIA" -eq 1 ] && [ "$HAS_DISPLAY" -eq 1 ]; then
        step "FractalShark: newest Linux release"
        line=$(gh_asset mattsaccount364/FractalShark 'FractalShark-Linux-.*\.tar\.gz$')
        if [ -n "$line" ]; then
            tag=$(printf '%s' "$line" | cut -f1); url=$(printf '%s' "$line" | cut -f2)
            tarball="$APPS/$(basename "$url")"
            if fetch "$url" "$tarball"; then
                rm -rf "$APPS/fractalshark"; mkdir -p "$APPS/fractalshark"
                tar -xzf "$tarball" -C "$APPS/fractalshark" &&
                    FRACTALSHARK_EXE=$(find "$APPS/fractalshark" -iname 'FractalShark*' -type f \
                        -perm -u+x ! -name '*.tar.gz' | head -1)
            fi
        fi
        if [ -n "$FRACTALSHARK_EXE" ]; then
            note_app fractalshark "$tag" "$url" "$FRACTALSHARK_EXE"
        else
            echo "   FAILED - lane will record DNF-fetch."
            note_app fractalshark FETCH-FAILED - ""
        fi
    fi
fi

# ---- results folder + sysinfo ----
STAMP=$(date '+%Y%m%d-%H%M%S')
OUT="$KIT/results/$(hostname)-$STAMP"
mkdir -p "$OUT"
CSV="$OUT/results.csv"
echo "renderer,scene,rep,status,wall_s,reported_s,note" >"$CSV"
{
    echo "Fractadyne benchmark kit - system info"
    echo "Timestamp: $(date '+%Y-%m-%d %H:%M:%S') (local)"
    echo "Host: $(hostname)"
    echo "OS: $(uname -srmo)"
    model=$(awk -F': ' '/model name/ {print $2; exit}' /proc/cpuinfo 2>/dev/null)
    echo "CPU: ${model:-unknown} ($(nproc 2>/dev/null || echo '?')T)"
    awk '/MemTotal/ {printf "RAM: %.1f GB\n", $2/1048576}' /proc/meminfo 2>/dev/null
    command -v nvidia-smi >/dev/null && nvidia-smi -L 2>/dev/null | sed 's/^/GPU: /'
    command -v lspci >/dev/null && lspci 2>/dev/null | grep -iE 'vga|3d controller' | sed 's/^/GPU: /'
} | tee "$OUT/sysinfo.txt"
cp "$MANIFEST" "$OUT/apps-manifest.txt" 2>/dev/null || true

result() { # renderer scene rep status wall reported note
    echo "$1,$2,$3,$4,$5,$6,\"$7\"" >>"$CSV"
    printf '  %-13s %-18s rep%s  %-12s wall=%-8s reported=%s\n' "$1" "$2" "$3" "$4" "$5" "$6"
}

# ---- scene table (scenes.csv, optionally filtered by --scenes slug-or-id list) ----
SCENES=()
while IFS=, read -r id slug mag_log10 iterations normalize; do
    [ "$id" = "id" ] && continue
    if [ -n "$SCENES_FILTER" ]; then
        case ",$SCENES_FILTER," in
            *",$slug,"*|*",$id,"*) ;;
            *) continue ;;
        esac
    fi
    SCENES+=("$id,$slug,$mag_log10,$iterations,$normalize")
done <"$KIT/scenes.csv"
[ "${#SCENES[@]}" -gt 0 ] || { echo "ERROR: --scenes matched nothing in scenes.csv"; exit 1; }

step "Lanes: fractadyne=$([ -n "$FRACTADYNE_EXE" ] && echo yes || echo no) fraktaler3=$([ -n "$FRAKTALER3_EXE" ] && echo yes || echo no) imagina=NA-windows-only fractalshark=$([ -n "$FRACTALSHARK_EXE" ] && echo assisted || echo no)"
step "Reps: $REPS  Timeout per render: ${TIMEOUT_S}s"

# Wall-timed run helper: prints "status wall_s" and leaves stdout in $RUN_OUT.
RUN_OUT=""
timed_run() { # exe args...
    local t0 t1 rc
    t0=$(date +%s.%N)
    RUN_OUT=$(timeout "$TIMEOUT_S" "$@" 2>&1); rc=$?
    t1=$(date +%s.%N)
    local wall
    wall=$(awk -v a="$t0" -v b="$t1" 'BEGIN {printf "%.1f", b - a}')
    if [ $rc -eq 0 ]; then echo "ok $wall"
    elif [ $rc -eq 124 ]; then echo "DNF-timeout $wall"
    else echo "DNF-exit$rc $wall"; fi
}

kfr_field() { awk -F': *' -v k="$2" '$1 == k {print $2; exit}' "$1" | tr -d '\r'; }

# "40.8s" / "2m07s" / "1h02m" -> plain seconds, so the CSV compares numbers, not strings.
to_seconds() {
    awk -v t="$1" 'BEGIN {
        h = 0; m = 0; s = 0
        if (t ~ /h/) { split(t, a, "h"); h = a[1]; t = a[2] }
        if (t ~ /m/) { split(t, a, "m"); m = a[1]; t = a[2] }
        if (t ~ /s/) { sub(/s$/, "", t); s = t }
        if (h + m + s > 0) printf "%.1f", h * 3600 + m * 60 + s
    }'
}

# ---- lane: fractadyne (automated; hermetic config per scene) ----
if [ -n "$FRACTADYNE_EXE" ] && ! skipped fractadyne; then
    for rep in $(seq 1 "$REPS"); do
        for row in "${SCENES[@]}"; do
            IFS=, read -r id slug mag_log10 iterations normalize <<<"$row"
            kfr="$KIT/scenes/$slug.kfr"
            re=$(kfr_field "$kfr" Re); im=$(kfr_field "$kfr" Im)
            zl2=$(awk -v m="$mag_log10" 'BEGIN {printf "%.10f", m * log(10) / log(2)}')
            cfg="$OUT/fd-config"; rm -rf "$cfg"; mkdir -p "$cfg"
            args=(--render --out "$OUT/fd-$slug.png" --size 1920x1080
                  --center "$re" "$im" --zoom-log2 "$zl2" --iter "$iterations" --ss 1 --palette 0)
            [ "$normalize" = "1" ] && args+=(--normalize)
            sw=$(FRACTADYNE_CONFIG_DIR="$cfg" timed_run "$FRACTADYNE_EXE" "${args[@]}")
            status=${sw% *}; wall=${sw#* }
            raw=$(printf '%s' "$RUN_OUT" | sed -n 's/.*(in \([0-9hms. ]*\)).*/\1/p' | head -1)
            reported=""; [ -n "$raw" ] && reported=$(to_seconds "$raw")
            result fractadyne "$slug" "$rep" "$status" "$wall" "$reported" ""
        done
    done
fi

# ---- lane: fraktaler3 (automated when a binary was provided; wisdom generated once) ----
if [ -n "$FRAKTALER3_EXE" ] && ! skipped fraktaler3; then
    wisdom="$OUT/f3-wisdom.toml"
    if [ ! -f "$wisdom" ]; then
        # `-w path -W` writes the initial hardware config to OUR file, `-w path -B` benchmarks
        # number types (the real tuning). A bare `-W path` treats the path as an input file and
        # silently writes nothing (see run-all.ps1 for the full story).
        step "Fraktaler-3: generating + benchmarking tuning wisdom (once)..."
        sw=$(timed_run "$FRAKTALER3_EXE" -w "$wisdom" -W)
        echo "  wisdom init: ${sw% *} in ${sw#* }s"
        sw=$(timed_run "$FRAKTALER3_EXE" -w "$wisdom" -B)
        echo "  wisdom benchmark: ${sw% *} in ${sw#* }s"
    fi
    for rep in $(seq 1 "$REPS"); do
        for row in "${SCENES[@]}"; do
            IFS=, read -r id slug mag_log10 iterations normalize <<<"$row"
            sw=$(timed_run "$FRAKTALER3_EXE" -w "$wisdom" -b "$KIT/scenes/$slug.f3.toml")
            result fraktaler3 "$slug" "$rep" "${sw% *}" "${sw#* }" "" ""
        done
    done
else
    for row in "${SCENES[@]}"; do
        IFS=, read -r id slug rest <<<"$row"
        result fraktaler3 "$slug" 1 NA-no-linux-binary "" "" "no upstream Linux binary; pass --fraktaler3 PATH to a local build"
    done
fi

# ---- lane: imagina (Windows-only upstream) ----
for row in "${SCENES[@]}"; do
    IFS=, read -r id slug rest <<<"$row"
    result imagina "$slug" 1 NA-windows-only "" "" "no Linux build published"
done

# ---- lane: fractalshark (operator-assisted, exactly like the Windows kit) ----
if [ -n "$FRACTALSHARK_EXE" ]; then
    for rep in $(seq 1 "$REPS"); do
        for row in "${SCENES[@]}"; do
            IFS=, read -r id slug rest <<<"$row"
            echo ""
            echo "--- fractalshark / $slug (rep $rep) ---"
            echo "FractalShark: load the scene .kfr (right-click > load location), render at"
            echo "1920x1080 with the scene iteration cap, then transcribe the displayed time."
            "$FRACTALSHARK_EXE" "$KIT/scenes/$slug.kfr" >/dev/null 2>&1 &
            read -r -p 'Enter the render time the app reports IN SECONDS (or DNF <reason>): ' ans
            case "$ans" in
                [Dd][Nn][Ff]*) result fractalshark "$slug" "$rep" DNF-operator "" "" "${ans#* }" ;;
                *[!0-9.]*|"") result fractalshark "$slug" "$rep" DNF-operator "" "" "unparseable entry: $ans" ;;
                *) result fractalshark "$slug" "$rep" ok "" "$ans" operator-transcribed ;;
            esac
        done
    done
elif [ "$HAS_NVIDIA" -eq 0 ]; then
    for row in "${SCENES[@]}"; do
        IFS=, read -r id slug rest <<<"$row"
        result fractalshark "$slug" 1 NA-no-nvidia "" "" "CUDA renderer, no NVIDIA GPU present"
    done
fi

# ---- summary: fastest run per renderer x scene (same table the PowerShell kit writes) ----
SUMMARY="$OUT/summary.md"
{
    echo "# Benchmark summary - $(hostname) - $STAMP"
    echo ""
    echo 'Fastest run per renderer and scene. `wall_s` compares the automated lanes end-to-end;'
    echo '`reported_s` is each renderer'"'"'s own figure (see README for why they are never mixed).'
    echo ""
    echo "| Scene | fractadyne wall | fraktaler3 wall | fd reported | imagina reported | fractalshark reported |"
    echo "|---|---|---|---|---|---|"
    for row in "${SCENES[@]}"; do
        IFS=, read -r id slug rest <<<"$row"
        line="| $slug "
        for ren in fractadyne fraktaler3; do
            best=$(awk -F, -v r="$ren" -v s="$slug" \
                '$1==r && $2==s && $4=="ok" && $5!="" {if (min=="" || $5+0<min+0) min=$5} END {print min}' "$CSV")
            [ -z "$best" ] && best=$(awk -F, -v r="$ren" -v s="$slug" '$1==r && $2==s {print $4; exit}' "$CSV")
            line="$line| ${best:--} "
        done
        fdrep=$(awk -F, -v s="$slug" '$1=="fractadyne" && $2==s && $4=="ok" {if (min=="" || $6+0<min+0) min=$6} END {print min}' "$CSV")
        line="$line| ${fdrep:--} | NA-windows-only "
        fs=$(awk -F, -v s="$slug" '$1=="fractalshark" && $2==s && $4=="ok" {if (min=="" || $6+0<min+0) min=$6} END {print min}' "$CSV")
        [ -z "$fs" ] && fs=$(awk -F, -v s="$slug" '$1=="fractalshark" && $2==s {print $4; exit}' "$CSV")
        line="$line| ${fs:--} |"
        echo "$line"
    done
    echo ""
    echo "## Apps measured"
    echo ""
    sed -n '2,$p' "$OUT/apps-manifest.txt" 2>/dev/null | sed 's/^/- /'
    echo ""
    echo "Send this folder (or its zip) to feedback@fractadyne.org or attach it to a GitHub issue."
} >"$SUMMARY"
echo ""
cat "$SUMMARY"
step "Report: $SUMMARY"
