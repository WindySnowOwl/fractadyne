//! The tiles instrument drawn headlessly for every kind of pass it shows: egui's own shapes,
//! tessellated as egui does, rasterized here in software (vertex colours, the font atlas, egui's
//! premultiplied gamma-space blending). The checks read the plane back pixel by pixel: which tiles
//! are lit and where. Set `FRACTADYNE_TEST_DUMP` to a folder to also write each case as a PNG.

use super::super::model::{Grid, Model, Plane};
use super::*;
use egui::epaint::{ImageData, Primitive};
use std::collections::HashMap;

/// A software framebuffer: premultiplied RGBA in gamma space, 0..=255 as f32.
struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

impl Canvas {
    fn at(&self, p: Pos2) -> [f32; 4] {
        let (x, y) = ((p.x.floor() as usize).min(self.w - 1), (p.y.floor() as usize).min(self.h - 1));
        self.px[y * self.w + x]
    }
    /// Brightness of the pixel at `p` (the amber's green channel tracks its alpha best).
    fn lum(&self, p: Pos2) -> f32 {
        let c = self.at(p);
        0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
    }
    fn dump(&self, name: &str) {
        let Some(dir) = std::env::var_os("FRACTADYNE_TEST_DUMP") else { return };
        let bytes: Vec<u8> = self.px.iter().flat_map(|c| [c[0] as u8, c[1] as u8, c[2] as u8, 255]).collect();
        let path = std::path::Path::new(&dir).join(format!("tiles-{name}.png"));
        fractadyne_export::write_png_rgba8(&path, self.w as u32, self.h as u32, &bytes, None).unwrap();
    }
}

/// Textures as egui hands them over: whole images, then patches.
#[derive(Default)]
struct Textures(HashMap<egui::TextureId, (usize, usize, Vec<Color32>)>);

impl Textures {
    fn apply(&mut self, delta: &egui::TexturesDelta) {
        for (id, d) in &delta.set {
            let (size, pixels): ([usize; 2], Vec<Color32>) = match &d.image {
                ImageData::Color(img) => (img.size, img.pixels.clone()),
                ImageData::Font(img) => (img.size, img.srgba_pixels(None).collect()),
            };
            match d.pos {
                None => {
                    self.0.insert(*id, (size[0], size[1], pixels));
                }
                Some([x0, y0]) => {
                    let (w, _, buf) = self.0.get_mut(id).expect("a patch of a texture egui never sent whole");
                    for y in 0..size[1] {
                        for x in 0..size[0] {
                            buf[(y0 + y) * *w + x0 + x] = pixels[y * size[0] + x];
                        }
                    }
                }
            }
        }
    }
    fn sample(&self, id: egui::TextureId, uv: egui::Pos2) -> [f32; 4] {
        let Some((w, h, buf)) = self.0.get(&id) else { return [255.0; 4] };
        let x = ((uv.x * *w as f32) as usize).min(w - 1);
        let y = ((uv.y * *h as f32) as usize).min(h - 1);
        let c = buf[y * w + x];
        [c.r() as f32, c.g() as f32, c.b() as f32, c.a() as f32]
    }
}

/// Fills a mesh's triangles into `canvas`, clipped to `clip`.
fn raster(canvas: &mut Canvas, tex: &Textures, clip: Rect, mesh: &egui::Mesh) {
    for t in mesh.indices.chunks_exact(3) {
        let [a, b, c] = [t[0], t[1], t[2]].map(|i| mesh.vertices[i as usize]);
        let area = (b.pos.x - a.pos.x) * (c.pos.y - a.pos.y) - (b.pos.y - a.pos.y) * (c.pos.x - a.pos.x);
        if area.abs() < 1e-6 {
            continue;
        }
        let lo = a.pos.min(b.pos).min(c.pos).max(clip.min).max(Pos2::ZERO);
        let hi = a.pos.max(b.pos).max(c.pos).min(clip.max).min(pos2(canvas.w as f32, canvas.h as f32));
        for y in lo.y.floor() as usize..hi.y.ceil() as usize {
            for x in lo.x.floor() as usize..hi.x.ceil() as usize {
                // Sampled a hair off the pixel centre, so a pixel on an edge two triangles share
                // (every rectangle's diagonal) falls in exactly one of them, as the GPU's fill rule
                // has it; a centre exactly on the edge would be filled twice.
                let p = pos2(x as f32 + 0.500_13, y as f32 + 0.500_29);
                let w0 = ((b.pos.x - p.x) * (c.pos.y - p.y) - (b.pos.y - p.y) * (c.pos.x - p.x)) / area;
                let w1 = ((c.pos.x - p.x) * (a.pos.y - p.y) - (c.pos.y - p.y) * (a.pos.x - p.x)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let mix = |f: fn(&egui::epaint::Vertex) -> f32| w0 * f(&a) + w1 * f(&b) + w2 * f(&c);
                let col = [mix(|v| v.color.r() as f32), mix(|v| v.color.g() as f32), mix(|v| v.color.b() as f32), mix(|v| v.color.a() as f32)];
                let uv = pos2(mix(|v| v.uv.x), mix(|v| v.uv.y));
                let tx = tex.sample(mesh.texture_id, uv);
                let src: [f32; 4] = std::array::from_fn(|i| col[i] * tx[i] / 255.0);
                let d = &mut canvas.px[y * canvas.w + x];
                for i in 0..4 {
                    d[i] = src[i] + d[i] * (1.0 - src[3] / 255.0);
                }
            }
        }
    }
}

/// The tiles instrument for `m`, `size` px with its face at the origin, at scale `s`: the canvas
/// and the instrument's rect.
fn render(m: &Model, size: egui::Vec2, s: f32) -> (Canvas, Rect) {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    paint::install_fonts(&mut fonts);
    ctx.set_fonts(fonts);
    let screen = Rect::from_min_size(Pos2::ZERO, size + vec2(20.0, 20.0));
    let raw = || egui::RawInput { screen_rect: Some(screen), ..Default::default() };
    let r = Rect::from_min_size(pos2(10.0, 10.0), size);
    let mut tex = Textures::default();
    // The fonts take effect on the next frame.
    tex.apply(&ctx.run(raw(), |_| {}).textures_delta);
    let out = ctx.run(raw(), |ctx| {
        let p = ctx.layer_painter(egui::LayerId::background());
        draw_tiles(&p, Layer::Static, r, s, m);
        draw_tiles(&p, Layer::Dynamic, r, s, m);
    });
    tex.apply(&out.textures_delta);
    let mut canvas = Canvas { w: screen.width() as usize, h: screen.height() as usize, px: vec![[0.0, 0.0, 0.0, 255.0]; (screen.width() * screen.height()) as usize] };
    for cp in ctx.tessellate(out.shapes, out.pixels_per_point) {
        if let Primitive::Mesh(mesh) = &cp.primitive {
            raster(&mut canvas, &tex, cp.clip_rect, mesh);
        }
    }
    (canvas, r)
}

const SIZE: egui::Vec2 = egui::vec2(900.0, 560.0);
/// The instrument's own size on the full panel at scale 1, for the pictures.
const REAL: egui::Vec2 = egui::vec2(320.0, 172.0);
const RES: (u32, u32) = (1920, 1102);

fn lum_of(c: Color32) -> f32 {
    0.3 * c.r() as f32 + 0.59 * c.g() as f32 + 0.11 * c.b() as f32
}
/// Halfway between two levels' brightness: the line between them on the plane.
fn between(a: f32, b: f32) -> f32 {
    (lum_of(level(a)) + lum_of(level(b))) / 2.0
}

fn model(plane: Plane) -> Model {
    let mut m = Model::default();
    m.snap.res = RES;
    m.plane = plane;
    m
}

/// A settle grid of `side`-px tiles over `res`, the first `done` tiles of its centre-out order
/// finished, the next one `strip` (rows sent, piece rows) on its way if given — as the renderer
/// would leave it (`render.rs`: the grid's visit order is by distance from the grid's centre).
fn grid(res: (u32, u32), side: u32, done: usize, strip: Option<(u32, u32)>) -> (Grid, Vec<u32>) {
    let (cols, rows) = (res.0.div_ceil(side).max(1), res.1.div_ceil(side).max(1));
    let n = (cols * rows) as usize;
    let mut order: Vec<u32> = (0..cols * rows).collect();
    let (cx, cy) = ((cols as f64 - 1.0) / 2.0, (rows as f64 - 1.0) / 2.0);
    let dist = |i: u32| ((i % cols) as f64 - cx).powi(2) + ((i / cols) as f64 - cy).powi(2);
    order.sort_by(|&a, &b| dist(a).partial_cmp(&dist(b)).unwrap());
    let mut done_v = vec![false; n];
    for &i in &order[..done.min(n)] {
        done_v[i as usize] = true;
    }
    let tile_rows = |i: u32| side.min(res.1 - (i / cols) * side);
    let last = match strip {
        Some((sent, piece)) if done < n => Some((order[done], sent, tile_rows(order[done]), piece)),
        _ if done > 0 => Some((order[done - 1], tile_rows(order[done - 1]), tile_rows(order[done - 1]), tile_rows(order[done - 1]))),
        _ => None,
    };
    (Grid { res, cols, rows, side, done: done_v, flash: vec![0.0; n], last, complete: done >= n }, order)
}

/// Every tile's centre on the plane, and whether it reads lit.
fn read_grid(c: &Canvas, plane: Rect, sc: f32, g: &Grid) -> Vec<bool> {
    let cs = g.side as f32 * sc;
    (0..g.cols * g.rows).map(|i| c.lum(tile_rect(plane, cs, g.cols, i).center()) > between(LEVEL_TO_COME, LEVEL_DONE)).collect()
}

#[test]
fn a_settle_grid_lights_exactly_its_finished_tiles_at_every_tile_size() {
    // From the finest grid the side allows (16 px: 120 × 69 tiles) to two tiles across.
    for side in [16u32, 32, 64, 128, 160, 256, 512, 1000] {
        let (cols, rows) = (RES.0.div_ceil(side), RES.1.div_ceil(side));
        let n = (cols * rows) as usize;
        // Tiles under 3 panel px cannot be told apart by their centres; their coverage is checked.
        for done in [0, 1, n / 3, n - 1, n] {
            let (g, _) = grid(RES, side, done, None);
            let m = model(Plane::Grid(g.clone()));
            let (c, r) = render(&m, SIZE, 1.0);
            c.dump(&format!("grid-{side}-{done}of{n}"));
            render(&m, REAL, 1.0).0.dump(&format!("real-grid-{side}-{done}of{n}"));
            let (plane, sc) = plane_rect(r, 1.0, RES);
            if side as f32 * sc >= 3.0 {
                let lit = read_grid(&c, plane, sc, &g);
                // The tile in progress is outlined, and its centre may read lit through it.
                let progress = g.last.filter(|_| !g.complete).map(|l| l.0 as usize);
                for (i, (&want, &got)) in g.done.iter().zip(lit.iter()).enumerate() {
                    if Some(i) == progress {
                        continue;
                    }
                    assert_eq!(got, want, "side {side} done {done}: tile {i} ({}, {}) lit {got}, want {want}", i as u32 % cols, i as u32 / cols);
                }
            }
            // Coverage: the plane's mean brightness grows with the tiles done.
            let mean = |c: &Canvas| {
                let (mut sum, mut k) = (0.0, 0);
                for y in (plane.top() as usize + 1)..(plane.bottom() as usize - 1) {
                    for x in (plane.left() as usize + 1)..(plane.right() as usize - 1) {
                        sum += c.lum(pos2(x as f32 + 0.5, y as f32 + 0.5));
                        k += 1;
                    }
                }
                sum / k as f32
            };
            if done == n {
                let (g0, _) = grid(RES, side, 0, None);
                let (c0, _) = render(&model(Plane::Grid(g0)), SIZE, 1.0);
                // The gaps between tiles stay at the plane's own level: at 16-px tiles they are a third of it.
                assert!(mean(&c) > mean(&c0) * 1.8, "side {side}: a finished grid {:.1} vs an empty one {:.1}", mean(&c), mean(&c0));
            }
            c.dump(&format!("grid-{side}-{done}of{n}"));
        }
    }
}

#[test]
fn a_tile_going_out_in_strips_fills_from_its_top() {
    // 512-px tiles, the third in its visit order a third sent, in strips of a sixth of its rows
    // (the bottom row's tiles are 78 rows, not 512).
    let (mut g, order) = grid(RES, 512, 2, None);
    let rows = 512.min(RES.1 - (order[2] / g.cols) * 512);
    let (sent, piece) = (rows / 3, (rows / 6).max(1));
    g.last = Some((order[2], sent, rows, piece));
    g.complete = false;
    let m = model(Plane::Grid(g.clone()));
    let (c, r) = render(&m, SIZE, 1.0);
    let (plane, sc) = plane_rect(r, 1.0, RES);
    let t = tile_rect(plane, 512.0 * sc, g.cols, order[2]);
    let (top, bottom) = (pos2(t.center().x, t.top() + t.height() * 0.2), pos2(t.center().x, t.top() + t.height() * 0.8));
    assert!(c.lum(top) > c.lum(bottom) * 2.0, "sent rows {:.0} vs the rest {:.0}", c.lum(top), c.lum(bottom));
    let (l1, tl, tr) = tile_labels(&m.plane, RES);
    assert!(l1.ends_with("2 OF 12 DONE · IN STRIPS"), "{l1}");
    assert_eq!((tl.as_str(), tr.as_str()), ("TILE 512 PX", format!("{:.2} TILE/FRAME", piece as f32 / rows as f32).as_str()));
    c.dump("grid-512-strips");
}

#[test]
fn a_split_refresh_lights_its_own_set_brightest_and_the_sets_before_it() {
    // At a size where its 16-px tiles show, and at the instrument's own, where they are drawn as
    // the same interleave at a legible cell.
    for (size, tag) in [(SIZE, ""), (REAL, "real-")] {
    for k in [2u32, 3, 4, 8] {
        for j in [0, k / 2, k - 1] {
            let m = model(Plane::Split { res: RES, k, j, glow: 1.0 });
            let (c, r) = render(&m, size, 1.0);
            c.dump(&format!("{tag}split-{k}-pass{}", j + 1));
            let (plane, _) = plane_rect(r, 1.0, RES);
            let (cols, rows) = split_display(plane, RES);
            if size == SIZE {
                assert_eq!((cols, rows), split_cells(RES), "big enough to draw the real tiles");
            } else {
                assert!(plane.width() / cols as f32 >= MIN_CELL_PX - 0.01, "a legible cell at real size");
            }
            let (cw, ch) = (plane.width() / cols as f32, plane.height() / rows as f32);
            // Three classes: this pass's set, the sets before it (done), the sets after it.
            let mut by = [Vec::new(), Vec::new(), Vec::new()];
            for y in (0..rows).step_by(3) {
                for x in (0..cols).step_by(2) {
                    let set = (x + y) % k;
                    let class = if set == j { 0 } else if set < j { 1 } else { 2 };
                    by[class].push(c.lum(pos2(plane.left() + (x as f32 + 0.45) * cw, plane.top() + (y as f32 + 0.45) * ch)));
                }
            }
            let mean = |v: &Vec<f32>| v.iter().sum::<f32>() / v.len().max(1) as f32;
            let (this, before, after) = (mean(&by[0]), mean(&by[1]), mean(&by[2]));
            // Each class shows its own level: this pass's, done, to come.
            let near = |got: f32, t: f32| (got - lum_of(level(t))).abs() < 6.0;
            assert!(near(this, 1.0), "k {k} j {j}: this pass {this:.0} vs {:.0}", lum_of(level(1.0)));
            assert!(by[1].is_empty() || near(before, LEVEL_DONE), "k {k} j {j}: done sets {before:.0} vs {:.0}", lum_of(level(LEVEL_DONE)));
            assert!(by[2].is_empty() || near(after, LEVEL_TO_COME), "k {k} j {j}: sets to come {after:.0} vs {:.0}", lum_of(level(LEVEL_TO_COME)));
            // ...and far enough apart to read: the pass at least 1.6x the done sets, the done sets
            // at least 2x the sets to come.
            assert!(lum_of(level(LEVEL_PASS)) > 1.6 * lum_of(level(LEVEL_DONE)) && lum_of(level(LEVEL_DONE)) > 2.0 * lum_of(level(LEVEL_TO_COME)));
            // The captions keep the truth whatever cell the plane draws.
            let (tc, tr_) = split_cells(RES);
            let (l1, tl, tr) = tile_labels(&m.plane, RES);
            assert_eq!(l1, format!("SPLIT REFRESH · PASS {} OF {k}", j + 1));
            assert_eq!(tl, "TILE 16 PX");
            assert_eq!(tr, format!("{}/FRAME", group(((tc * tr_) as f32 / k as f32).ceil() as u64)));
        }
    }
    }
}

#[test]
fn small_settle_tiles_meet_edge_to_edge_so_a_finished_grid_lights_the_plane() {
    // At the instrument's own size a 16-px tile is ~2 panel px: with a gap round each, a finished
    // grid read as a field of dots.
    for side in [16u32, 32] {
        let (cols, rows) = (RES.0.div_ceil(side), RES.1.div_ceil(side));
        let n = (cols * rows) as usize;
        let lit_share = |done: usize| {
            let (g, _) = grid(RES, side, done, None);
            let (c, r) = render(&model(Plane::Grid(g)), REAL, 1.0);
            let (plane, _) = plane_rect(r, 1.0, RES);
            let line = between(LEVEL_TO_COME, LEVEL_DONE);
            let (mut lit, mut all) = (0, 0);
            for y in (plane.top() as usize + 2)..(plane.bottom() as usize - 2) {
                for x in (plane.left() as usize + 2)..(plane.right() as usize - 2) {
                    all += 1;
                    if c.lum(pos2(x as f32 + 0.5, y as f32 + 0.5)) > line {
                        lit += 1;
                    }
                }
            }
            (lit as f32 / all as f32, c.lum(plane.center()) > line, c.lum(plane.left_top() + vec2(3.0, 3.0)) > line)
        };
        let (full, _, _) = lit_share(n);
        assert!(full > 0.97, "side {side}: a finished grid lights {:.0}% of the plane", 100.0 * full);
        // A third done, centre out: a lit middle and dark corners.
        let (third, centre, corner) = lit_share(n / 3);
        assert!((0.2..0.5).contains(&third), "side {side}: a third done lights {:.0}%", 100.0 * third);
        assert!(centre && !corner, "side {side}: centre lit {centre}, corner lit {corner}");
    }
}

#[test]
fn whole_frames_supersampling_and_idle_fill_the_plane_or_leave_it_dark() {
    let cases = [
        ("whole", Plane::Whole { res: RES, band: None, settled: false, glow: 1.0 }, true),
        ("pin", Plane::Whole { res: RES, band: Some((0, 22_224)), settled: false, glow: 1.0 }, true),
        ("settle-walk", Plane::Whole { res: RES, band: Some((65_536, 98_304)), settled: true, glow: 1.0 }, true),
        ("ssaa", Plane::Ssaa { res: RES, samples: 16, band: None, glow: 1.0 }, true),
        ("ssaa-walk", Plane::Ssaa { res: RES, samples: 3, band: Some((1_024, 2_048)), glow: 1.0 }, true),
        ("idle", Plane::Idle("HELD"), false),
    ];
    for (name, plane_kind, lit) in cases {
        let m = model(plane_kind);
        let (c, r) = render(&m, SIZE, 1.0);
        c.dump(name);
        render(&m, REAL, 1.0).0.dump(&format!("real-{name}"));
        let (plane, _) = plane_rect(r, 1.0, RES);
        let samples = [0.1f32, 0.5, 0.9].iter().flat_map(|&fx| [0.1f32, 0.5, 0.9].map(|fy| pos2(plane.left() + plane.width() * fx, plane.top() + plane.height() * fy))).collect::<Vec<_>>();
        for p in samples {
            assert_eq!(c.lum(p) > between(LEVEL_PLANE, LEVEL_DONE), lit, "{name} at {p:?}: {:.0}", c.lum(p));
        }
        c.dump(name);
    }
    assert_eq!(tile_labels(&Plane::Ssaa { res: RES, samples: 16, band: None, glow: 0.0 }, RES).2, "1 TILE/SAMPLE");
    assert_eq!(tile_labels(&Plane::Idle("HELD"), RES), ("HELD".to_owned(), "NO PASS".to_owned(), "0/FRAME".to_owned()));
}

#[test]
fn the_plane_keeps_the_pictures_aspect_and_fits_above_the_captions() {
    for res in [(1920u32, 1102u32), (3840, 2160), (800, 1200), (64, 64)] {
        let r = Rect::from_min_size(pos2(10.0, 10.0), SIZE);
        let (plane, sc) = plane_rect(r, 1.0, res);
        assert!((plane.width() / plane.height() - res.0 as f32 / res.1 as f32).abs() < 0.01, "{res:?}: {plane:?}");
        assert!(plane.left() >= r.left() && plane.right() <= r.right() && plane.bottom() <= r.bottom() - 30.0, "{res:?}: {plane:?} in {r:?}");
        assert!((plane.width() - res.0 as f32 * sc).abs() < 0.01);
    }
}
