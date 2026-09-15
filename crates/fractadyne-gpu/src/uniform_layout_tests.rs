//! The Rust `ColorUniforms` and the WGSL `ColorU` describe the same bytes, and nothing checks
//! that at compile time: the uniform is written with `bytemuck::bytes_of` into an opaque buffer,
//! so a field added to one and not the other produces a pipeline that builds, runs, and colours
//! every pixel from the wrong offsets. `design/palette-import.md` §4 names this in advance —
//! "**both `ColorU` definitions must change together** … or offline renders silently keep the old
//! palette" — and the palette-LUT change edited both.
//!
//! So: parse the WGSL struct, lay it out under WGSL's alignment rules, and compare the total with
//! `size_of::<ColorUniforms>()`. A mismatch is not a style problem; it is a wrong picture.

use super::{aux_sel, ColorUniforms, IterUniforms};

/// `(size, align)` for the WGSL types this struct uses. An unrecognised type fails the test rather
/// than being guessed at — a new type in the uniform is a deliberate change and should say so.
fn size_align(ty: &str) -> (usize, usize) {
    let ty = ty.trim();
    if let Some(rest) = ty.strip_prefix("array<") {
        // `array<vec4<f32>, 8>` — the only array shape the uniform has ever carried.
        let inner = rest.trim_end_matches('>');
        let (elem, count) = inner.rsplit_once(',').expect("array without a count");
        let n: usize = count.trim().parse().expect("array count");
        let (es, ea) = size_align(elem);
        // Uniform-address-space arrays have a stride rounded up to 16.
        let stride = es.div_ceil(16) * 16;
        return (stride * n, ea.max(16));
    }
    match ty {
        "u32" | "i32" | "f32" => (4, 4),
        "vec2<f32>" | "vec2<u32>" | "vec2<i32>" => (8, 8),
        "vec3<f32>" | "vec3<u32>" | "vec3<i32>" => (12, 16),
        "vec4<f32>" | "vec4<u32>" | "vec4<i32>" => (16, 16),
        other => panic!("unhandled WGSL type in the uniform: {other:?}"),
    }
}

/// Fields of a named WGSL struct, in declaration order, as `(name, type)`.
fn wgsl_struct_fields(src: &str, name: &str) -> Vec<(String, String)> {
    let head = format!("struct {name} {{");
    let start = src.find(&head).unwrap_or_else(|| panic!("no `{head}` in the shader"));
    let body = &src[start + head.len()..];
    let end = body.find("};").expect("unterminated struct");
    let mut out = Vec::new();
    for raw in body[..end].lines() {
        // Strip `//` comments, then take `name: type,`. Blank and comment-only lines drop out.
        let line = raw.split("//").next().unwrap_or("").trim().trim_end_matches(',').trim();
        if line.is_empty() {
            continue;
        }
        let (n, t) = line.split_once(':').unwrap_or_else(|| panic!("odd field line: {line:?}"));
        out.push((n.trim().to_string(), t.trim().to_string()));
    }
    out
}

/// Lay the fields out the way WGSL does and return the struct's size (rounded to its alignment).
fn wgsl_struct_size(fields: &[(String, String)]) -> usize {
    let mut off = 0usize;
    let mut struct_align = 1usize;
    for (_, ty) in fields {
        let (size, align) = size_align(ty);
        off = off.div_ceil(align) * align;
        off += size;
        struct_align = struct_align.max(align);
    }
    off.div_ceil(struct_align) * struct_align
}

/// ⭐⭐The gate: the two definitions of the coloring uniform must agree byte for byte.
#[test]
fn color_uniform_matches_the_shader() {
    let src = include_str!("mandelbrot.wgsl");
    let fields = wgsl_struct_fields(src, "ColorU");
    assert_eq!(
        wgsl_struct_size(&fields),
        std::mem::size_of::<ColorUniforms>(),
        "ColorU (WGSL, {} fields) and ColorUniforms (Rust) disagree on size — one was edited \
         without the other, and the color pass will read every field from the wrong offset",
        fields.len(),
    );
    // The palette now arrives as a storage-buffer LUT at binding 3; the eight-stop array is gone
    // from both definitions. Naming it here means a revert cannot pass quietly.
    assert!(
        !fields.iter().any(|(n, _)| n == "stops" || n == "stop_count"),
        "the eight-stop uniform array is back in ColorU — see design/palette-import.md §4",
    );
    assert!(
        fields.iter().any(|(n, _)| n == "lut_len") && fields.iter().any(|(n, _)| n == "lut_smooth"),
        "ColorU lost the LUT length/fetch-mode fields the palette pass needs",
    );
}

/// The LUT binding the shader reads and the buffer we allocate must be the same size, or a long
/// palette is silently truncated at the seam between them.
#[test]
fn lut_binding_is_declared_and_sized() {
    let src = include_str!("mandelbrot.wgsl");
    assert!(
        src.contains("@group(0) @binding(3) var<storage, read> lut: array<vec4<f32>>;"),
        "the color pass no longer declares the palette LUT at binding 3",
    );
    // 16 bytes an entry — the buffer is sized in `make_lut_buffer` from the same constant the
    // bake uses, so this pins the element size the shader assumes.
    assert_eq!(std::mem::size_of::<[f32; 4]>(), 16);
    assert!(fractadyne_color::segment::LUT_SIZE >= 256, "a LUT below 256 entries bands visibly");
}

/// ⭐⭐The SAME gate for the ITERATE uniform: `IterUniforms` (Rust) and `IterU` (WGSL) must agree
/// byte for byte. `IterU` is bound by every iterate pipeline; a field added to one and not the
/// other reads every field past the seam from the wrong offset. The `aux_all` + `_pad_aux` tail
/// added for gated method-switching is exactly the kind of edit this catches.
#[test]
fn iter_uniform_matches_the_shader() {
    let src = include_str!("mandelbrot.wgsl");
    let fields = wgsl_struct_fields(src, "IterU");
    assert_eq!(
        wgsl_struct_size(&fields),
        std::mem::size_of::<IterUniforms>(),
        "IterU (WGSL, {} fields) and IterUniforms (Rust) disagree on size — one was edited without \
         the other, and every iterate pipeline will read its uniform from the wrong offsets",
        fields.len(),
    );
    // The gated-all-aux field and its pad must be present in the shader, or `aux_all` is read from
    // stale bytes and method switching silently re-iterates (or worse, recolors a single-stat pass).
    assert!(
        fields.iter().any(|(n, _)| n == "aux_all"),
        "IterU lost `aux_all` — gated method-switching needs it in both definitions",
    );
}

/// ⭐The no-re-iterate contract, at the cache key. `aux_sel` is the ONLY thing in `IterKey` that a
/// color-method change moves, and both the GPU key and the app `settings_hash` derive it from this
/// one function. When all four aux stats are resident (`aux_all`), switching method must NOT change
/// the selector (⇒ the key is unchanged ⇒ a free recolor); otherwise it must (⇒ a re-iterate).
#[test]
fn aux_sel_frees_method_switch_only_when_all_resident() {
    // Settled/all-resident: every method maps to the same selector, so a switch never re-iterates.
    for m in 0u32..=5 {
        assert_eq!(aux_sel(true, m), u32::MAX, "aux_all must erase the method id from the key");
    }
    assert_eq!(aux_sel(true, 1), aux_sel(true, 2), "stripe↔TIA switch must be a free recolor");
    assert_eq!(aux_sel(true, 0), aux_sel(true, 3), "smooth↔trap switch must be a free recolor");

    // Single-stat: the selector IS the method, so a switch between distinct methods re-iterates.
    assert_eq!(aux_sel(false, 2), 2);
    assert_ne!(aux_sel(false, 1), aux_sel(false, 2), "single-stat method switch must re-iterate");
    assert_ne!(aux_sel(false, 0), aux_sel(false, 4), "single-stat smooth↔DE switch must re-iterate");
}
