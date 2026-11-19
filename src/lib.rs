//! Turkish grammar radial mind-map — WASM entry point.
//!
//! Everything interactive lives here: the node arena, camera, pointer / wheel /
//! keyboard handling, spring animation, hit-testing, fuzzy search and canvas
//! rendering. JavaScript only forwards DOM events and renders the inspector
//! drawer from the JSON this module hands it.

pub mod dictionary;
pub mod grammar_data;
pub mod layout;
pub mod player;

use std::cell::RefCell;
use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use grammar_data::{Example, NodeDef};
use layout::LayoutNode;

const BG: &str = "#0D0D12";
const CRIMSON: &str = "#DC143C";
const DEEP_RED: &str = "#9B111E";
const ROOT_R: f64 = 62.0;
const MIN_ZOOM: f64 = 0.08;
const MAX_ZOOM: f64 = 4.0;

const FONT_ROOT: &str = "600 14px Inter, system-ui, sans-serif";
const FONT_ROOT_SUB: &str = "400 10.5px Inter, system-ui, sans-serif";
/// Smallest on-screen label size (CSS px) worth drawing; smaller labels are
/// left out until the user zooms in (semantic zoom).
const MIN_LABEL_PX: f64 = 7.0;
/// Momentum decay per millisecond (Apple's scroll deceleration rate).
const DECELERATION: f64 = 0.998;
const HOUSE_ROOT_R: f64 = 44.0;

/// Which arrangement of the grammar is shown.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum View {
    /// Radial mind-map grouped by topic.
    Radial,
    /// House of six CEFR floors.
    Levels,
}

struct Node {
    def: NodeDef,
    parent: Option<usize>,
    children: Vec<usize>,
    depth: usize,
    expanded: bool,
    // Animated position and its spring velocity / target.
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    tx: f64,
    ty: f64,
    // 0 = hidden inside the parent, 1 = fully shown.
    appear: f64,
    va: f64,
    tappear: f64,
    label_w: f64,
    /// Normalised search haystack (title, translation, formula).
    hay: Vec<char>,
    matched: bool,
    on_match_path: bool,
    view: View,
    /// Number of leaf rules in this node's subtree (1 for a leaf).
    leaf_count: usize,
}

impl Node {
    fn radius(&self) -> f64 {
        match (self.view, self.depth) {
            (View::Radial, 0) => ROOT_R,
            (View::Radial, 1) => 11.0,
            (View::Radial, 2) => 7.0,
            (View::Radial, _) => 4.5,
            (View::Levels, 0) => HOUSE_ROOT_R,
            (View::Levels, 1) => 13.0,
            (View::Levels, 2) => 6.0,
            (View::Levels, _) => 4.0,
        }
    }
    /// Text drawn next to the node.
    fn label(&self) -> String {
        if self.view == View::Levels && self.depth == 1 {
            format!("{} · {}", self.def.tr, self.def.en)
        } else {
            self.def.tr.to_string()
        }
    }
    /// Label font size in CSS px before the user's text scale.
    fn font_px(&self) -> f64 {
        if self.depth == 1 {
            15.0
        } else if self.view == View::Levels && self.depth == 2 {
            14.0
        } else if self.def.tr.starts_with('-') {
            12.5
        } else {
            13.0
        }
    }
    fn font(&self, scale: f64) -> String {
        let px = self.font_px() * scale;
        if self.depth == 1 {
            format!("600 {px}px Inter, system-ui, sans-serif")
        } else if self.view == View::Levels && self.depth == 2 {
            format!("600 {px}px Inter, system-ui, sans-serif")
        } else if self.def.tr.starts_with('-') {
            format!("500 {px}px 'JetBrains Mono', ui-monospace, monospace")
        } else {
            format!("500 {px}px Inter, system-ui, sans-serif")
        }
    }
    /// Radial length taken up beyond the node centre (dot + label + padding).
    fn extent(&self) -> f64 {
        if self.depth == 0 {
            ROOT_R
        } else {
            self.radius() + self.label_w + 22.0
        }
    }
}

fn flatten(mut def: NodeDef, parent: Option<usize>, depth: usize, view: View, out: &mut Vec<Node>) -> usize {
    let kids = std::mem::take(&mut def.children);
    let id = out.len();
    // The title is indexed twice: as written, and with optional "(…)" parts
    // removed, so that "-yor" finds "-(I)yor".
    let mut bare = String::new();
    let mut nest = 0;
    for c in def.tr.chars() {
        match c {
            '(' => nest += 1,
            ')' => nest -= 1,
            _ if nest == 0 => bare.push(c),
            _ => {}
        }
    }
    let hay = normalize(&format!("{} {} {} {}", def.tr, def.en, def.formula, bare));
    out.push(Node {
        def,
        parent,
        children: Vec::new(),
        depth,
        expanded: depth <= 1,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        tx: 0.0,
        ty: 0.0,
        appear: if depth == 0 { 1.0 } else { 0.0 },
        va: 0.0,
        tappear: 0.0,
        label_w: 0.0,
        hay,
        matched: false,
        on_match_path: false,
        view,
        leaf_count: 1,
    });
    for k in kids {
        let c = flatten(k, Some(id), depth + 1, view, out);
        out[id].children.push(c);
    }
    if !out[id].children.is_empty() {
        out[id].leaf_count = out[id].children.iter().map(|&c| out[c].leaf_count).sum();
    }
    id
}

/// Lower-case and fold Turkish diacritics so `fistikci` finds `Fıstıkçı`.
pub fn normalize(s: &str) -> Vec<char> {
    s.chars()
        .map(|c| match c {
            'İ' | 'I' | 'ı' | 'î' | 'Î' => 'i',
            'Ç' | 'ç' => 'c',
            'Ğ' | 'ğ' => 'g',
            'Ö' | 'ö' => 'o',
            'Ş' | 'ş' => 's',
            'Ü' | 'ü' | 'û' | 'Û' => 'u',
            'Â' | 'â' => 'a',
            _ => c.to_lowercase().next().unwrap_or(c),
        })
        .collect()
}

/// Substring match, or — for queries of four or more characters — a
/// subsequence match whose span is at most two characters longer than the
/// query (tolerates a couple of skipped letters).
pub fn fuzzy(hay: &[char], q: &[char]) -> bool {
    if q.is_empty() {
        return false;
    }
    if hay.len() >= q.len() && hay.windows(q.len()).any(|w| w == q) {
        return true;
    }
    if q.len() < 4 {
        return false;
    }
    for start in 0..hay.len() {
        if hay[start] != q[0] {
            continue;
        }
        let limit = (start + q.len() + 2).min(hay.len());
        let mut qi = 1;
        for &c in &hay[start + 1..limit] {
            if qi < q.len() && c == q[qi] {
                qi += 1;
            }
        }
        if qi == q.len() {
            return true;
        }
    }
    false
}

/// One step of a critically damped spring. Returns `true` while still moving.
fn spring(x: &mut f64, v: &mut f64, target: f64, k: f64, dt: f64, eps: f64) -> bool {
    let d = target - *x;
    if d.abs() < eps && v.abs() < eps * 10.0 {
        *x = target;
        *v = 0.0;
        return false;
    }
    let c = 2.0 * k.sqrt();
    *v += (k * d - c * *v) * dt;
    *x += *v * dt;
    true
}

/// Release velocity in px/s from recent `(ms, x, y)` samples; zero when the
/// pointer had already stopped before it was released.
pub fn release_velocity(hist: &[(f64, f64, f64)], now: f64) -> (f64, f64) {
    let Some(&(t1, x1, y1)) = hist.last() else { return (0.0, 0.0) };
    if now - t1 > 80.0 {
        return (0.0, 0.0);
    }
    let first = hist.iter().find(|h| t1 - h.0 <= 100.0).copied().unwrap_or((t1, x1, y1));
    let dt = (t1 - first.0) / 1000.0;
    if dt <= 0.0 {
        return (0.0, 0.0);
    }
    ((x1 - first.1) / dt, (y1 - first.2) / dt)
}

/// Distance a fling travels before it stops (Apple's projection formula).
pub fn project(velocity: f64) -> f64 {
    (velocity / 1000.0) * DECELERATION / (1.0 - DECELERATION)
}

/// Where the camera should return to when the content box `(x0, y0, x1, y1)`
/// is entirely outside the view centred on `(cx, cy)` with half-size
/// `(hw, hh)`: just far enough to bring its nearest edge `margin` into view.
/// `None` while any part of the content is visible.
pub fn return_target(bbox: (f64, f64, f64, f64), cx: f64, cy: f64, hw: f64, hh: f64, margin: f64) -> Option<(f64, f64)> {
    let (x0, y0, x1, y1) = bbox;
    if x1 >= cx - hw && x0 <= cx + hw && y1 >= cy - hh && y0 <= cy + hh {
        return None;
    }
    let mut tx = cx;
    if x1 < cx - hw + margin {
        tx = x1 + hw - margin;
    } else if x0 > cx + hw - margin {
        tx = x0 - hw + margin;
    }
    let mut ty = cy;
    if y1 < cy - hh + margin {
        ty = y1 + hh - margin;
    } else if y0 > cy + hh - margin {
        ty = y0 - hh + margin;
    }
    Some((tx, ty))
}

/// Zoom with soft limits: past `MIN_ZOOM` / `MAX_ZOOM` the zoom follows the
/// request with growing resistance (rubber-band in log space).
pub fn soft_zoom(raw: f64) -> f64 {
    let band = |x: f64| (x * 0.25 * 0.55) / (0.25 + 0.55 * x.abs());
    if raw > MAX_ZOOM {
        MAX_ZOOM * band((raw / MAX_ZOOM).ln()).exp()
    } else if raw < MIN_ZOOM {
        MIN_ZOOM * (-band((MIN_ZOOM / raw).ln())).exp()
    } else {
        raw
    }
}

#[derive(Serialize)]
struct Crumb<'a> {
    id: usize,
    tr: &'a str,
    en: &'a str,
}

#[derive(Serialize)]
struct Inspect<'a> {
    id: usize,
    tr: &'a str,
    en: &'a str,
    level: &'a str,
    formula: &'a str,
    harmony: &'a str,
    notes: &'a str,
    leaf: bool,
    examples: &'a [Example],
    path: Vec<Crumb<'a>>,
    children: Vec<Crumb<'a>>,
}

struct State {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    nodes: Vec<Node>,
    w: f64,
    h: f64,
    dpr: f64,
    // Camera: world point at the centre of the screen, and zoom.
    cx: f64,
    cy: f64,
    zoom: f64,
    vcx: f64,
    vcy: f64,
    vz: f64,
    tcx: f64,
    tcy: f64,
    tzoom: f64,
    /// While a wheel zoom animates: (screen x, screen y, world x, world y)
    /// that must stay glued together.
    anchor: Option<(f64, f64, f64, f64)>,
    pointers: Vec<(i32, f64, f64)>,
    press: (f64, f64),
    last: (f64, f64),
    dragged: bool,
    pinch: f64,
    /// True until the user moves the camera; while set, resizes re-fit the view.
    auto_fit: bool,
    hover: Option<usize>,
    selected: Option<usize>,
    searching: bool,
    saved_expanded: Option<Vec<bool>>,
    last_ts: f64,
    dirty: bool,
    pending_inspect: Option<usize>,
    view: View,
    /// Skip animations: every spring jumps straight to its target.
    reduced: bool,
    /// Floor rectangles of the house view (ground floor first), animated.
    bands: Vec<BandAnim>,
    /// Topic tiles of the cards view, animated like the cards.
    tiles: Vec<BandAnim>,
    /// The pointer is over the ⓘ button.
    info_hover: bool,
    /// Cards view on a narrow screen: level headers sit on top of the cards.
    stacked: bool,
    /// Height of the page's top bar (CSS px), kept clear when fitting.
    top_inset: f64,
    /// Bounding box of the house view.
    bbox: (f64, f64, f64, f64),
    /// Label size multiplier (browser text size × the page's text-size control).
    text_scale: f64,
    /// Wall-clock time in ms (`Date.now()`), updated by input and frames.
    now: f64,
    /// Recent drag positions `(ms, x, y)` for the release velocity.
    hist: Vec<(f64, f64, f64)>,
    /// Momentum after a fling, in world units per second.
    glide: Option<(f64, f64)>,
    /// Check, once the camera rests, whether the map has been left off screen.
    settle_check: bool,
    /// Unclamped zoom the user asked for, and when / where they last zoomed.
    zoom_raw: f64,
    zoom_input_at: f64,
    zoom_point: (f64, f64),
    /// Last input came from a touch screen (larger hit areas).
    touch: bool,
    /// Node under the finger / mouse button while it is held down.
    pressed: Option<usize>,
    /// Bumped on every relayout so the page can refresh its keyboard tree.
    version: u32,
}

/// A floor rectangle `[x, y, w, h]` easing towards its target.
struct BandAnim {
    node: usize,
    cur: [f64; 4],
    vel: [f64; 4],
    tgt: [f64; 4],
}

fn level_color(level: &str) -> &'static str {
    match level.as_bytes().first() {
        Some(b'A') => "#5FD38D",
        Some(b'B') => "#F2B84B",
        _ => "#FF5C7A",
    }
}

/// A small chevron: `›` when closed, `⌄` when open.
fn draw_chevron(ctx: &CanvasRenderingContext2d, x: f64, y: f64, open: bool, colour: &str, hair: f64) {
    ctx.set_stroke_style_str(colour);
    ctx.set_line_width(1.7 * hair);
    ctx.begin_path();
    if open {
        ctx.move_to(x - 5.0, y - 2.5);
        ctx.line_to(x, y + 2.5);
        ctx.line_to(x + 5.0, y - 2.5);
    } else {
        ctx.move_to(x - 2.5, y - 5.0);
        ctx.line_to(x + 2.5, y);
        ctx.line_to(x - 2.5, y + 5.0);
    }
    ctx.stroke();
}

fn rounded_rect(ctx: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.arc_to(x + w, y, x + w, y + h, r).ok();
    ctx.arc_to(x + w, y + h, x, y + h, r).ok();
    ctx.arc_to(x, y + h, x, y, r).ok();
    ctx.arc_to(x, y, x + w, y, r).ok();
    ctx.close_path();
}

impl State {
    fn measure_labels(&mut self) {
        for n in self.nodes.iter_mut() {
            self.ctx.set_font(&n.font(self.text_scale));
            let text = n.label();
            n.label_w = self
                .ctx
                .measure_text(&text)
                .map(|m| m.width())
                .unwrap_or(text.chars().count() as f64 * 7.5);
        }
    }

    /// Recompute targets for the current expand/collapse state.
    fn relayout(&mut self) {
        let ln: Vec<LayoutNode> = self
            .nodes
            .iter()
            .map(|n| LayoutNode { children: n.children.clone(), expanded: n.expanded, extent: n.extent() })
            .collect();
        let placed = match self.view {
            View::Radial => layout::layout(&ln, 0),
            View::Levels => {
                let widths: Vec<f64> = self.nodes.iter().map(|n| n.label_w).collect();
                let avail = if self.w > 0.0 { self.w - 32.0 } else { 1280.0 };
                let house = layout::layout_floors(&ln, &widths, 0, avail);
                self.stacked = house.stacked;
                let old = std::mem::take(&mut self.bands);
                self.bands = house
                    .bands
                    .iter()
                    .map(|b| {
                        let tgt = [b.x, b.y, b.w, b.h];
                        let cur = old.iter().find(|o| o.node == b.node).map_or(tgt, |o| o.cur);
                        BandAnim { node: b.node, cur, vel: [0.0; 4], tgt }
                    })
                    .collect();
                let old_tiles = std::mem::take(&mut self.tiles);
                let mut tiles: Vec<BandAnim> = house
                    .tiles
                    .iter()
                    .map(|t| {
                        let tgt = [t.x, t.y, t.w, t.h];
                        let cur = old_tiles.iter().find(|o| o.node == t.node).map_or(tgt, |o| o.cur);
                        BandAnim { node: t.node, cur, vel: [0.0; 4], tgt }
                    })
                    .collect();
                // Tiles that disappear (their level folded) stay in place while they fade.
                for o in old_tiles {
                    if !tiles.iter().any(|t| t.node == o.node) && self.nodes[o.node].appear > 0.01 {
                        tiles.push(BandAnim { tgt: o.cur, vel: [0.0; 4], ..o });
                    }
                }
                self.tiles = tiles;
                self.bbox = house.bbox;
                house.placed
            }
        };
        self.version = self.version.wrapping_add(1);
        // Parents always precede their children in the arena, so a parent's
        // position/target is final by the time its children are processed.
        for i in 0..self.nodes.len() {
            let p = placed[i];
            let par = self.nodes[i].parent.map(|pi| {
                let q = &self.nodes[pi];
                (q.x, q.y, q.tx, q.ty)
            });
            let n = &mut self.nodes[i];
            if p.visible {
                if n.tappear < 0.5 && n.appear < 0.02 {
                    // Newly revealed: grow out of the parent.
                    if let Some((px, py, _, _)) = par {
                        n.x = px;
                        n.y = py;
                        n.vx = 0.0;
                        n.vy = 0.0;
                    }
                }
                n.tx = p.x;
                n.ty = p.y;
                n.tappear = 1.0;
            } else {
                // Hidden: retreat into the nearest visible ancestor.
                if let Some((_, _, ptx, pty)) = par {
                    n.tx = ptx;
                    n.ty = pty;
                }
                n.tappear = 0.0;
            }
        }
        self.dirty = true;
    }

    /// Jump every animated value (nodes, floors, camera) to its target.
    fn snap(&mut self) {
        for n in self.nodes.iter_mut() {
            n.x = n.tx;
            n.y = n.ty;
            n.vx = 0.0;
            n.vy = 0.0;
            n.appear = n.tappear;
            n.va = 0.0;
        }
        for b in self.bands.iter_mut().chain(self.tiles.iter_mut()) {
            b.cur = b.tgt;
            b.vel = [0.0; 4];
        }
        self.zoom = self.tzoom;
        self.vz = 0.0;
        if let Some((sx, sy, wx, wy)) = self.anchor.take() {
            self.cx = wx - (sx - self.w / 2.0) / self.zoom;
            self.cy = wy - (sy - self.h / 2.0) / self.zoom;
            self.tcx = self.cx;
            self.tcy = self.cy;
        } else {
            self.cx = self.tcx;
            self.cy = self.tcy;
        }
        self.vcx = 0.0;
        self.vcy = 0.0;
        self.dirty = true;
    }

    fn update(&mut self, dt: f64) -> bool {
        if self.reduced {
            if self.dirty {
                self.snap();
            }
            return false;
        }
        let mut active = false;
        for n in self.nodes.iter_mut() {
            active |= spring(&mut n.x, &mut n.vx, n.tx, 130.0, dt, 0.05);
            active |= spring(&mut n.y, &mut n.vy, n.ty, 130.0, dt, 0.05);
            active |= spring(&mut n.appear, &mut n.va, n.tappear, 170.0, dt, 0.002);
            n.appear = n.appear.clamp(0.0, 1.0);
        }
        for b in self.bands.iter_mut().chain(self.tiles.iter_mut()) {
            for k in 0..4 {
                active |= spring(&mut b.cur[k], &mut b.vel[k], b.tgt[k], 130.0, dt, 0.05);
            }
        }
        // Momentum after a fling: decays like scrolling, never resisted.
        if let Some((gx, gy)) = self.glide {
            self.cx += gx * dt;
            self.cy += gy * dt;
            self.tcx = self.cx;
            self.tcy = self.cy;
            self.vcx = 0.0;
            self.vcy = 0.0;
            let f = DECELERATION.powf(dt * 1000.0);
            let (gx, gy) = (gx * f, gy * f);
            if gx.hypot(gy) * self.zoom < 8.0 {
                self.glide = None;
                self.settle_check = true;
            } else {
                self.glide = Some((gx, gy));
            }
            active = true;
        } else if self.settle_check && self.pointers.is_empty() {
            self.settle_check = false;
            if let Some((tx, ty)) = self.content_return() {
                self.tcx = tx;
                self.tcy = ty;
                active = true;
            }
        }
        // Zoom pushed past its limits settles back once the user stops zooming.
        if (self.tzoom > MAX_ZOOM || self.tzoom < MIN_ZOOM) && self.pointers.len() < 2 && self.now - self.zoom_input_at > 180.0 {
            let (sx, sy) = self.zoom_point;
            let (wx, wy) = self.to_world(sx, sy);
            self.tzoom = self.tzoom.clamp(MIN_ZOOM, MAX_ZOOM);
            self.zoom_raw = self.tzoom;
            self.anchor = Some((sx, sy, wx, wy));
            active = true;
        }
        let mut lz = self.zoom.ln();
        let zooming = spring(&mut lz, &mut self.vz, self.tzoom.ln(), 220.0, dt, 0.0005);
        self.zoom = lz.exp();
        active |= zooming;
        if let Some((sx, sy, wx, wy)) = self.anchor {
            self.cx = wx - (sx - self.w / 2.0) / self.zoom;
            self.cy = wy - (sy - self.h / 2.0) / self.zoom;
            self.tcx = self.cx;
            self.tcy = self.cy;
            self.vcx = 0.0;
            self.vcy = 0.0;
            if !zooming {
                self.anchor = None;
            }
        } else {
            active |= spring(&mut self.cx, &mut self.vcx, self.tcx, 200.0, dt, 0.02);
            active |= spring(&mut self.cy, &mut self.vcy, self.tcy, 200.0, dt, 0.02);
        }
        active
    }

    /// Bounding box of the visible content in world units.
    fn content_box(&self) -> (f64, f64, f64, f64) {
        if self.view == View::Levels {
            return self.bbox;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (-ROOT_R, -ROOT_R, ROOT_R, ROOT_R);
        for n in self.nodes.iter().filter(|n| n.tappear > 0.5) {
            let e = n.extent();
            x0 = x0.min(n.tx - e);
            y0 = y0.min(n.ty - e);
            x1 = x1.max(n.tx + e);
            y1 = y1.max(n.ty + e);
        }
        (x0, y0, x1, y1)
    }

    /// Camera position that brings the map back into view, if it was left
    /// entirely off screen.
    fn content_return(&self) -> Option<(f64, f64)> {
        let (hw, hh) = (self.w / 2.0 / self.zoom, self.h / 2.0 / self.zoom);
        return_target(self.content_box(), self.cx, self.cy, hw, hh, hw.min(hh) * 0.35)
    }

    fn frame(&mut self, ts: f64) {
        let dt = if self.last_ts == 0.0 { 0.016 } else { ((ts - self.last_ts) / 1000.0).clamp(0.0, 0.033) };
        self.last_ts = ts;
        let active = self.update(dt);
        if active || self.dirty {
            self.dirty = false;
            self.render();
        }
    }

    fn to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        (self.cx + (sx - self.w / 2.0) / self.zoom, self.cy + (sy - self.h / 2.0) / self.zoom)
    }

    /// Topmost node whose dot or label contains the screen point.
    fn hit(&self, sx: f64, sy: f64) -> Option<usize> {
        let (wx, wy) = self.to_world(sx, sy);
        if self.view == View::Levels {
            return self.hit_levels(wx, wy);
        }
        let slop = if self.touch { 22.0 / self.zoom } else { 4.0 / self.zoom.min(1.0) };
        let band = if self.touch { (22.0 / self.zoom).max(11.0) } else { 11.0 };
        let mut label_hit = None;
        for (i, n) in self.nodes.iter().enumerate().rev() {
            if n.appear < 0.5 {
                continue;
            }
            let (dx, dy) = (wx - n.x, wy - n.y);
            let r = n.radius();
            if dx.hypot(dy) <= r + slop {
                return Some(i);
            }
            if n.depth == 0 || label_hit.is_some() {
                continue;
            }
            let (lx, ly, reach) = match self.view {
                View::Radial => {
                    let a = n.y.atan2(n.x);
                    (dx * a.cos() + dy * a.sin(), -dx * a.sin() + dy * a.cos(), r + n.label_w + 16.0)
                }
                // The cards view returned early via `hit_levels`.
                View::Levels => (dx, dy, 0.0),
            };
            if lx >= 0.0 && lx <= reach && ly.abs() <= band {
                label_hit = Some(i);
            }
        }
        label_hit
    }

    fn zoom_at(&mut self, sx: f64, sy: f64, factor: f64, immediate: bool) {
        let (wx, wy) = self.to_world(sx, sy);
        let base = if self.tzoom > MAX_ZOOM || self.tzoom < MIN_ZOOM { self.zoom_raw } else { self.tzoom };
        self.zoom_raw = (base * factor).clamp(MIN_ZOOM / 8.0, MAX_ZOOM * 8.0);
        self.tzoom = if self.reduced { self.zoom_raw.clamp(MIN_ZOOM, MAX_ZOOM) } else { soft_zoom(self.zoom_raw) };
        self.zoom_input_at = self.now;
        self.zoom_point = (sx, sy);
        self.glide = None;
        self.auto_fit = false;
        if immediate {
            self.zoom = self.tzoom;
            self.vz = 0.0;
            self.cx = wx - (sx - self.w / 2.0) / self.zoom;
            self.cy = wy - (sy - self.h / 2.0) / self.zoom;
            self.tcx = self.cx;
            self.tcy = self.cy;
            self.anchor = None;
        } else {
            self.anchor = Some((sx, sy, wx, wy));
        }
        self.dirty = true;
    }

    /// Centre on the root and zoom so every visible node fits.
    fn fit(&mut self, snap: bool) {
        if self.w <= 0.0 || self.h <= 0.0 {
            return;
        }
        let (zoom, cx, cy) = match self.view {
            View::Radial => {
                let reach = self
                    .nodes
                    .iter()
                    .filter(|n| n.tappear > 0.5)
                    .map(|n| n.tx.hypot(n.ty) + n.extent())
                    .fold(ROOT_R, f64::max)
                    + 30.0;
                let avail = (self.w.min(self.h - 60.0) / 2.0).max(60.0);
                (avail / reach, 0.0, 0.0)
            }
            View::Levels => {
                // A page: fit the width, start at the top just under the HUD.
                let (x0, y0, x1, _) = self.bbox;
                let zoom = ((self.w - 32.0).max(80.0) / (x1 - x0).max(1.0)).clamp(MIN_ZOOM, 1.15);
                (zoom, (x0 + x1) / 2.0, y0 + (self.h / 2.0 - self.top_inset) / zoom)
            }
        };
        self.tzoom = zoom.clamp(MIN_ZOOM, 1.25);
        self.tcx = cx;
        self.tcy = cy;
        self.anchor = None;
        self.auto_fit = true;
        if snap {
            self.zoom = self.tzoom;
            self.cx = self.tcx;
            self.cy = self.tcy;
            self.vz = 0.0;
            self.vcx = 0.0;
            self.vcy = 0.0;
        }
        self.dirty = true;
    }

    fn resize(&mut self, w: f64, h: f64, dpr: f64) {
        let first = self.w == 0.0;
        self.w = w;
        self.h = h;
        self.dpr = dpr.max(1.0);
        self.canvas.set_width((w * self.dpr).round().max(1.0) as u32);
        self.canvas.set_height((h * self.dpr).round().max(1.0) as u32);
        if self.view == View::Levels {
            self.relayout();
            if first {
                self.snap();
            }
        }
        if self.auto_fit {
            self.fit(first);
        }
        self.dirty = true;
    }

    fn pointer_down(&mut self, id: i32, x: f64, y: f64) {
        self.pointers.retain(|p| p.0 != id);
        self.pointers.push((id, x, y));
        // Grabbing the map stops any glide or return on the spot.
        self.glide = None;
        self.settle_check = false;
        self.tcx = self.cx;
        self.tcy = self.cy;
        self.vcx = 0.0;
        self.vcy = 0.0;
        self.anchor = None;
        if self.pointers.len() == 1 {
            self.press = (x, y);
            self.last = (x, y);
            self.dragged = false;
            self.hist.clear();
            self.hist.push((self.now, x, y));
            self.pressed = if self.info_hit(x, y).is_some() { None } else { self.hit(x, y) };
            self.dirty = true;
        } else if self.pointers.len() == 2 {
            let (a, b) = (self.pointers[0], self.pointers[1]);
            self.pinch = (a.1 - b.1).hypot(a.2 - b.2).max(1.0);
            self.last = ((a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0);
            self.dragged = true;
            self.pressed = None;
        }
    }

    fn pan_by(&mut self, dx: f64, dy: f64) {
        self.cx -= dx / self.zoom;
        self.cy -= dy / self.zoom;
        self.tcx = self.cx;
        self.tcy = self.cy;
        self.vcx = 0.0;
        self.vcy = 0.0;
        self.anchor = None;
        self.auto_fit = false;
        self.tzoom = self.zoom;
        self.vz = 0.0;
        self.dirty = true;
    }

    /// Returns `true` when the pointer is over a node (for the cursor style).
    fn pointer_move(&mut self, id: i32, x: f64, y: f64) -> bool {
        if let Some(p) = self.pointers.iter_mut().find(|p| p.0 == id) {
            p.1 = x;
            p.2 = y;
        }
        match self.pointers.len() {
            0 => {
                let on_info = self.info_hit(x, y).is_some();
                if on_info != self.info_hover {
                    self.info_hover = on_info;
                    self.dirty = true;
                }
                let h = self.info_hit(x, y).or_else(|| self.hit(x, y));
                if h != self.hover {
                    self.hover = h;
                    self.dirty = true;
                }
                h.is_some()
            }
            1 => {
                let threshold = if self.touch { 10.0 } else { 4.0 };
                if !self.dragged && (x - self.press.0).hypot(y - self.press.1) > threshold {
                    self.dragged = true;
                    if self.pressed.take().is_some() {
                        self.dirty = true;
                    }
                }
                if self.dragged {
                    self.pan_by(x - self.last.0, y - self.last.1);
                }
                self.last = (x, y);
                self.hist.push((self.now, x, y));
                let now = self.now;
                self.hist.retain(|h| now - h.0 <= 120.0);
                false
            }
            _ => {
                let (a, b) = (self.pointers[0], self.pointers[1]);
                let dist = (a.1 - b.1).hypot(a.2 - b.2).max(1.0);
                let mid = ((a.1 + b.1) / 2.0, (a.2 + b.2) / 2.0);
                self.pan_by(mid.0 - self.last.0, mid.1 - self.last.1);
                self.zoom_at(mid.0, mid.1, dist / self.pinch, true);
                self.pinch = dist;
                self.last = mid;
                false
            }
        }
    }

    fn pointer_up(&mut self, id: i32, x: f64, y: f64) {
        let had = self.pointers.len();
        self.pointers.retain(|p| p.0 != id);
        if self.pressed.take().is_some() {
            self.dirty = true;
        }
        if had == 1 && self.pointers.is_empty() && !self.dragged {
            self.click(x, y);
        } else if had == 1 && self.pointers.is_empty() {
            // Hand the finger's speed to the camera, which glides on and slows
            // down like a scroll view.
            let (vx, vy) = release_velocity(&self.hist, self.now);
            if !self.reduced && vx.hypot(vy) > 60.0 {
                self.glide = Some((-vx / self.zoom, -vy / self.zoom));
            } else {
                self.settle_check = true;
            }
        }
        if let Some(p) = self.pointers.first() {
            self.last = (p.1, p.2);
        }
    }

    fn click(&mut self, x: f64, y: f64) {
        if let Some(i) = self.info_hit(x, y) {
            self.selected = Some(i);
            self.pending_inspect = Some(i);
            self.dirty = true;
            return;
        }
        match self.hit(x, y) {
            Some(i) => {
                self.selected = Some(i);
                if self.nodes[i].children.is_empty() {
                    self.pending_inspect = Some(i);
                } else {
                    self.nodes[i].expanded = !self.nodes[i].expanded;
                    self.relayout();
                }
            }
            None => self.selected = None,
        }
        self.dirty = true;
    }

    /// Double-click is deliberately not used: it would delay or undo single
    /// clicks. Branch details open from the ⓘ button or the `I` key.
    fn dblclick(&mut self, _x: f64, _y: f64) {}

    /// The branch whose ⓘ button is shown: the hovered branch, else the
    /// selected one.
    fn info_node(&self) -> Option<usize> {
        let z = self.zoom;
        [self.hover, self.selected].into_iter().flatten().find(|&i| {
            let n = &self.nodes[i];
            i > 0 && !n.children.is_empty() && n.appear > 0.5 && n.font_px() * self.text_scale * z >= MIN_LABEL_PX
                && (self.view == View::Radial || n.depth <= 2)
        })
    }

    /// World position of the ⓘ button of node `i` (just past its label).
    fn info_pos(&self, i: usize) -> (f64, f64) {
        let n = &self.nodes[i];
        match self.view {
            View::Radial => {
                // Just past the end of the label pill, on its centre line.
                let a = n.y.atan2(n.x);
                let d = n.radius() + 4.0 + n.label_w + 12.0 + 14.0 / self.zoom;
                (n.x + a.cos() * d, n.y + a.sin() * d)
            }
            View::Levels => {
                if n.depth == 1 {
                    if let Some(b) = self.bands.iter().find(|b| b.node == i) {
                        let [x, y, w, h] = b.cur;
                        return if n.expanded && !self.stacked && h >= 110.0 { (x + layout::BADGE_W - 26.0, y + 40.0) } else { (x + w - 52.0, y + h.min(layout::FOLDED_H) / 2.0) };
                    }
                } else if let Some(t) = self.tiles.iter().find(|t| t.node == i) {
                    let [x, y, w, _] = t.cur;
                    return (x + w - 44.0, y + layout::TILE_H / 2.0);
                }
                (n.x, n.y)
            }
        }
    }

    /// The node whose ⓘ button is under the screen point.
    fn info_hit(&self, sx: f64, sy: f64) -> Option<usize> {
        let i = self.info_node()?;
        let (wx, wy) = self.info_pos(i);
        let px = (self.w / 2.0 + (wx - self.cx) * self.zoom, self.h / 2.0 + (wy - self.cy) * self.zoom);
        let reach = if self.touch { 22.0 } else { 11.0 };
        ((sx - px.0).hypot(sy - px.1) <= reach).then_some(i)
    }

    /// Screen position of the ⓘ button, if one is shown (for tests).
    fn info_screen(&self) -> Option<(f64, f64)> {
        let i = self.info_node()?;
        let (wx, wy) = self.info_pos(i);
        Some((self.w / 2.0 + (wx - self.cx) * self.zoom, self.h / 2.0 + (wy - self.cy) * self.zoom))
    }

    /// Outlined ⓘ (drawn, not typed, so it is centred at every zoom).
    fn draw_info(&self) {
        let Some(i) = self.info_node() else { return };
        let ctx = &self.ctx;
        let z = self.zoom;
        let (x, y) = self.info_pos(i);
        let u = 1.0 / z; // one screen pixel in world units
        let colour = if self.info_hover { "#FF4D6D" } else { "rgba(255,255,255,0.62)" };
        ctx.set_global_alpha(1.0);
        if self.info_hover {
            ctx.set_shadow_color("rgba(220,20,60,0.55)");
            ctx.set_shadow_blur(10.0);
        }
        ctx.begin_path();
        ctx.arc(x, y, 8.0 * u, 0.0, TAU).ok();
        ctx.set_fill_style_str("rgba(13,13,18,0.9)");
        ctx.fill();
        ctx.set_stroke_style_str(colour);
        ctx.set_line_width(1.5 * u);
        ctx.stroke();
        ctx.set_shadow_blur(0.0);
        // The "i": a round dot and a rounded stem.
        ctx.set_fill_style_str(colour);
        ctx.begin_path();
        ctx.arc(x, y - 3.4 * u, 1.15 * u, 0.0, TAU).ok();
        ctx.fill();
        ctx.set_line_cap("round");
        ctx.set_line_width(1.7 * u);
        ctx.begin_path();
        ctx.move_to(x, y - 0.6 * u);
        ctx.line_to(x, y + 3.9 * u);
        ctx.stroke();
    }

    fn key(&mut self, key: &str) -> bool {
        let step = 90.0 / self.zoom;
        let (cx, cy) = (self.w / 2.0, self.h / 2.0);
        match key {
            "ArrowLeft" => self.tcx -= step,
            "ArrowRight" => self.tcx += step,
            "ArrowUp" => self.tcy -= step,
            "ArrowDown" => self.tcy += step,
            "+" | "=" => self.zoom_at(cx, cy, 1.3, false),
            "-" | "_" => self.zoom_at(cx, cy, 1.0 / 1.3, false),
            "0" | "Home" => self.fit(false),
            "e" | "E" => self.expand_all(),
            "i" | "I" => match self.selected.or(self.hover) {
                Some(i) => self.pending_inspect = Some(i),
                None => return false,
            },
            "c" | "C" => self.collapse_all(),
            _ => return false,
        }
        if key.starts_with("Arrow") {
            self.auto_fit = false;
            self.anchor = None;
        }
        self.dirty = true;
        true
    }

    fn expand_all(&mut self) {
        for n in self.nodes.iter_mut() {
            n.expanded = !n.children.is_empty();
        }
        self.relayout();
        self.fit(false);
        if self.view == View::Radial {
            // Fitting every rule on screen would make all labels unreadable:
            // stay at a zoom where the first ring can be read, and let deeper
            // labels appear as the user zooms in.
            self.tzoom = self.tzoom.max(MIN_LABEL_PX / (15.0 * self.text_scale) * 1.05);
            self.auto_fit = false;
        }
    }

    fn collapse_all(&mut self) {
        // The house keeps its floors open and closes the rooms.
        let keep = if self.view == View::Levels { 1 } else { 0 };
        for n in self.nodes.iter_mut() {
            n.expanded = n.depth <= keep;
        }
        self.relayout();
        self.fit(false);
    }

    fn search(&mut self, query: &str) -> u32 {
        let q = normalize(query.trim());
        if q.is_empty() {
            if let Some(saved) = self.saved_expanded.take() {
                for (n, e) in self.nodes.iter_mut().zip(saved) {
                    n.expanded = e;
                }
            }
            for n in self.nodes.iter_mut() {
                n.matched = false;
                n.on_match_path = false;
            }
            self.searching = false;
            self.relayout();
            self.fit(false);
            return 0;
        }
        if self.saved_expanded.is_none() {
            self.saved_expanded = Some(self.nodes.iter().map(|n| n.expanded).collect());
        }
        self.searching = true;
        let mut hits = Vec::new();
        for (i, n) in self.nodes.iter_mut().enumerate() {
            n.matched = n.depth > 0 && fuzzy(&n.hay, &q);
            n.on_match_path = false;
            n.expanded = n.depth == 0;
            if n.matched {
                hits.push(i);
            }
        }
        for &i in &hits {
            let mut cur = self.nodes[i].parent;
            while let Some(p) = cur {
                self.nodes[p].expanded = true;
                self.nodes[p].on_match_path = true;
                cur = self.nodes[p].parent;
            }
        }
        self.relayout();
        self.fit(false);
        hits.len() as u32
    }

    /// Reveal a node, centre the camera on it and open it in the inspector.
    fn focus(&mut self, id: usize) {
        if id >= self.nodes.len() {
            return;
        }
        let mut cur = self.nodes[id].parent;
        while let Some(p) = cur {
            self.nodes[p].expanded = true;
            cur = self.nodes[p].parent;
        }
        if !self.nodes[id].children.is_empty() {
            self.nodes[id].expanded = true;
        }
        self.relayout();
        self.selected = Some(id);
        self.tcx = self.nodes[id].tx;
        self.tcy = self.nodes[id].ty;
        self.anchor = None;
        self.tzoom = self.tzoom.max(0.8);
        self.auto_fit = false;
        self.pending_inspect = Some(id);
        self.dirty = true;
    }

    fn node_json(&self, id: usize) -> String {
        let Some(n) = self.nodes.get(id) else { return "null".into() };
        let crumb = |i: usize| Crumb { id: i, tr: self.nodes[i].def.tr, en: self.nodes[i].def.en };
        let mut path = Vec::new();
        let mut cur = n.parent;
        while let Some(p) = cur {
            path.push(crumb(p));
            cur = self.nodes[p].parent;
        }
        path.reverse();
        let info = Inspect {
            id,
            tr: n.def.tr,
            en: n.def.en,
            level: n.def.level,
            formula: n.def.formula,
            harmony: n.def.harmony,
            notes: n.def.notes,
            leaf: n.children.is_empty(),
            examples: &n.def.examples,
            path,
            children: n.children.iter().map(|&c| crumb(c)).collect(),
        };
        serde_json::to_string(&info).unwrap_or_else(|_| "null".into())
    }

    // ───────────────────────────── rendering ─────────────────────────────

    /// Font string scaled by the user's text size.
    fn ui_font(&self, weight: u32, px: f64) -> String {
        format!("{weight} {}px Inter, system-ui, sans-serif", px * self.text_scale)
    }

    /// Draws the "study by level" page: title, one card per level, topic
    /// tiles and rule rows (camera transform already applied).
    fn render_house(&self) {
        let ctx = &self.ctx;
        let z = self.zoom;
        let hair = 1.0 / z.max(0.5);
        let mut on_path = vec![false; self.nodes.len()];
        let mut cur = self.hover.or(self.selected);
        while let Some(i) = cur {
            on_path[i] = true;
            cur = self.nodes[i].parent;
        }
        ctx.set_line_join("round");
        ctx.set_line_cap("round");
        ctx.set_shadow_blur(0.0);
        ctx.set_text_baseline("middle");
        ctx.set_text_align("left");

        // ── page title ──
        let root = &self.nodes[0];
        let total = root.leaf_count.max(1) as f64;
        if let Some(first) = self.bands.first() {
            let x = first.cur[0];
            ctx.set_global_alpha(1.0);
            ctx.set_fill_style_str("#F2F2F7");
            ctx.set_font(&self.ui_font(700, 24.0));
            ctx.fill_text(root.def.en, x, 30.0).ok();
            ctx.set_fill_style_str("#9A9AAE");
            ctx.set_font(&self.ui_font(500, 13.0));
            let sub = format!("A1 → C2  ·  {} rules  ·  {} levels", root.leaf_count, root.children.len());
            ctx.fill_text(&sub, x, 58.0).ok();
        }

        // ── level cards ──
        for b in &self.bands {
            let n = &self.nodes[b.node];
            let [x, y, w, h] = b.cur;
            let alpha = n.appear * self.dim(n);
            let hot = on_path[b.node] || (self.searching && (n.matched || n.on_match_path));
            let folded = !n.expanded;
            ctx.set_global_alpha(alpha);
            rounded_rect(ctx, x, y, w, h, 14.0);
            ctx.set_fill_style_str(if self.pressed == Some(b.node) { "rgba(255,255,255,0.06)" } else { "rgba(255,255,255,0.032)" });
            ctx.fill();
            ctx.set_stroke_style_str(if hot { "rgba(220,20,60,0.75)" } else { "rgba(255,255,255,0.085)" });
            ctx.set_line_width(if hot { 1.6 * hair } else { hair });
            ctx.stroke();

            let colour = level_color(n.def.level);
            let mut names = n.def.en.split(" · ");
            let (name_tr, name_en) = (names.next().unwrap_or(""), names.next().unwrap_or(""));
            let topics = n.children.len();
            let summary = format!("{} rules  ·  {} topics", n.leaf_count, topics);
            if self.stacked && !folded {
                ctx.set_stroke_style_str("rgba(255,255,255,0.07)");
                ctx.set_line_width(hair);
                ctx.begin_path();
                ctx.move_to(x + 16.0, y + layout::FOLDED_H);
                ctx.line_to(x + w - 16.0, y + layout::FOLDED_H);
                ctx.stroke();
            }
            if folded || self.stacked || h < 110.0 {
                // One row: code, name, summary, chevron.
                let cy = y + h.min(layout::FOLDED_H) / 2.0;
                ctx.set_fill_style_str(colour);
                ctx.begin_path();
                ctx.arc(x + 22.0, cy, 4.0, 0.0, TAU).ok();
                ctx.fill();
                ctx.set_fill_style_str("#F2F2F7");
                ctx.set_font(&self.ui_font(700, 22.0));
                ctx.fill_text(n.def.tr, x + 36.0, cy).ok();
                ctx.set_font(&self.ui_font(600, 14.0));
                ctx.fill_text(name_en, x + 84.0, cy).ok();
                let ew = ctx.measure_text(name_en).map(|m| m.width()).unwrap_or(0.0);
                ctx.set_fill_style_str("#8A8AA0");
                ctx.set_font(&self.ui_font(500, 13.0));
                let rest = if self.stacked { format!("{} rules", n.leaf_count) } else { format!("{name_tr}   ·   {summary}") };
                // Leave room for the ⓘ and the chevron; drop the summary if it would collide.
                let rw = ctx.measure_text(&rest).map(|m| m.width()).unwrap_or(0.0);
                let reserve = if self.info_node() == Some(b.node) { 66.0 } else { 44.0 };
                if x + 84.0 + ew + 14.0 + rw <= x + w - reserve {
                    ctx.fill_text(&rest, x + 84.0 + ew + 14.0, cy).ok();
                }
                draw_chevron(ctx, x + w - 26.0, cy, !folded, "rgba(255,255,255,0.5)", hair);
                continue;
            }
            // Badge column.
            ctx.set_fill_style_str("#F2F2F7");
            ctx.set_font(&self.ui_font(700, 30.0));
            ctx.fill_text(n.def.tr, x + 24.0, y + 40.0).ok();
            ctx.set_font(&self.ui_font(600, 14.0));
            ctx.fill_text(name_en, x + 24.0, y + 72.0).ok();
            ctx.set_fill_style_str("#9A9AAE");
            ctx.set_font(&self.ui_font(500, 12.5));
            ctx.fill_text(name_tr, x + 24.0, y + 92.0).ok();
            ctx.set_fill_style_str("#8A8AA0");
            ctx.fill_text(&summary, x + 24.0, y + 112.0).ok();
            let bar_w = layout::BADGE_W - 48.0;
            rounded_rect(ctx, x + 24.0, y + 126.0, bar_w, 3.0, 1.5);
            ctx.set_fill_style_str("rgba(255,255,255,0.08)");
            ctx.fill();
            rounded_rect(ctx, x + 24.0, y + 126.0, (bar_w * n.leaf_count as f64 / total * 3.0).clamp(6.0, bar_w), 3.0, 1.5);
            ctx.set_fill_style_str(colour);
            ctx.fill();
            // Hairline between the badge and the topics.
            ctx.set_stroke_style_str("rgba(255,255,255,0.07)");
            ctx.set_line_width(hair);
            ctx.begin_path();
            ctx.move_to(x + layout::BADGE_W, y + 18.0);
            ctx.line_to(x + layout::BADGE_W, y + h - 18.0);
            ctx.stroke();
        }

        // ── topic tiles ──
        for t in &self.tiles {
            let n = &self.nodes[t.node];
            if n.appear < 0.01 {
                continue;
            }
            let [x, y, w, h] = t.cur;
            let hovered = self.hover == Some(t.node);
            let selected = self.selected == Some(t.node);
            let hot = n.matched || (self.searching && n.on_match_path);
            ctx.set_global_alpha(n.appear * self.dim(n));
            rounded_rect(ctx, x, y, w, h, 10.0);
            ctx.set_fill_style_str(if self.pressed == Some(t.node) {
                "rgba(220,20,60,0.14)"
            } else if hovered {
                "rgba(255,255,255,0.075)"
            } else if n.expanded {
                "rgba(255,255,255,0.055)"
            } else {
                "rgba(255,255,255,0.04)"
            });
            ctx.fill();
            ctx.set_stroke_style_str(if hot || selected {
                "rgba(220,20,60,0.75)"
            } else if hovered {
                "rgba(220,20,60,0.45)"
            } else {
                "rgba(255,255,255,0.08)"
            });
            ctx.set_line_width(hair);
            ctx.stroke();

            let cy = y + layout::TILE_H / 2.0;
            ctx.set_fill_style_str(if n.matched { "#FF5C7A" } else { "#F2F2F7" });
            ctx.set_font(&n.font(self.text_scale));
            ctx.fill_text(n.def.tr, x + 16.0, cy).ok();
            // Rule count pill, then (ⓘ when shown) and the chevron.
            let count = n.leaf_count.to_string();
            ctx.set_font(&format!("500 {}px 'JetBrains Mono', ui-monospace, monospace", 11.5 * self.text_scale));
            let cw = ctx.measure_text(&count).map(|m| m.width()).unwrap_or(14.0) + 14.0;
            let px = x + w - 62.0 - cw;
            rounded_rect(ctx, px, cy - 10.0, cw, 20.0, 10.0);
            ctx.set_fill_style_str("rgba(255,255,255,0.06)");
            ctx.fill();
            ctx.set_fill_style_str("#9A9AAE");
            ctx.set_text_align("center");
            ctx.fill_text(&count, px + cw / 2.0, cy + 0.5).ok();
            ctx.set_text_align("left");
            draw_chevron(ctx, x + w - 20.0, cy, n.expanded, if hovered { "#FFFFFF" } else { "rgba(255,255,255,0.5)" }, hair);
            if n.expanded && h > layout::TILE_H + 1.0 {
                ctx.set_stroke_style_str("rgba(255,255,255,0.06)");
                ctx.set_line_width(hair);
                ctx.begin_path();
                ctx.move_to(x + 16.0, y + layout::TILE_H);
                ctx.line_to(x + w - 16.0, y + layout::TILE_H);
                ctx.stroke();
            }
        }

        // ── rules inside open topics ──
        let readable = |n: &Node| n.font_px() * self.text_scale * z >= MIN_LABEL_PX;
        for (i, n) in self.nodes.iter().enumerate() {
            if n.depth != 3 || n.appear < 0.02 {
                continue;
            }
            let hovered = self.hover == Some(i);
            let selected = self.selected == Some(i);
            ctx.set_global_alpha(n.appear * self.dim(n));
            if hovered || selected || self.pressed == Some(i) {
                rounded_rect(ctx, n.x - 8.0, n.y - layout::RULE_ROW / 2.0 + 2.0, layout::BULLET_W + n.label_w + 16.0, layout::RULE_ROW - 4.0, 6.0);
                ctx.set_fill_style_str(if self.pressed == Some(i) { "rgba(220,20,60,0.18)" } else { "rgba(255,255,255,0.06)" });
                ctx.fill();
            }
            ctx.set_fill_style_str(if n.matched { "#FF2D55" } else { "rgba(255,255,255,0.35)" });
            ctx.begin_path();
            ctx.arc(n.x + 3.0, n.y, 2.5, 0.0, TAU).ok();
            ctx.fill();
            if !readable(n) {
                continue;
            }
            ctx.set_font(&n.font(self.text_scale));
            ctx.set_fill_style_str(if n.matched {
                "#FF5C7A"
            } else if hovered || selected {
                "#FFFFFF"
            } else {
                "#C8C8D6"
            });
            ctx.fill_text(n.def.tr, n.x + layout::BULLET_W, n.y + 0.5).ok();
        }

        self.draw_info();
    }

    /// Rule, topic tile or level card under a world point (cards view).
    fn hit_levels(&self, wx: f64, wy: f64) -> Option<usize> {
        let pad = if self.touch { 10.0 / self.zoom } else { 2.0 };
        for (i, n) in self.nodes.iter().enumerate() {
            if n.depth == 3 && n.appear > 0.5 {
                let x0 = n.x - 8.0 - pad;
                let x1 = n.x + layout::BULLET_W + n.label_w + 8.0 + pad;
                if wx >= x0 && wx <= x1 && (wy - n.y).abs() <= layout::RULE_ROW / 2.0 + pad * 0.5 {
                    return Some(i);
                }
            }
        }
        let inside = |c: &[f64; 4], dy: f64| wx >= c[0] - pad && wx <= c[0] + c[2] + pad && wy >= c[1] - pad && wy <= c[1] + dy + pad;
        for t in self.tiles.iter().rev() {
            if self.nodes[t.node].appear > 0.5 && inside(&t.cur, layout::TILE_H) {
                return Some(t.node);
            }
        }
        for b in &self.bands {
            let folded = !self.nodes[b.node].expanded;
            let [x, y, w, h] = b.cur;
            let area = if folded || self.stacked { [x, y, w, h.min(layout::FOLDED_H)] } else { [x, y, layout::BADGE_W, h] };
            if inside(&area, area[3]) {
                return Some(b.node);
            }
        }
        None
    }

    fn dim(&self, n: &Node) -> f64 {
        if self.searching && !(n.matched || n.on_match_path || n.depth == 0) {
            0.2
        } else {
            1.0
        }
    }

    fn draw_grid(&self) {
        let ctx = &self.ctx;
        let z = self.zoom;
        let mut step = 60.0;
        while step * z < 26.0 {
            step *= 2.0;
        }
        let half = self.w.hypot(self.h) / 2.0 / z;
        let d = self.cx.hypot(self.cy);
        let (rmin, rmax) = ((d - half).max(0.0), d + half);
        // Only the arc that can intersect the viewport.
        let (a0, da) = if d > half { (self.cy.atan2(self.cx), (half / d).asin() + 0.02) } else { (0.0, PI) };
        let s = 1.7 / z;
        let (hw, hh) = (self.w / 2.0 / z + s, self.h / 2.0 / z + s);
        ctx.set_global_alpha(1.0);
        ctx.set_fill_style_str("rgba(255,255,255,0.11)");
        let mut k = (rmin / step).ceil().max(1.0);
        while k * step <= rmax {
            let r = k * step;
            let count = (TAU * k).round().max(6.0);
            let dtheta = TAU / count;
            let j0 = ((a0 - da) / dtheta).floor() as i64;
            let j1 = ((a0 + da) / dtheta).ceil() as i64;
            for j in j0..=j1.min(j0 + count as i64 - 1) {
                let a = j as f64 * dtheta;
                let (x, y) = (r * a.cos(), r * a.sin());
                if (x - self.cx).abs() <= hw && (y - self.cy).abs() <= hh {
                    ctx.fill_rect(x - s / 2.0, y - s / 2.0, s, s);
                }
            }
            k += 1.0;
        }
    }

    fn render(&self) {
        let ctx = &self.ctx;
        let z = self.zoom;
        ctx.set_transform(self.dpr, 0.0, 0.0, self.dpr, 0.0, 0.0).ok();
        ctx.set_global_alpha(1.0);
        ctx.set_shadow_blur(0.0);
        ctx.set_fill_style_str(BG);
        ctx.fill_rect(0.0, 0.0, self.w, self.h);
        ctx.translate(self.w / 2.0, self.h / 2.0).ok();
        ctx.scale(z, z).ok();
        ctx.translate(-self.cx, -self.cy).ok();

        self.draw_grid();

        if self.view == View::Levels {
            self.render_house();
            return;
        }

        // Active branch: root → hovered (or selected) node.
        let mut on_path = vec![false; self.nodes.len()];
        let mut cur = self.hover.or(self.selected);
        while let Some(i) = cur {
            on_path[i] = true;
            cur = self.nodes[i].parent;
        }

        // Edges, normal ones first so highlighted ones sit on top.
        ctx.set_line_cap("round");
        for pass in 0..2 {
            for (i, n) in self.nodes.iter().enumerate() {
                let Some(pi) = n.parent else { continue };
                if n.appear < 0.01 {
                    continue;
                }
                let hot = on_path[i] || (self.searching && (n.matched || n.on_match_path));
                if hot != (pass == 1) {
                    continue;
                }
                let p = &self.nodes[pi];
                let rc = n.x.hypot(n.y);
                let ca = n.y.atan2(n.x);
                let (sx, sy, rp, pa) = if p.depth == 0 {
                    (ROOT_R * ca.cos(), ROOT_R * ca.sin(), ROOT_R, ca)
                } else {
                    (p.x, p.y, p.x.hypot(p.y), p.y.atan2(p.x))
                };
                let m = (rp + rc) / 2.0;
                let (width, alpha) = match n.depth {
                    1 => (3.2, 0.7),
                    2 => (2.1, 0.5),
                    3 => (1.4, 0.38),
                    _ => (1.0, 0.3),
                };
                ctx.begin_path();
                ctx.move_to(sx, sy);
                ctx.bezier_curve_to(m * pa.cos(), m * pa.sin(), m * ca.cos(), m * ca.sin(), n.x, n.y);
                if hot {
                    ctx.set_global_alpha(0.95 * n.appear);
                    ctx.set_stroke_style_str(CRIMSON);
                    ctx.set_line_width(width * 1.3);
                    ctx.set_shadow_color(CRIMSON);
                    ctx.set_shadow_blur(12.0);
                } else {
                    ctx.set_global_alpha(alpha * n.appear * self.dim(n));
                    ctx.set_stroke_style_str("#8A8AA8");
                    ctx.set_line_width(width);
                    ctx.set_shadow_blur(0.0);
                }
                ctx.stroke();
            }
        }
        ctx.set_shadow_blur(0.0);

        let readable = |n: &Node| n.font_px() * self.text_scale * z >= MIN_LABEL_PX;
        for (i, n) in self.nodes.iter().enumerate().skip(1) {
            if n.appear < 0.01 {
                continue;
            }
            let alpha = n.appear * self.dim(n);
            let r = n.radius() * (0.4 + 0.6 * n.appear);
            let hovered = self.hover == Some(i);
            let selected = self.selected == Some(i);
            let branch = !n.children.is_empty();

            ctx.set_global_alpha(alpha);
            if hovered || selected || n.matched {
                ctx.set_shadow_color(CRIMSON);
                ctx.set_shadow_blur(if hovered { 22.0 } else { 14.0 });
            }
            ctx.begin_path();
            ctx.arc(n.x, n.y, r, 0.0, TAU).ok();
            if n.matched {
                ctx.set_fill_style_str("#FF2D55");
            } else if branch {
                ctx.set_fill_style_str(if n.expanded { "#16161F" } else { DEEP_RED });
            } else {
                ctx.set_fill_style_str(if hovered || selected { "#FFFFFF" } else { "#B9B9C9" });
            }
            ctx.fill();
            ctx.set_shadow_blur(0.0);
            if self.pressed == Some(i) {
                ctx.set_stroke_style_str("#FFFFFF");
                ctx.set_line_width(2.0 / z.min(1.0));
                ctx.begin_path();
                ctx.arc(n.x, n.y, r + 3.0 / z.min(1.0), 0.0, TAU).ok();
                ctx.stroke();
                ctx.begin_path();
                ctx.arc(n.x, n.y, r, 0.0, TAU).ok();
            }
            if branch {
                ctx.set_stroke_style_str(CRIMSON);
                ctx.set_line_width(if n.depth == 1 { 2.2 } else { 1.5 });
                ctx.stroke();
            }

            if !readable(n) || n.appear < 0.25 {
                continue;
            }
            let ang = if n.x.hypot(n.y) < 1.0 { 0.0 } else { n.y.atan2(n.x) };
            let flip = ang.cos() < 0.0;
            ctx.save();
            ctx.translate(n.x, n.y).ok();
            ctx.rotate(if flip { ang + PI } else { ang }).ok();
            let x0 = n.radius() + 4.0;
            let pill_w = n.label_w + 12.0;
            if branch {
                // Dark pill so the child edges pass underneath the label.
                let left = if flip { -(x0 + pill_w) } else { x0 };
                ctx.set_fill_style_str("rgba(13,13,18,0.86)");
                ctx.begin_path();
                ctx.arc(left + 10.0, 0.0, 10.0, PI / 2.0, PI * 1.5).ok();
                ctx.arc(left + pill_w - 10.0, 0.0, 10.0, -PI / 2.0, PI / 2.0).ok();
                ctx.close_path();
                ctx.fill();
            }
            ctx.set_font(&n.font(self.text_scale));
            ctx.set_text_baseline("middle");
            ctx.set_text_align(if flip { "right" } else { "left" });
            ctx.set_fill_style_str(if n.matched {
                "#FF5C7A"
            } else if hovered || selected {
                "#FFFFFF"
            } else if n.depth == 1 {
                "#F2F2F7"
            } else {
                "#C8C8D6"
            });
            let tx = x0 + 6.0;
            ctx.fill_text(n.def.tr, if flip { -tx } else { tx }, 0.5).ok();
            ctx.restore();
        }

        self.draw_info();

        // Central core.
        let root = &self.nodes[0];
        ctx.set_global_alpha(1.0);
        ctx.set_shadow_color("rgba(220,20,60,0.6)");
        ctx.set_shadow_blur(if self.hover == Some(0) { 60.0 } else { 40.0 });
        ctx.begin_path();
        ctx.arc(root.x, root.y, ROOT_R, 0.0, TAU).ok();
        match ctx.create_radial_gradient(root.x - 18.0, root.y - 22.0, 6.0, root.x, root.y, ROOT_R) {
            Ok(g) => {
                g.add_color_stop(0.0, "#E8284A").ok();
                g.add_color_stop(0.55, DEEP_RED).ok();
                g.add_color_stop(1.0, "#4A0710").ok();
                ctx.set_fill_style_canvas_gradient(&g);
            }
            Err(_) => ctx.set_fill_style_str(DEEP_RED),
        }
        ctx.fill();
        ctx.set_shadow_blur(0.0);
        ctx.set_global_alpha(0.55);
        ctx.set_stroke_style_str(CRIMSON);
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.arc(root.x, root.y, ROOT_R + 7.0, 0.0, TAU).ok();
        ctx.stroke();
        ctx.set_global_alpha(1.0);
        if 14.0 * z >= 3.5 {
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_fill_style_str("#FFFFFF");
            ctx.set_font(FONT_ROOT);
            ctx.fill_text(root.def.tr, root.x, root.y - 7.0).ok();
            ctx.set_fill_style_str("rgba(255,255,255,0.78)");
            ctx.set_font(FONT_ROOT_SUB);
            ctx.fill_text(&format!("({})", root.def.en), root.x, root.y + 11.0).ok();
        }
    }
}

type RafClosure = Closure<dyn FnMut(f64)>;

/// Handle exported to JavaScript.
#[wasm_bindgen]
pub struct App {
    inner: Rc<RefCell<State>>,
    on_inspect: js_sys::Function,
    raf: Rc<RefCell<Option<RafClosure>>>,
}

#[wasm_bindgen]
impl App {
    /// `on_inspect(json)` is called whenever a rule should open in the drawer.
    #[wasm_bindgen(constructor)]
    /// `view` is `"levels"` for the house-of-floors view, anything else for
    /// the radial map.
    pub fn new(canvas: HtmlCanvasElement, on_inspect: js_sys::Function, view: &str) -> Result<App, JsValue> {
        let view = if view == "levels" { View::Levels } else { View::Radial };
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D canvas context unavailable"))?
            .dyn_into::<CanvasRenderingContext2d>()?;
        let mut nodes = Vec::new();
        let tree = match view {
            View::Radial => grammar_data::grammar(),
            View::Levels => grammar_data::by_level(),
        };
        flatten(tree, None, 0, view, &mut nodes);
        let mut state = State {
            canvas,
            ctx,
            nodes,
            w: 0.0,
            h: 0.0,
            dpr: 1.0,
            cx: 0.0,
            cy: 0.0,
            zoom: 1.0,
            vcx: 0.0,
            vcy: 0.0,
            vz: 0.0,
            tcx: 0.0,
            tcy: 0.0,
            tzoom: 1.0,
            anchor: None,
            pointers: Vec::new(),
            press: (0.0, 0.0),
            last: (0.0, 0.0),
            dragged: false,
            pinch: 1.0,
            auto_fit: true,
            hover: None,
            selected: None,
            searching: false,
            saved_expanded: None,
            last_ts: 0.0,
            dirty: true,
            pending_inspect: None,
            view,
            reduced: false,
            bands: Vec::new(),
            tiles: Vec::new(),
            info_hover: false,
            stacked: false,
            top_inset: 76.0,
            bbox: (0.0, 0.0, 0.0, 0.0),
            text_scale: 1.0,
            now: 0.0,
            hist: Vec::new(),
            glide: None,
            settle_check: false,
            zoom_raw: 1.0,
            zoom_input_at: 0.0,
            zoom_point: (0.0, 0.0),
            touch: false,
            pressed: None,
            version: 0,
        };
        state.measure_labels();
        state.relayout();
        if view == View::Levels {
            // The house is shown already built; only later changes animate.
            state.snap();
        }
        Ok(App { inner: Rc::new(RefCell::new(state)), on_inspect, raf: Rc::new(RefCell::new(None)) })
    }

    /// Start the `requestAnimationFrame` loop (idempotent).
    pub fn start(&self) -> Result<(), JsValue> {
        if self.raf.borrow().is_some() {
            return Ok(());
        }
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let inner = self.inner.clone();
        let slot = self.raf.clone();
        let win = window.clone();
        *self.raf.borrow_mut() = Some(Closure::wrap(Box::new(move |ts: f64| {
            {
                let mut s = inner.borrow_mut();
                s.now = js_sys::Date::now();
                s.frame(ts);
            }
            if let Some(cb) = slot.borrow().as_ref() {
                let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
            }
        }) as Box<dyn FnMut(f64)>));
        let slot = self.raf.borrow();
        window.request_animation_frame(slot.as_ref().unwrap().as_ref().unchecked_ref())?;
        Ok(())
    }

    /// Deliver a queued inspector request once no borrow is held.
    fn flush(&self) {
        let pending = self.inner.borrow_mut().pending_inspect.take();
        if let Some(id) = pending {
            let json = self.inner.borrow().node_json(id);
            let _ = self.on_inspect.call1(&JsValue::NULL, &JsValue::from_str(&json));
        }
    }

    pub fn resize(&self, width: f64, height: f64, dpr: f64) {
        self.inner.borrow_mut().resize(width, height, dpr);
    }

    pub fn pointer_down(&self, id: i32, x: f64, y: f64) {
        let mut s = self.inner.borrow_mut();
        s.now = js_sys::Date::now();
        s.pointer_down(id, x, y);
    }

    /// Whether the latest input came from a touch screen (larger hit areas).
    pub fn set_touch(&self, touch: bool) {
        self.inner.borrow_mut().touch = touch;
    }

    pub fn pointer_move(&self, id: i32, x: f64, y: f64) -> bool {
        let mut s = self.inner.borrow_mut();
        s.now = js_sys::Date::now();
        s.pointer_move(id, x, y)
    }

    pub fn pointer_up(&self, id: i32, x: f64, y: f64) {
        {
            let mut s = self.inner.borrow_mut();
            s.now = js_sys::Date::now();
            s.pointer_up(id, x, y);
        }
        self.flush();
    }

    pub fn pointer_cancel(&self, id: i32) {
        let mut s = self.inner.borrow_mut();
        s.pointers.retain(|p| p.0 != id);
        s.dragged = true;
    }

    pub fn pointer_leave(&self) {
        let mut s = self.inner.borrow_mut();
        if s.hover.take().is_some() {
            s.dirty = true;
        }
    }

    /// Wheel with both axes: on the cards page it scrolls (Ctrl zooms);
    /// on the radial map it zooms at the cursor.
    pub fn wheel_ex(&self, x: f64, y: f64, delta_x: f64, delta_y: f64, ctrl: bool) {
        let mut s = self.inner.borrow_mut();
        s.now = js_sys::Date::now();
        if s.view == View::Levels && !ctrl {
            let z = s.zoom;
            s.glide = None;
            s.anchor = None;
            s.auto_fit = false;
            s.tcx += delta_x / z;
            s.tcy += delta_y / z;
            s.settle_check = true;
            s.dirty = true;
        } else {
            let boost = if ctrl { 4.0 } else { 1.0 };
            let factor = (-delta_y * boost * 0.0016).exp().clamp(0.5, 2.0);
            s.zoom_at(x, y, factor, false);
        }
    }

    /// Height of the page's top bar, kept clear when the view is fitted.
    pub fn set_top_inset(&self, px: f64) {
        let mut s = self.inner.borrow_mut();
        s.top_inset = px.max(0.0);
        if s.auto_fit {
            s.fit(false);
        }
    }

    /// Screen position of the ⓘ button, if shown (testing aid).
    pub fn info_screen_pos(&self) -> Vec<f64> {
        self.inner.borrow().info_screen().map_or(Vec::new(), |(x, y)| vec![x, y])
    }

    pub fn wheel(&self, x: f64, y: f64, delta_y: f64) {
        let factor = (-delta_y * 0.0016).exp().clamp(0.5, 2.0);
        let mut s = self.inner.borrow_mut();
        s.now = js_sys::Date::now();
        s.zoom_at(x, y, factor, false);
    }

    pub fn dblclick(&self, x: f64, y: f64) {
        self.inner.borrow_mut().dblclick(x, y);
        self.flush();
    }

    /// Returns `true` if the key was handled.
    pub fn key(&self, key: &str) -> bool {
        let handled = self.inner.borrow_mut().key(key);
        self.flush();
        handled
    }

    /// Multiplies every map label size (1 = normal). Re-lays out the map.
    pub fn set_text_scale(&self, scale: f64) {
        let mut s = self.inner.borrow_mut();
        s.text_scale = scale.clamp(0.8, 1.8);
        s.measure_labels();
        s.relayout();
        s.dirty = true;
    }

    /// Changes whenever the visible tree changes (for the keyboard tree).
    pub fn layout_version(&self) -> u32 {
        self.inner.borrow().version
    }

    /// Visible nodes in reading order, as JSON
    /// `[{id, parent, depth, tr, en, branch, expanded, count}]`.
    pub fn tree_json(&self) -> String {
        let s = self.inner.borrow();
        let mut out = Vec::new();
        fn walk(s: &State, i: usize, out: &mut Vec<serde_json::Value>) {
            let n = &s.nodes[i];
            out.push(serde_json::json!({
                "id": i,
                "parent": n.parent,
                "depth": n.depth,
                "tr": n.label(),
                "en": n.def.en,
                "branch": !n.children.is_empty(),
                "expanded": n.expanded,
                "count": n.leaf_count,
            }));
            if n.expanded {
                for &c in &n.children {
                    walk(s, c, out);
                }
            }
        }
        walk(&s, 0, &mut out);
        serde_json::Value::Array(out).to_string()
    }

    /// Opens or closes a branch.
    pub fn set_expanded(&self, id: usize, expanded: bool) {
        let mut s = self.inner.borrow_mut();
        if id < s.nodes.len() && !s.nodes[id].children.is_empty() && s.nodes[id].expanded != expanded {
            s.nodes[id].expanded = expanded;
            s.relayout();
            s.dirty = true;
        }
    }

    /// Highlights a node and brings it into view (keyboard navigation).
    pub fn select(&self, id: usize) {
        let mut s = self.inner.borrow_mut();
        if id >= s.nodes.len() {
            return;
        }
        s.selected = Some(id);
        let (x, y) = (s.nodes[id].tx, s.nodes[id].ty);
        let (hw, hh) = (s.w / 2.0 / s.zoom, s.h / 2.0 / s.zoom);
        if (x - s.cx).abs() > hw * 0.7 || (y - s.cy).abs() > hh * 0.7 {
            s.tcx = x;
            s.tcy = y;
            s.glide = None;
            s.anchor = None;
        }
        s.auto_fit = false;
        s.dirty = true;
    }

    pub fn expand_all(&self) {
        self.inner.borrow_mut().expand_all();
    }

    pub fn collapse_all(&self) {
        self.inner.borrow_mut().collapse_all();
    }

    pub fn reset_camera(&self) {
        self.inner.borrow_mut().fit(false);
    }

    /// Apply a search query; returns the number of matching rules.
    pub fn search(&self, query: &str) -> u32 {
        self.inner.borrow_mut().search(query)
    }

    pub fn focus(&self, id: usize) {
        self.inner.borrow_mut().focus(id);
        self.flush();
    }

    pub fn clear_selection(&self) {
        let mut s = self.inner.borrow_mut();
        s.selected = None;
        s.dirty = true;
    }

    pub fn node_json(&self, id: usize) -> String {
        self.inner.borrow().node_json(id)
    }

    /// Id of the first rule whose Turkish title is exactly `title`, or -1.
    pub fn rule_id_by_title(&self, title: &str) -> i32 {
        let s = self.inner.borrow();
        s.nodes.iter().position(|n| n.def.tr == title).map_or(-1, |i| i as i32)
    }

    /// Turn animations off (`true`) or on. With animations off every change
    /// is applied on the next frame without easing.
    pub fn set_reduced_motion(&self, reduced: bool) {
        let mut s = self.inner.borrow_mut();
        s.reduced = reduced;
        s.dirty = true;
    }

    pub fn node_count(&self) -> usize {
        self.inner.borrow().nodes.len()
    }
}

/// Frequency dictionary lookup. `kind` is "verbs" or "adjectives"; returns
/// JSON `{kind, total, items}` with the matching entries, most common first.
/// Turkish-English dictionary for the `/player/` page, built from the TSV the
/// page fetches (`player/tr_en_dictionary.tsv`).
#[wasm_bindgen]
pub struct PlayerDictionary {
    inner: player::lookup::Dictionary,
}

#[wasm_bindgen]
impl PlayerDictionary {
    #[wasm_bindgen(constructor)]
    pub fn new(tsv: &str) -> PlayerDictionary {
        PlayerDictionary { inner: player::lookup::Dictionary::from_tsv(tsv) }
    }

    /// Number of entries loaded.
    pub fn size(&self) -> usize {
        self.inner.len()
    }

    /// JSON `{word, translation, note}` for a clicked word, or `null`.
    pub fn lookup(&self, word: &str) -> String {
        self.inner.lookup_json(word)
    }
}

/// Parses a Turkish and an (optional) English subtitle file and pairs them.
/// Returns JSON `{ok, error, cues: [{id, start, end, tr, en}], warnings}`.
#[wasm_bindgen]
pub fn parse_dual_srt(turkish: &str, english: &str) -> String {
    player::srt::parse_dual_json(turkish, english)
}

#[wasm_bindgen]
pub fn dict_search(kind: &str, query: &str) -> String {
    dictionary::search(kind, query)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> Vec<char> {
        normalize(s)
    }

    #[test]
    fn normalize_folds_turkish_letters() {
        assert_eq!(n("Fıstıkçı ŞAHAP İĞÖÜ").iter().collect::<String>(), "fistikci sahap igou");
    }

    #[test]
    fn fuzzy_matches_substrings_and_near_misses() {
        let hay = n("Ünsüz Benzeşmesi Consonant Assimilation (Fıstıkçı Şahap)");
        assert!(fuzzy(&hay, &n("fıstıkçı")));
        assert!(fuzzy(&hay, &n("fistikci")));
        assert!(fuzzy(&hay, &n("asimilation"))); // one letter missing
        assert!(!fuzzy(&hay, &n("zzz")));
        assert!(!fuzzy(&hay, &n("")));
        assert!(fuzzy(&n("-(I)yor Present Continuous V + (I)yor + person"), &n("yor")));
    }

    #[test]
    fn suffix_search_ignores_optional_parts() {
        let mut nodes = Vec::new();
        flatten(grammar_data::grammar(), None, 0, View::Radial, &mut nodes);
        let hit = |q: &str| nodes.iter().filter(|x| fuzzy(&x.hay, &n(q))).map(|x| x.def.tr).collect::<Vec<_>>();
        assert!(hit("-yor").contains(&"-(I)yor"));
        assert!(hit("-acak").iter().any(|t| *t == "-(y)AcAk"));
        assert!(hit("Fıstıkçı").contains(&"Ünsüz Benzeşmesi"));
        assert!(hit("genitive").contains(&"İlgi Hâli -(n)In"));
    }

    #[test]
    fn house_tree_lays_out_as_six_floors() {
        let mut nodes = Vec::new();
        flatten(grammar_data::by_level(), None, 0, View::Levels, &mut nodes);
        assert_eq!(nodes[0].children.len(), 6);
        assert_eq!(nodes[0].leaf_count, {
            let mut radial = Vec::new();
            flatten(grammar_data::grammar(), None, 0, View::Radial, &mut radial);
            radial[0].leaf_count
        });
        for node in nodes.iter_mut() {
            node.label_w = node.label().chars().count() as f64 * 7.5;
            node.expanded = !node.children.is_empty();
        }
        let ln: Vec<LayoutNode> = nodes
            .iter()
            .map(|n| LayoutNode { children: n.children.clone(), expanded: n.expanded, extent: 0.0 })
            .collect();
        let widths: Vec<f64> = nodes.iter().map(|n| n.label_w).collect();
        let house = layout::layout_floors(&ln, &widths, 0, 1600.0);
        assert!(house.placed.iter().all(|p| p.visible));
        assert_eq!(house.bands.len(), 6);
        // A1 is the ground floor; every higher level sits above the one before.
        let floors = &nodes[0].children;
        assert_eq!(nodes[floors[0]].def.tr, "A1");
        assert_eq!(nodes[floors[5]].def.tr, "C2");
        for w in house.bands.windows(2) {
            assert!(w[0].y + w[0].h < w[1].y, "A1 first, each level below the previous one");
        }
    }

    #[test]
    fn fling_velocity_and_projection() {
        let hist = [(0.0, 0.0, 0.0), (16.0, 20.0, 0.0), (32.0, 40.0, 0.0), (48.0, 60.0, 0.0)];
        let (vx, vy) = release_velocity(&hist, 50.0);
        assert!((vx - 1250.0).abs() < 1.0 && vy == 0.0, "{vx}");
        // A finger that stopped before lifting does not fling.
        assert_eq!(release_velocity(&hist, 200.0), (0.0, 0.0));
        assert_eq!(release_velocity(&[], 0.0), (0.0, 0.0));
        // Apple's projection: 1000 px/s travels about 499 px.
        assert!((project(1000.0) - 499.0).abs() < 1.0);
        // The per-frame decay used for gliding travels the same distance.
        let (mut x, mut v) = (0.0, 1000.0);
        while v > 1.0 {
            x += v / 1000.0;
            v *= DECELERATION;
        }
        assert!((x - project(1000.0)).abs() < 5.0, "{x}");
    }

    #[test]
    fn return_only_when_content_is_off_screen() {
        let bbox = (-100.0, -100.0, 100.0, 100.0);
        // Content partly visible: no return, however far the user flew.
        assert_eq!(return_target(bbox, 150.0, 0.0, 100.0, 80.0, 20.0), None);
        // Content entirely to the left: come back just far enough.
        let (tx, ty) = return_target(bbox, 1000.0, 0.0, 100.0, 80.0, 20.0).unwrap();
        assert_eq!((tx, ty), (100.0 + 100.0 - 20.0, 0.0));
        // Off below and to the right.
        let (tx, ty) = return_target(bbox, -900.0, -700.0, 100.0, 80.0, 20.0).unwrap();
        assert_eq!((tx, ty), (-100.0 - 100.0 + 20.0, -100.0 - 80.0 + 20.0));
    }

    #[test]
    fn zoom_limits_are_soft() {
        assert_eq!(soft_zoom(1.0), 1.0);
        let over = soft_zoom(MAX_ZOOM * 2.0);
        assert!(over > MAX_ZOOM && over < MAX_ZOOM * 1.3, "{over}");
        assert!(soft_zoom(MAX_ZOOM * 100.0) < MAX_ZOOM * 1.29);
        let under = soft_zoom(MIN_ZOOM / 2.0);
        assert!(under < MIN_ZOOM && under > MIN_ZOOM / 1.3, "{under}");
    }

    #[test]
    fn spring_settles_on_target() {
        let (mut x, mut v) = (0.0, 0.0);
        let mut steps = 0;
        while spring(&mut x, &mut v, 100.0, 130.0, 1.0 / 60.0, 0.05) {
            steps += 1;
            assert!(steps < 600, "spring did not settle");
            assert!(x < 101.0, "spring overshot: {x}");
        }
        assert_eq!(x, 100.0);
    }

    #[test]
    fn real_tree_lays_out_without_overlap() {
        let mut nodes = Vec::new();
        flatten(grammar_data::grammar(), None, 0, View::Radial, &mut nodes);
        assert!(nodes.len() > 110);
        for node in nodes.iter_mut() {
            node.label_w = node.def.tr.chars().count() as f64 * 7.5;
            node.expanded = !node.children.is_empty();
        }
        let ln: Vec<LayoutNode> = nodes
            .iter()
            .map(|n| LayoutNode { children: n.children.clone(), expanded: n.expanded, extent: n.extent() })
            .collect();
        let placed = layout::layout(&ln, 0);
        assert!(placed.iter().all(|p| p.visible));
        // Same-ring neighbours are at least one line apart at the label start.
        for d in 1..=3 {
            let mut ring: Vec<&layout::Placed> = placed.iter().filter(|p| p.depth == d).collect();
            ring.sort_by(|a, b| a.angle.partial_cmp(&b.angle).unwrap());
            for w in ring.windows(2) {
                let dist = (w[0].x - w[1].x).hypot(w[0].y - w[1].y);
                assert!(dist >= layout::LINE_H * 0.99, "ring {d}: {dist}");
            }
        }
    }
}
