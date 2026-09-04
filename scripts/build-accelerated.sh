#!/usr/bin/env bash
# Build and package the OPTIONAL accelerated (MPFR) Fractadyne build for Linux x64.
#
# The standard Fractadyne binary computes its deep-zoom reference orbits with astro-float, a pure
# Rust library. This packages an alternative binary that uses MPFR/GMP instead, which is 2.5x to
# 6.4x faster at that one job. The two produce BYTE-IDENTICAL output; the difference is speed only.
# `scripts/build-accelerated.ps1` is the Windows half of this; keep the two in step.
#
# WHY A SEPARATE DOWNLOAD ON LINUX. Only ONE of the two Windows reasons survives here, and it is
# worth being precise about which:
#   * NOT the toolchain. On Windows, MPFR cannot build against the MSVC toolchain the standard
#     binary uses, so the accelerated build needs a whole separate GNU toolchain. On Linux the
#     standard build is already gcc/GNU, so this uses the same toolchain as its sibling job.
#   * The LICENSING, which does survive: rug, gmp-mpfr-sys, GMP and MPFR are all LGPL-3.0-or-later
#     while Fractadyne is MIT OR Apache-2.0, and those obligations attach to CONVEYING a binary.
#     They apply to this package and not to the standard one.
#
# LICENSING SHAPE (deliberate, not incidental — read before changing the build line):
#   * `--features gmp-mpfr-sys/use-system-libs` links GMP and MPFR as SHARED libraries, the
#     mechanism LGPLv3 section 4(d)(1) provides for, which keeps the obligations to NOTICES.
#     Linking them statically instead lands on 4(d)(0), which requires shipping the application in
#     relinkable form with EVERY release — a permanent tax on the release cadence. Do not
#     "simplify" this into a static build.
#   * ⭐**This package does NOT ship libgmp/libmpfr**, and that is the one deliberate difference
#     from the Windows package, which does ship them as DLLs. On Windows those libraries come from
#     MSYS2 and are not present on a user's machine; on Linux they are stock distro packages
#     (`libgmp10`, `libmpfr6`) present on essentially every desktop, and bundling system libraries
#     is not the platform convention. Not conveying them at all is also the cleaner licence
#     position: what is conveyed here is a work that dynamically LINKS them. The runtime
#     requirement is stated in the package's README-ACCELERATED.txt instead.
#   * The LGPL and GPL texts still ship: rug and gmp-mpfr-sys' own Rust code IS statically linked
#     into this binary and so IS conveyed. LGPLv3 section 4(b) requires the LGPL and the GPL it
#     refers to, hence both.
#
# Usage:  scripts/build-accelerated.sh [--tag vX.Y.Z] [--out DIR] [--skip-verify]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

tag=""
outdir="dist"
verify=1
while [ $# -gt 0 ]; do
    case "$1" in
        --tag) tag="${2:?--tag needs a value}"; shift 2 ;;
        --out) outdir="${2:?--out needs a value}"; shift 2 ;;
        --skip-verify) verify=0; shift ;;
        -h|--help) sed -n '2,34p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

step() { printf '\n=== %s\n' "$1"; }
fail() { echo "ERROR: $1" >&2; exit 1; }

# Default the tag to the workspace version, exactly as the PowerShell half does.
if [ -z "$tag" ]; then
    tag="v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
fi
[ "$tag" != "v" ] || fail "could not read the version from Cargo.toml"

# The libraries must be present to link against. Failing here with the install line is far kinder
# than a screen of linker errors.
for lib in gmp mpfr; do
    pkg-config --exists "$lib" 2>/dev/null && continue
    ldconfig -p 2>/dev/null | grep -q "lib${lib}\.so" && continue
    fail "lib${lib} not found. Install the development packages:
    Debian/Ubuntu:  sudo apt-get install -y libgmp-dev libmpfr-dev
    Fedora:         sudo dnf install gmp-devel mpfr-devel
    Arch:           sudo pacman -S gmp mpfr"
done

step "Building (rug + system GMP/MPFR)"
# `use-system-libs` is the LGPL section 4(d)(1) shape. See the header.
cargo build --release --bin fractadyne \
    --features fractadyne-core/rug \
    --features gmp-mpfr-sys/use-system-libs

dir="$outdir/fractadyne-$tag-linux-x64-accelerated"
step "Packaging into $dir"
rm -rf "$dir"
mkdir -p "$dir/tours" "$dir/scripts" "$dir/validation/golden" "$dir/benchmarks"
cp target/release/fractadyne "$dir/"
cp README.md CHANGELOG.md TOURS.md DIAGNOSTICS.md THIRD-PARTY-NOTICES.md \
   LICENSE-APACHE LICENSE-MIT "$dir/"
cp tours/*.toml "$dir/tours/"
cp scripts/*.example.toml "$dir/scripts/"
cp scripts/deep-sample.fdn "$dir/scripts/"
# Validation data so `--selftest` / `--bench-matrix` work from the extracted install; `anchored()`
# walks up from the binary and finds this tree beside it. BLESSED-GPU.txt records which card
# produced the goldens — without it a tester's self-test falls back to the STRICT tolerance and
# reports expected cross-vendor differences as failures.
cp validation/golden/*.png "$dir/validation/golden/"
cp validation/golden/BLESSED-GPU.txt "$dir/validation/golden/" 2>/dev/null || true
cp validation/catalog.toml "$dir/validation/"
cp benchmarks/bench-matrix-baseline.json "$dir/benchmarks/"

# Licence texts for the LGPL code compiled INTO this binary. Section 4(b) requires the LGPL and
# the GPL it refers to, so both ship or the package is not conveyable — fail rather than omit.
found_lgpl=0
for cand in /usr/share/doc/libmpfr6/copyright /usr/share/doc/libgmp10/copyright \
            /usr/share/doc/mpfr/COPYING.LESSER /usr/share/licenses/mpfr/COPYING.LESSER; do
    [ -f "$cand" ] || continue
    cp "$cand" "$dir/LICENSE-LGPL-3.0.txt"; found_lgpl=1; break
done
[ "$found_lgpl" = 1 ] || fail "no LGPL text found on this system; refusing to package LGPL code without it"
for cand in /usr/share/common-licenses/GPL-3 /usr/share/doc/mpfr/COPYING \
            /usr/share/licenses/mpfr/COPYING; do
    [ -f "$cand" ] || continue
    cp "$cand" "$dir/LICENSE-GPL-3.0.txt"; break
done
[ -f "$dir/LICENSE-GPL-3.0.txt" ] || fail "no GPL-3 text found; LGPLv3 4(b) requires it alongside the LGPL"

gmp_ver="$(pkg-config --modversion gmp 2>/dev/null || echo unknown)"
mpfr_ver="$(pkg-config --modversion mpfr 2>/dev/null || echo unknown)"

cat > "$dir/README-ACCELERATED.txt" <<EOF
Fractadyne $tag - ACCELERATED build (Linux x64)

This is the same program as the standard Linux download, with one difference: deep-zoom
reference orbits are computed with MPFR/GMP instead of astro-float, which is 2.5x to 6.4x
faster at that step. Output is BYTE-IDENTICAL to the standard build - the difference is
speed only. Settings, saved sessions and locations are shared, so you can move between the
two builds freely. Confirm which one you are running under Help -> About.

RUNTIME REQUIREMENT
  This build links GMP and MPFR dynamically and does NOT bundle them. They are stock
  packages present on essentially every Linux desktop; if the binary reports a missing
  library, install them:
    Debian/Ubuntu:  sudo apt-get install -y libgmp10 libmpfr6
    Fedora:         sudo dnf install gmp mpfr
    Arch:           sudo pacman -S gmp mpfr
  Built against GMP $gmp_ver and MPFR $mpfr_ver.

LICENSING
  Fractadyne's own code: MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE).
  This build additionally contains rug and gmp-mpfr-sys, which are LGPL-3.0-or-later, and
  links GNU GMP (https://gmplib.org/) and GNU MPFR (https://www.mpfr.org/), also
  LGPL-3.0-or-later. Copies of the GNU Lesser General Public License and of the GNU General
  Public License it refers to are included as LICENSE-LGPL-3.0.txt and LICENSE-GPL-3.0.txt.
  GMP and MPFR are linked as shared libraries, so you may substitute your own builds of
  either. Source for those libraries is available from the project sites above.
EOF

step "Archiving"
( cd "$outdir" && tar czf "fractadyne-$tag-linux-x64-accelerated.tar.gz" \
    "fractadyne-$tag-linux-x64-accelerated" )
( cd "$outdir" && sha256sum "fractadyne-$tag-linux-x64-accelerated.tar.gz" \
    > "fractadyne-$tag-linux-x64-accelerated.tar.gz.sha256" )

if [ "$verify" = 1 ]; then
    # Verify the PACKAGED binary, not the one in the build tree: the point is to catch a package
    # that cannot resolve its libraries on a normal system. `--version` needs no GPU or display.
    step "Verifying the packaged binary"
    ver_out="$("$dir/fractadyne" --version 2>&1)" || fail "packaged binary failed to run:
$ver_out"
    echo "$ver_out" | grep -q "fractadyne" || fail "unexpected --version output: $ver_out"
    # It must actually BE the accelerated build; a silent fall back to astro-float would ship a
    # package that is identical to the standard one and 6x slower than it claims.
    banner="$("$dir/fractadyne" --version 2>&1 | grep -i "backends compiled in" || true)"
    case "$banner" in
        *rug*) : ;;
        *) fail "packaged binary does not report the rug backend (got: ${banner:-<no banner>})" ;;
    esac
    echo "$ver_out"
fi

step "Done"
ls -la "$outdir"/*.tar.gz "$outdir"/*.sha256
