#!/usr/bin/env bash
# Build DELAY-IMPORT libraries for libmpfr-6.dll and libgmp-10.dll, for the accelerated Windows
# build (see scripts/build-accelerated.ps1 and fractadyne_core::resolve_startup_backend).
#
# WHY THIS EXISTS AS A SCRIPT: the mingw/GNU toolchain has no `ld --delayload`, and its
# libdelayimp.a is an empty stub, so delay loading has to be done with `dlltool -y` delay-import
# libraries whose stubs resolve __delayLoadHelper2 from libmingwex (linked by default). The
# build-accelerated.ps1 orchestrator then SWAPS these in for the normal import libs for the
# duration of the build (deterministic: gmp-mpfr-sys's own `-lmpfr` resolves to the delay stub,
# with no linker search-order gamble). Doing the generation here in bash rather than in PowerShell
# sidesteps two traps a PowerShell reimplementation hit: Set-Content writes CRLF, and dlltool folds
# a trailing CR into every symbol name so the stubs stop matching `-lmpfr`; and a `.text` section
# symbol from nm becomes a `.`-leading name that is a .def syntax error. LF, identifiers only.
#
# Usage:  make-delay-libs.sh <output-dir>
# Writes  <output-dir>/libmpfr.dll.a  and  <output-dir>/libgmp.dll.a  (delay-import libraries).
set -euo pipefail

OUT="${1:?usage: make-delay-libs.sh <output-dir>}"
LIB="${MINGW_LIB:-/mingw64/lib}"
BIN="${MINGW_BIN:-/mingw64/bin}"
mkdir -p "$OUT"

for pair in libmpfr-6.dll:libmpfr.dll.a libgmp-10.dll:libgmp.dll.a; do
    dll="${pair%%:*}"
    imp="${pair#*:}"
    src="$LIB/$imp"
    [ -f "$src" ] || { echo "make-delay-libs: import library not found: $src" >&2; exit 2; }

    # Exported function names = the ' T ' symbols of the normal import library, real C identifiers
    # only (drop nm's section symbols like .text, which are illegal in a .def EXPORTS block).
    "$BIN/nm" "$src" | awk '/ T /{print $3}' | grep -E '^[A-Za-z_][A-Za-z0-9_]*$' | sort -u > "$OUT/$imp.syms"
    n="$(wc -l < "$OUT/$imp.syms")"
    [ "$n" -ge 100 ] || { echo "make-delay-libs: only $n exports in $src -- refusing a truncated delay lib" >&2; exit 3; }

    { printf 'LIBRARY %s\nEXPORTS\n' "$dll"; cat "$OUT/$imp.syms"; } > "$OUT/$imp.def"
    "$BIN/dlltool" -m i386:x86-64 -y "$OUT/$imp" -d "$OUT/$imp.def" -D "$dll"
    echo "  $imp: $n exports -> delay-import stub"
done
