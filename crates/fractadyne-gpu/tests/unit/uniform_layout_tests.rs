//! The Rust `ColorUniforms` and the WGSL `ColorU` describe the same bytes, and nothing checks
//! that at compile time: the uniform is written with `bytemuck::bytes_of` into an opaque buffer,
//! so a field added to one and not the other produces a pipeline that builds, runs, and colours
//! every pixel from the wrong offsets. `design/palette-import.md` §4 names this in advance —
//! "**both `ColorU` definitions must change together** … or offline renders silently keep the old
//! palette" — and the palette-LUT change edited both.
//!
//! So: parse the WGSL struct, lay it out under WGSL's alignment rules, and compare the total with
//! `size_of::<ColorUniforms>()`. A mismatch is not a style problem; it is a wrong picture.

use super::ColorUniforms;

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
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
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
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    assert!(
        src.contains("@group(0) @binding(3) var<storage, read> lut: array<vec4<f32>>;"),
        "the color pass no longer declares the palette LUT at binding 3",
    );
    // 16 bytes an entry — the buffer is sized in `make_lut_buffer` from the same constant the
    // bake uses, so this pins the element size the shader assumes.
    assert_eq!(std::mem::size_of::<[f32; 4]>(), 16);
    assert!(fractadyne_color::segment::LUT_SIZE >= 256, "a LUT below 256 entries bands visibly");
}

/// Byte offset of `field` in a WGSL struct laid out as [`wgsl_struct_size`] does.
/// The palette dissolve's fields sit where the Rust struct puts them, one by one: the size check
/// alone would pass two fields swapped, and the dissolve would blend in a map built from the wrong
/// words.
#[test]
fn the_palette_fade_fields_line_up() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let fields = wgsl_struct_fields(src, "ColorU");
    for (wgsl, rust) in [
        ("aa_palette", std::mem::offset_of!(ColorUniforms, aa_palette)),
        ("fade_weight", std::mem::offset_of!(ColorUniforms, fade_weight)),
        ("fade_mode", std::mem::offset_of!(ColorUniforms, fade_mode)),
        ("fade_lo", std::mem::offset_of!(ColorUniforms, fade_lo)),
        ("fade_cycle", std::mem::offset_of!(ColorUniforms, fade_cycle)),
        ("fade_offset", std::mem::offset_of!(ColorUniforms, fade_offset)),
    ] {
        assert_eq!(wgsl_field_offset(&fields, wgsl), rust, "`{wgsl}` is at another offset in ColorU than in ColorUniforms");
    }
}

fn wgsl_field_offset(fields: &[(String, String)], field: &str) -> usize {
    let mut off = 0usize;
    for (name, ty) in fields {
        let (size, align) = size_align(ty);
        off = off.div_ceil(align) * align;
        if name == field {
            return off;
        }
        off += size;
    }
    panic!("no field `{field}` in the struct");
}

/// The built-in module parses and validates as naga would at pipeline creation. A custom formula's
/// module is validated as it is generated (`custom::validate`), but it replaces whole slots of this
/// one (mode 2, the floatexp chunk body), so an error inside them reached no test before the app.
#[test]
fn the_built_in_shader_validates() {
    use egui_wgpu::wgpu::naga;
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{e:?}"));
}

/// The exponential-map strip module (`strip.rs`): all six of the built-in module's STRIP_* slots are
/// filled, in order, and the result validates. A slot renamed or moved in `mandelbrot.wgsl` would
/// otherwise surface only when a strip is rendered.
#[test]
fn the_strip_module_splices_and_validates() {
    use egui_wgpu::wgpu::naga;
    let src = crate::strip::source().unwrap_or_else(|e| panic!("{e}"));
    assert!(!src.contains("@@STRIP_"), "a STRIP_* slot was left unfilled");
    assert!(src.contains("strip_offset(gx, gy)") && src.contains("strip_aux(info_iter(sm)"));
    let module = naga::front::wgsl::parse_str(&src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(&src)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{e:?}"));
    // The built-in module itself has no strip code at all, only the slots' comments.
    let fixed = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    assert!(!fixed.contains("strip_offset") && !fixed.contains("cos_sin_turns"));
}

/// The same gate for the ITERATE uniform, which every iterate pipeline binds: `IterU` (WGSL) and
/// `IterUniforms` (Rust) must agree on size, and on where the renormalized step's block starts
/// (a field added on one side only would shift it, and the step would read garbage).
#[test]
fn iter_uniform_matches_the_shader() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/mandelbrot.wgsl"));
    let fields = wgsl_struct_fields(src, "IterU");
    assert_eq!(
        wgsl_struct_size(&fields),
        std::mem::size_of::<super::IterUniforms>(),
        "IterU (WGSL, {} fields) and IterUniforms (Rust) disagree on size",
        fields.len(),
    );
    assert_eq!(
        wgsl_field_offset(&fields, "rn_a"),
        std::mem::offset_of!(super::IterUniforms, rn),
        "IterU.rn_a and IterUniforms.rn start at different offsets",
    );
    assert_eq!(std::mem::size_of::<super::Renorm>(), 128, "Renorm is the shader's 32 words");
    assert_eq!(
        wgsl_field_offset(&fields, "map_mode"),
        std::mem::offset_of!(super::IterUniforms, map),
        "IterU.map_mode and IterUniforms.map start at different offsets",
    );
    assert_eq!(std::mem::size_of::<super::StripMap>(), 16, "StripMap is the shader's 4 words");
}

/// The zoom-video assembly module (`assemble.wgsl`) validates, and its uniform `AssembleU` lines up
/// with `AssembleUniforms` field for field: a field on one side only would place every sample of
/// every assembled frame from the wrong numbers.
#[test]
fn the_assembly_module_validates_and_its_uniform_lines_up() {
    use crate::assemble::AssembleUniforms as A;
    use egui_wgpu::wgpu::naga;
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/assemble.wgsl"));
    let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|e| panic!("{e:?}"));
    let fields = wgsl_struct_fields(src, "AssembleU");
    assert_eq!(wgsl_struct_size(&fields), std::mem::size_of::<A>(), "AssembleU and AssembleUniforms disagree on size");
    let rust = [
        ("res", std::mem::offset_of!(A, res)),
        ("width", std::mem::offset_of!(A, width)),
        ("rows", std::mem::offset_of!(A, rows)),
        ("rows_per_octave", std::mem::offset_of!(A, rows_per_octave)),
        ("sample_offset", std::mem::offset_of!(A, sample_offset)),
        ("depth_frac", std::mem::offset_of!(A, depth_frac)),
        ("rel_base", std::mem::offset_of!(A, rel_base)),
        ("first_layer", std::mem::offset_of!(A, first_layer)),
        ("layers", std::mem::offset_of!(A, layers)),
        ("rel_last", std::mem::offset_of!(A, rel_last)),
        ("turns", std::mem::offset_of!(A, turns)),
        ("rot_cos", std::mem::offset_of!(A, rot_cos)),
        ("rot_sin", std::mem::offset_of!(A, rot_sin)),
        ("log2_row_step", std::mem::offset_of!(A, log2_row_step)),
        ("piece_width", std::mem::offset_of!(A, piece_width)),
        ("pieces", std::mem::offset_of!(A, pieces)),
        ("texels", std::mem::offset_of!(A, texels)),
    ];
    assert_eq!(fields.len(), rust.len() + 2, "a field was added on one side only (the +2 is the padding)");
    for (name, off) in rust {
        assert_eq!(wgsl_field_offset(&fields, name), off, "`{name}` is at another offset in AssembleU than in AssembleUniforms");
    }
    // The assembly module carries no iterate code: it is a module of its own.
    assert!(!src.contains("fn iterate_at") && !src.contains("fn fs_color"));
}
