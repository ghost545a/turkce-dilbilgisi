//! Radial (polar) tree layout.
//!
//! Only *visible* nodes are placed: the root, plus the children of every
//! visible, expanded node. Each visible leaf of the current view gets an equal
//! angular slice; an inner node sits in the middle of the slices of its
//! subtree. Labels are drawn along the radius, so two things keep them apart:
//!
//! * ring `d` starts beyond the longest label of ring `d - 1` (radial spacing);
//! * ring `d` is pushed out until the chord between its two angularly closest
//!   nodes is at least one line of text (angular spacing).
//!
//! This module is pure maths (no DOM), so it is unit-tested on the host.

use std::f64::consts::{PI, TAU};

/// Minimum distance between two neighbouring label baselines, in world units.
pub const LINE_H: f64 = 24.0;
/// Free space between the end of one ring's labels and the next ring.
pub const RING_GAP: f64 = 46.0;

pub struct LayoutNode {
    pub children: Vec<usize>,
    pub expanded: bool,
    /// Radial length the node occupies beyond its centre (dot + label).
    pub extent: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Placed {
    pub visible: bool,
    pub depth: usize,
    pub angle: f64,
    pub radius: f64,
    pub x: f64,
    pub y: f64,
}

fn weigh(nodes: &[LayoutNode], i: usize, weight: &mut [f64]) -> f64 {
    let n = &nodes[i];
    let w = if n.expanded && !n.children.is_empty() {
        n.children.iter().map(|&c| weigh(nodes, c, weight)).sum()
    } else {
        1.0
    };
    weight[i] = w;
    w
}

fn assign(
    nodes: &[LayoutNode],
    weight: &[f64],
    i: usize,
    a0: f64,
    a1: f64,
    depth: usize,
    out: &mut [Placed],
) {
    out[i].visible = true;
    out[i].depth = depth;
    out[i].angle = (a0 + a1) / 2.0;
    let n = &nodes[i];
    if n.expanded && !n.children.is_empty() {
        let span = a1 - a0;
        let mut a = a0;
        for &c in &n.children {
            let s = span * weight[c] / weight[i];
            assign(nodes, weight, c, a, a + s, depth + 1, out);
            a += s;
        }
    }
}

/// Lay out the visible part of the tree rooted at `root`. The root is placed
/// at the origin; the first top-level branch starts at twelve o'clock.
pub fn layout(nodes: &[LayoutNode], root: usize) -> Vec<Placed> {
    let mut out = vec![Placed::default(); nodes.len()];
    if nodes.is_empty() {
        return out;
    }
    let mut weight = vec![0.0; nodes.len()];
    weigh(nodes, root, &mut weight);
    assign(nodes, &weight, root, -PI / 2.0, -PI / 2.0 + TAU, 0, &mut out);

    let max_depth = out.iter().filter(|p| p.visible).map(|p| p.depth).max().unwrap_or(0);
    let mut angles: Vec<Vec<f64>> = vec![Vec::new(); max_depth + 1];
    let mut max_extent = vec![0.0f64; max_depth + 1];
    for (i, p) in out.iter().enumerate() {
        if p.visible {
            angles[p.depth].push(p.angle);
            max_extent[p.depth] = max_extent[p.depth].max(nodes[i].extent);
        }
    }

    let mut radius = vec![0.0f64; max_depth + 1];
    for d in 1..=max_depth {
        let a = &mut angles[d];
        a.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let mut gap = TAU;
        if a.len() > 1 {
            gap = a[0] + TAU - a[a.len() - 1];
            for w in a.windows(2) {
                gap = gap.min(w[1] - w[0]);
            }
        }
        let angular = if gap >= PI { 0.0 } else { LINE_H / (2.0 * (gap / 2.0).sin()) };
        let radial = radius[d - 1] + max_extent[d - 1] + RING_GAP;
        radius[d] = radial.max(angular);
    }

    for p in out.iter_mut().filter(|p| p.visible) {
        p.radius = radius[p.depth];
        p.x = p.radius * p.angle.cos();
        p.y = p.radius * p.angle.sin();
    }
    out
}

// ───────────────────────── "Study by level" cards ─────────────────────────
//
// The tree is root → levels → topics → rules. Each level is a card, stacked
// top to bottom (A1 first). A card has a badge column on the left and a grid
// of equally wide topic tiles on the right. An open topic spans the whole row
// and lists its rules in aligned columns. A folded level shrinks to one row.

/// Width of the badge column on the left of each card.
pub const BADGE_W: f64 = 184.0;
/// Height of a topic tile's header row.
pub const TILE_H: f64 = 44.0;
/// Height of one rule row inside an open topic.
pub const RULE_ROW: f64 = 26.0;
/// Space a rule's bullet takes before its label.
pub const BULLET_W: f64 = 14.0;
/// Room reserved at the right of a tile header for count, ⓘ and chevron.
pub const TILE_TAIL: f64 = 92.0;
/// Height of a folded card.
pub const FOLDED_H: f64 = 64.0;
/// Height of the page title above the first card.
pub const HEADER_H: f64 = 86.0;
const BADGE_MIN_H: f64 = 140.0;
const TILE_GAP: f64 = 10.0;
const CARD_PAD: f64 = 20.0;
const CARD_GAP: f64 = 16.0;
const COLS: usize = 3;
const MAX_ROWS: usize = 8;
const RULE_COL_GAP: f64 = 28.0;
const MIN_TILE_W: f64 = 220.0;

/// An axis-aligned rectangle owned by a node (a card or a tile).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub node: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

pub struct Floors {
    pub placed: Vec<Placed>,
    /// Narrow screens: the level header sits on top of the card, not beside it.
    pub stacked: bool,
    /// One card per level, top to bottom.
    pub bands: Vec<Band>,
    /// One tile per visible topic.
    pub tiles: Vec<Band>,
    /// Bounding box of the page, header included: (min x, min y, max x, max y).
    pub bbox: (f64, f64, f64, f64),
}

/// Rows per column so that the rule columns fit `inner` width.
fn rule_columns(rules: &[usize], label_w: &[f64], inner: f64) -> (usize, Vec<f64>) {
    let mut rows = MAX_ROWS.min(rules.len().max(1));
    loop {
        let widths: Vec<f64> = rules
            .chunks(rows)
            .map(|col| BULLET_W + col.iter().map(|&c| label_w[c]).fold(0.0, f64::max))
            .collect();
        let total = widths.iter().sum::<f64>() + RULE_COL_GAP * widths.len().saturating_sub(1) as f64;
        if total <= inner || rows >= rules.len() {
            return (rows, widths);
        }
        rows += 1;
    }
}

/// Lays out the visible part of a root → levels → topics → rules tree as
/// cards. `label_w[i]` is the width of node `i`'s label; `avail` is the width
/// the page has (CSS px), which decides the number of tile columns and, below
/// 640 px, puts each level's header on top of its card.
pub fn layout_floors(nodes: &[LayoutNode], label_w: &[f64], root: usize, avail: f64) -> Floors {
    let mut placed = vec![Placed::default(); nodes.len()];
    let mut bands = Vec::new();
    let mut tiles = Vec::new();
    if nodes.is_empty() {
        return Floors { placed, stacked: false, bands, tiles, bbox: (0.0, 0.0, 0.0, 0.0) };
    }
    let floors: &[usize] = if nodes[root].expanded { &nodes[root].children } else { &[] };

    // Every tile has the same width: wide enough for the longest topic title.
    let longest = floors
        .iter()
        .flat_map(|&f| nodes[f].children.iter())
        .map(|&r| label_w[r])
        .fold(0.0, f64::max);
    let tile_w = (16.0 + longest + 12.0 + TILE_TAIL).max(MIN_TILE_W);
    let stacked = avail < 640.0;
    let side = if stacked { 0.0 } else { BADGE_W };
    let fit = ((avail - side - 2.0 * CARD_PAD + TILE_GAP) / (tile_w + TILE_GAP)).floor() as usize;
    let cols = fit.clamp(1, COLS);
    let inner = cols as f64 * tile_w + (cols - 1) as f64 * TILE_GAP;
    let card_w = side + CARD_PAD + inner + CARD_PAD;
    let left = -card_w / 2.0;
    let body_x = left + side + CARD_PAD;

    placed[root] = Placed { visible: true, depth: 0, angle: 0.0, radius: 0.0, x: left, y: 0.0 };
    let mut y = HEADER_H;
    for &f in floors {
        let top = y;
        placed[f] = Placed { visible: true, depth: 1, angle: 0.0, radius: 0.0, x: left + 24.0, y: top + 36.0 };
        let mut h = FOLDED_H;
        if nodes[f].expanded {
            let mut cy = top + if stacked { FOLDED_H } else { CARD_PAD };
            let mut col = 0usize;
            for &r in &nodes[f].children {
                let open = nodes[r].expanded && !nodes[r].children.is_empty();
                if open {
                    if col > 0 {
                        cy += TILE_H + TILE_GAP;
                        col = 0;
                    }
                    let (rows, widths) = rule_columns(&nodes[r].children, label_w, inner - 32.0);
                    let shown = nodes[r].children.len().min(rows);
                    let th = TILE_H + shown as f64 * RULE_ROW + 12.0;
                    tiles.push(Band { node: r, x: body_x, y: cy, w: inner, h: th });
                    placed[r] = Placed { visible: true, depth: 2, angle: 0.0, radius: 0.0, x: body_x + 16.0, y: cy + TILE_H / 2.0 };
                    let mut cx = body_x + 16.0;
                    for (ci, column) in nodes[r].children.chunks(rows).enumerate() {
                        for (ri, &c) in column.iter().enumerate() {
                            let ry = cy + TILE_H + 6.0 + ri as f64 * RULE_ROW + RULE_ROW / 2.0;
                            placed[c] = Placed { visible: true, depth: 3, angle: 0.0, radius: 0.0, x: cx, y: ry };
                        }
                        cx += widths[ci] + RULE_COL_GAP;
                    }
                    cy += th + TILE_GAP;
                } else {
                    let tx = body_x + col as f64 * (tile_w + TILE_GAP);
                    tiles.push(Band { node: r, x: tx, y: cy, w: tile_w, h: TILE_H });
                    placed[r] = Placed { visible: true, depth: 2, angle: 0.0, radius: 0.0, x: tx + 16.0, y: cy + TILE_H / 2.0 };
                    col += 1;
                    if col == cols {
                        col = 0;
                        cy += TILE_H + TILE_GAP;
                    }
                }
            }
            if col > 0 {
                cy += TILE_H + TILE_GAP;
            }
            let body_bottom = cy - TILE_GAP + CARD_PAD;
            h = if stacked { body_bottom - top } else { (body_bottom - top).max(BADGE_MIN_H) };
        }
        bands.push(Band { node: f, x: left, y: top, w: card_w, h });
        y = top + h + CARD_GAP;
    }
    let bottom = (y - CARD_GAP).max(HEADER_H);
    Floors { placed, stacked, bands, tiles, bbox: (left - 24.0, -24.0, -left + 24.0, bottom + 24.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_cards_stack_downwards_without_overlap() {
        let mut nodes = tree(&[6, 5, 11], 0.0);
        let floors = nodes[0].children.clone();
        // Open one topic on even levels, none on odd ones; fold the last level.
        for (k, &f) in floors.iter().enumerate() {
            for (j, &r) in nodes[f].children.clone().iter().enumerate() {
                nodes[r].expanded = k % 2 == 0 && j == 1;
            }
        }
        nodes[floors[5]].expanded = false;
        let widths: Vec<f64> = (0..nodes.len()).map(|i| 60.0 + (i % 7) as f64 * 25.0).collect();
        let out = layout_floors(&nodes, &widths, 0, 2000.0);

        assert_eq!(out.bands.len(), 6);
        assert_eq!(out.bands[0].node, floors[0], "first level is the first card");
        assert!(out.bands[0].y >= HEADER_H, "cards start below the page title");
        for w in out.bands.windows(2) {
            assert!(w[0].y + w[0].h < w[1].y, "each card ends before the next one starts");
            assert_eq!(w[0].w, w[1].w);
        }
        assert_eq!(out.bands[5].h, FOLDED_H, "a folded level is one row");
        for &r in &nodes[floors[5]].children {
            assert!(!out.placed[r].visible, "a folded level hides its topics");
        }

        // Tiles sit inside their card's body and never overlap.
        let card_of = |t: &Band| out.bands.iter().find(|b| nodes[b.node].children.contains(&t.node)).unwrap();
        for (i, a) in out.tiles.iter().enumerate() {
            let c = card_of(a);
            assert!(a.x >= c.x + BADGE_W && a.x + a.w <= c.x + c.w + 0.01, "tile outside its card horizontally");
            assert!(a.y >= c.y && a.y + a.h <= c.y + c.h + 0.01, "tile outside its card vertically");
            assert!(widths[a.node] + 28.0 + TILE_TAIL <= a.w + 0.01, "topic title does not fit its tile");
            for b in &out.tiles[i + 1..] {
                let apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(apart, "tiles overlap: {a:?} {b:?}");
            }
        }

        // Rules of an open topic sit inside its tile, below the header, and fit.
        check_rules(&nodes, &out, &widths);
    }

    #[test]
    fn narrow_screens_stack_the_level_header() {
        let mut nodes = tree(&[6, 5, 11], 0.0);
        let first = nodes[0].children[0];
        let topic = nodes[first].children[2];
        nodes[topic].expanded = true;
        for f in nodes[0].children.clone() {
            for r in nodes[f].children.clone() {
                if r != topic {
                    nodes[r].expanded = false;
                }
            }
        }
        let widths: Vec<f64> = (0..nodes.len()).map(|i| 40.0 + (i % 5) as f64 * 20.0).collect();
        let out = layout_floors(&nodes, &widths, 0, 360.0);
        assert!(out.stacked);
        let card = out.bands[0];
        assert!(card.w <= 360.0, "a card fits a phone screen: {}", card.w);
        for t in out.tiles.iter().filter(|t| nodes[card.node].children.contains(&t.node)) {
            assert!(t.y >= card.y + FOLDED_H, "tiles sit below the level header");
            assert!(t.x + t.w <= card.x + card.w + 0.01);
        }
        check_rules(&nodes, &out, &widths);
    }

    fn check_rules(nodes: &[LayoutNode], out: &Floors, widths: &[f64]) {
        for t in out.tiles.iter().filter(|t| nodes[t.node].expanded) {
            for &c in &nodes[t.node].children {
                let p = out.placed[c];
                assert!(p.visible);
                assert!(p.y - RULE_ROW / 2.0 >= t.y + TILE_H && p.y + RULE_ROW / 2.0 <= t.y + t.h + 0.01);
                assert!(p.x + BULLET_W + widths[c] <= t.x + t.w + 0.01, "rule label overflows its tile");
            }
        }
    }

    /// Build a tree with the given fan-out per level; returns nodes in
    /// pre-order, like the runtime arena.
    fn tree(fan: &[usize], extent: f64) -> Vec<LayoutNode> {
        fn add(fan: &[usize], extent: f64, out: &mut Vec<LayoutNode>) -> usize {
            let id = out.len();
            out.push(LayoutNode { children: vec![], expanded: true, extent });
            if let Some((&n, rest)) = fan.split_first() {
                for _ in 0..n {
                    let c = add(rest, extent, out);
                    out[id].children.push(c);
                }
            }
            id
        }
        let mut out = Vec::new();
        add(fan, extent, &mut out);
        out
    }

    /// Smallest distance between the label segments of any two visible,
    /// non-root nodes (each label is sampled along its radial segment).
    fn min_label_distance(nodes: &[LayoutNode], placed: &[Placed]) -> f64 {
        let seg = |i: usize| -> Vec<(f64, f64)> {
            let p = placed[i];
            (0..=12)
                .map(|k| {
                    let r = p.radius + nodes[i].extent * k as f64 / 12.0;
                    (r * p.angle.cos(), r * p.angle.sin())
                })
                .collect()
        };
        let vis: Vec<usize> = (0..nodes.len()).filter(|&i| placed[i].visible && placed[i].depth > 0).collect();
        let segs: Vec<Vec<(f64, f64)>> = vis.iter().map(|&i| seg(i)).collect();
        let mut best = f64::MAX;
        for a in 0..segs.len() {
            for b in (a + 1)..segs.len() {
                for p in &segs[a] {
                    for q in &segs[b] {
                        best = best.min((p.0 - q.0).hypot(p.1 - q.1));
                    }
                }
            }
        }
        best
    }

    #[test]
    fn root_is_at_origin() {
        let nodes = tree(&[6, 4, 5], 150.0);
        let placed = layout(&nodes, 0);
        assert!(placed[0].visible);
        assert_eq!((placed[0].x, placed[0].y), (0.0, 0.0));
    }

    #[test]
    fn fully_expanded_labels_do_not_overlap() {
        let nodes = tree(&[6, 4, 5], 180.0);
        let placed = layout(&nodes, 0);
        assert!(placed.iter().all(|p| p.visible));
        let d = min_label_distance(&nodes, &placed);
        assert!(d >= LINE_H * 0.95, "labels too close: {d}");
    }

    #[test]
    fn partially_expanded_labels_do_not_overlap() {
        let mut nodes = tree(&[6, 5, 7], 140.0);
        // Collapse every other top-level branch and a few inner ones.
        let top: Vec<usize> = nodes[0].children.clone();
        for (k, &c) in top.iter().enumerate() {
            if k % 2 == 0 {
                nodes[c].expanded = false;
            } else {
                let inner = nodes[c].children[0];
                nodes[inner].expanded = false;
            }
        }
        let placed = layout(&nodes, 0);
        let d = min_label_distance(&nodes, &placed);
        assert!(d >= LINE_H * 0.95, "labels too close: {d}");
    }

    #[test]
    fn collapsed_subtrees_are_hidden() {
        let mut nodes = tree(&[3, 2], 100.0);
        let first = nodes[0].children[0];
        nodes[first].expanded = false;
        let placed = layout(&nodes, 0);
        for &c in &nodes[first].children {
            assert!(!placed[c].visible);
        }
        assert!(placed[first].visible);
    }

    #[test]
    fn rings_grow_outwards() {
        let nodes = tree(&[6, 4, 5], 160.0);
        let placed = layout(&nodes, 0);
        let r = |d: usize| placed.iter().find(|p| p.depth == d).unwrap().radius;
        assert!(r(1) > 0.0 && r(2) >= r(1) + 160.0 && r(3) >= r(2) + 160.0);
    }
}
