//! The help card over the live-render panel, driven through egui's own input: the pointer rests on
//! an instrument and the card opens, and stays open. A card placed over the pointer takes the
//! hover from the panel (egui hit-tests a tooltip-order area even when it is not interactable), so
//! it closed, the panel had the hover again, and it reopened a quarter-second later: the TILES card,
//! about 450 points tall, flickered in any window too short to hold it below or above the face
//! (7 openings in 2 s at 1280 x 720 with the panel bottom-left).

use super::*;
use egui::{Event, LayerId, Order, RawInput};

/// An open panel in a window of its own, frames at 60 a second.
struct Rig {
    ctx: egui::Context,
    o: Overlay,
    screen: Rect,
    t: f64,
}

/// What resting the pointer on a card's region did.
#[derive(Debug)]
struct Rest {
    /// How many times the card opened.
    opened: usize,
    /// Whether it was open at the end.
    open: bool,
    /// Whether the card was ever drawn over the pointer.
    covered: bool,
}

impl Rig {
    fn new(w: f32, h: f32, corner: Corner, size: Size) -> Rig {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        paint::install_fonts(&mut fonts);
        ctx.set_fonts(fonts);
        Rig { ctx, o: Overlay::from_session(size.name(), corner.name(), 0.95), screen: Rect::from_min_size(Pos2::ZERO, vec2(w, h)), t: 0.0 }
    }

    /// One frame with `event`, if any.
    fn frame(&mut self, event: Option<Event>) {
        self.t += 1.0 / 60.0;
        let input = RawInput { screen_rect: Some(self.screen), time: Some(self.t), events: event.into_iter().collect(), ..Default::default() };
        // The fractal area: the window less a menu bar and a status bar, as the app lays it out.
        let view = Rect::from_min_max(self.screen.min + vec2(0.0, 24.0), self.screen.max - vec2(0.0, 44.0));
        let o = &mut self.o;
        let _ = self.ctx.run(input, |ctx| o.show(ctx, view, 0.0, 0.0));
    }

    /// Whether a help card was drawn in the frame just run.
    fn card_shown(&self) -> bool {
        self.ctx.memory(|m| m.areas().visible_layer_ids().iter().any(|l: &LayerId| l.order == Order::Tooltip))
    }

    /// A point in `key`'s region that opens `key`'s card (a lamp sits in the middle of the bank),
    /// with no card open; `None` for a key this size of panel does not draw.
    fn point_on(&mut self, key: &'static str) -> Option<Pos2> {
        self.o.pinned_help = Some(key);
        self.frame(Some(Event::PointerGone));
        self.o.pinned_help = None;
        let region = self.o.hover.filter(|(k, _, _)| *k == key).map(|(_, rect, _)| rect)?;
        let grid = (1..8).flat_map(|i| (1..8).map(move |j| (i as f32 / 8.0, j as f32 / 8.0)));
        let found = [(0.5, 0.5)].into_iter().chain(grid).map(|(u, v)| region.lerp_inside(vec2(u, v))).find(|&p| {
            self.frame(Some(Event::PointerMoved(p)));
            self.o.hover.is_some_and(|(k, _, _)| k == key)
        });
        self.frame(Some(Event::PointerGone));
        self.frame(None);
        assert!(!self.card_shown());
        found
    }

    /// Rests the pointer on `key`'s card's region for 0.8 s.
    fn rest_on(&mut self, key: &'static str) -> Option<Rest> {
        let at = self.point_on(key)?;
        let mut r = Rest { opened: 0, open: false, covered: false };
        for i in 0..48 {
            self.frame((i == 0).then_some(Event::PointerMoved(at)));
            let now = self.card_shown();
            if now && !r.open {
                r.opened += 1;
            }
            r.open = now;
            r.covered |= self.o.card.is_some_and(|c| c.contains(at));
        }
        Some(r)
    }
}

const CORNERS: [Corner; 4] = [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight];

#[test]
fn the_tiles_card_opens_once_stays_and_keeps_off_the_pointer() {
    // 1280 x 720: the panel at the top leaves too little room below the face for the card, and at
    // the bottom too little above it. 1366 x 768 and 1536 x 864 are 1366 x 768 and 1920 x 1080
    // screens at 100% and 125%.
    for (w, h) in [(1280.0, 720.0), (1366.0, 768.0), (1280.0, 800.0), (1536.0, 864.0), (1920.0, 1080.0), (2560.0, 1080.0)] {
        for corner in CORNERS {
            let r = Rig::new(w, h, corner, Size::Panel).rest_on("tiles").expect("the panel draws the tiles face");
            assert!(r.open && r.opened == 1 && !r.covered, "{w} x {h}, {corner:?}: {r:?}");
        }
    }
}

#[test]
fn every_card_on_the_panel_opens_once_stays_and_keeps_off_the_pointer() {
    let keys: Vec<&'static str> = panel::HELP_KEYS.iter().chain(panel::LAMP_KEYS).copied().collect();
    let mut rested = std::collections::BTreeSet::new();
    for size in [Size::Panel, Size::Full] {
        for corner in [Corner::TopLeft, Corner::BottomRight] {
            let mut rig = Rig::new(1280.0, 720.0, corner, size);
            for &key in &keys {
                let Some(r) = rig.rest_on(key) else { continue };
                assert!(r.open && r.opened == 1 && !r.covered, "{key}, {size:?}, {corner:?}: {r:?}");
                rested.insert(key);
            }
        }
    }
    let missed: Vec<_> = keys.iter().filter(|k| !rested.contains(*k)).collect();
    assert!(missed.is_empty(), "never rested on: {missed:?}");
}

#[test]
fn in_a_window_too_small_to_keep_the_card_off_the_pointer_it_still_stays() {
    // 1024 x 640 and 760 x 420: the tiles card (437 points tall at 1024) fits nowhere clear of the
    // pointer, so egui moves it over the pointer; it stays open while the pointer is on it rather
    // than closing and reopening.
    for (w, h) in [(1024.0, 640.0), (760.0, 420.0)] {
        for corner in CORNERS {
            let r = Rig::new(w, h, corner, Size::Panel).rest_on("tiles").expect("the panel draws the tiles face");
            assert!(r.open && r.opened == 1, "{w} x {h}, {corner:?}: {r:?}");
        }
    }
}

#[test]
fn a_card_goes_below_then_above_then_beside_then_off_the_pointer() {
    let s = Rect::from_min_size(Pos2::ZERO, vec2(1280.0, 800.0));
    let size = vec2(500.0, 450.0);
    let at = |region: Rect| card_place(region, region.center(), size, s);
    // Room below.
    let r = Rect::from_min_size(Pos2::new(400.0, 100.0), vec2(200.0, 170.0));
    assert_eq!(at(r).top(), r.bottom() + 8.0);
    // None below, room above.
    let r = Rect::from_min_size(Pos2::new(400.0, 560.0), vec2(200.0, 170.0));
    assert_eq!(at(r).bottom(), r.top() - 8.0);
    // Neither: right of it, then left of it.
    let r = Rect::from_min_size(Pos2::new(400.0, 250.0), vec2(200.0, 170.0));
    assert_eq!(at(r).left(), r.right() + 8.0);
    let r = Rect::from_min_size(Pos2::new(700.0, 250.0), vec2(200.0, 170.0));
    assert_eq!(at(r).right(), r.left() - 8.0);
    // Nor beside (a wide region): below the pointer, over the region but not the pointer.
    let r = Rect::from_min_size(Pos2::new(300.0, 200.0), vec2(700.0, 300.0));
    let p = Pos2::new(600.0, 260.0);
    let c = card_place(r, p, size, s);
    assert!(c.intersects(r) && !c.contains(p) && s.contains_rect(c), "{c:?}");
}

#[test]
fn wherever_a_card_has_room_it_stays_on_the_screen_and_off_the_pointer() {
    let mut placed = 0;
    for (sw, sh) in [(800.0, 500.0), (1024.0, 640.0), (1280.0, 720.0), (1920.0, 1080.0)] {
        let s = Rect::from_min_size(Pos2::ZERO, vec2(sw, sh));
        for size in [vec2(350.0, 200.0), vec2(500.0, 450.0), vec2(500.0, 700.0)] {
            for rx in (0..10).map(|i| i as f32 * sw / 10.0) {
                for ry in (0..10).map(|i| i as f32 * sh / 10.0) {
                    for rsize in [vec2(40.0, 20.0), vec2(215.0, 190.0), vec2(600.0, 240.0)] {
                        let region = Rect::from_min_size(Pos2::new(rx, ry), rsize).intersect(s);
                        for p in [region.center(), region.left_top() + vec2(2.0, 2.0), region.right_bottom() - vec2(2.0, 2.0)] {
                            let c = card_place(region, p, size, s);
                            let fallback = Rect::from_min_size(Pos2::new(c.left(), region.bottom() + 8.0), size);
                            if c == fallback && !s.contains_rect(c) {
                                continue; // no room anywhere: egui moves it, and the card stays while on the pointer
                            }
                            assert!(s.contains_rect(c) && !c.contains(p), "screen {s:?} region {region:?} pointer {p:?}: {c:?}");
                            placed += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(placed > 3000, "only {placed} placements had room");
}
