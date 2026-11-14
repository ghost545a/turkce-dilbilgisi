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

// ───────────────────────── House-of-floors layout ─────────────────────────
//
// Used by the "study by level" view. The tree is root → floors → rooms →
// rules. Floors are horizontal bands stacked upwards (the first floor is the
// ground floor, so it has the largest y); inside a floor the rooms flow left
// to right and wrap; inside an open room the rules are listed in columns of
// at most `MAX_ROWS`. Labels are horizontal and start `DOT` to the right of
// the node, so a label's box is [x, x + DOT + width] × [y ± ROW / 2].

/// Vertical distance between two rules in a column.
pub const ROW: f64 = 24.0;
/// Horizontal space reserved for a node's dot before its label starts.
pub const DOT: f64 = 22.0;
/// Extra width reserved after a floor or room title for its rule count.
pub const COUNT_W: f64 = 64.0;
/// Height of the roof drawn above the top floor.
pub const ROOF_H: f64 = 170.0;
const MAX_ROWS: usize = 8;
const COL_GAP: f64 = 26.0;
const ROOM_GAP: f64 = 34.0;
const PAD: f64 = 20.0;
const TITLE_H: f64 = 54.0;
const HEAD_H: f64 = 30.0;
const FLOOR_GAP: f64 = 12.0;
const MAX_ROW_W: f64 = 1500.0;
const MIN_W: f64 = 760.0;

/// Rectangle of one floor, in world units.
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
    /// One band per visible floor, ground floor first.
    pub bands: Vec<Band>,
    /// Bounding box of the whole house: (min x, min y, max x, max y).
    pub bbox: (f64, f64, f64, f64),
}

/// Lay out the visible part of a root → floors → rooms → rules tree as a
/// house. `label_w[i]` is the width of node `i`'s label.
pub fn layout_floors(nodes: &[LayoutNode], label_w: &[f64], root: usize) -> Floors {
    let mut placed = vec![Placed::default(); nodes.len()];
    let mut bands = Vec::new();
    if nodes.is_empty() {
        return Floors { placed, bands, bbox: (0.0, 0.0, 0.0, 0.0) };
    }
    let floors: &[usize] = if nodes[root].expanded { &nodes[root].children } else { &[] };

    // Pass 1: lay out each floor in its own coordinates (origin = top-left).
    struct Local {
        w: f64,
        h: f64,
        items: Vec<(usize, f64, f64, usize)>, // node, x, y, depth
    }
    let mut locals = Vec::with_capacity(floors.len());
    for &f in floors {
        let mut items = vec![(f, PAD + 14.0, TITLE_H / 2.0, 1)];
        let mut w = PAD + 14.0 + DOT + label_w[f] + COUNT_W + PAD;
        let mut h = TITLE_H;
        if nodes[f].expanded && !nodes[f].children.is_empty() {
            let mut x = PAD;
            let mut y = TITLE_H;
            let mut row_h: f64 = 0.0;
            for &r in &nodes[f].children {
                let kids: &[usize] = if nodes[r].expanded { &nodes[r].children } else { &[] };
                let col_w: Vec<f64> = kids
                    .chunks(MAX_ROWS)
                    .map(|col| DOT + col.iter().map(|&c| label_w[c]).fold(0.0, f64::max))
                    .collect();
                let body_w = col_w.iter().sum::<f64>() + COL_GAP * col_w.len().saturating_sub(1) as f64;
                let room_w = (DOT + label_w[r] + COUNT_W).max(body_w);
                let rows = kids.len().min(MAX_ROWS);
                let room_h = HEAD_H + if rows > 0 { rows as f64 * ROW + 6.0 } else { 0.0 };
                if x > PAD && x + room_w > MAX_ROW_W {
                    x = PAD;
                    y += row_h + 12.0;
                    row_h = 0.0;
                }
                items.push((r, x + 8.0, y + HEAD_H / 2.0, 2));
                let mut cx = x;
                for (ci, col) in kids.chunks(MAX_ROWS).enumerate() {
                    for (ri, &c) in col.iter().enumerate() {
                        items.push((c, cx + 8.0, y + HEAD_H + ri as f64 * ROW + ROW / 2.0, 3));
                    }
                    cx += col_w[ci] + COL_GAP;
                }
                row_h = row_h.max(room_h);
                w = w.max(x + room_w + PAD);
                x += room_w + ROOM_GAP;
            }
            h = y + row_h + PAD;
        }
        locals.push(Local { w, h, items });
    }

    // Pass 2: stack the floors upwards from y = 0, all with the same width.
    let width = locals.iter().map(|l| l.w).fold(MIN_W, f64::max);
    let left = -width / 2.0;
    let mut bottom = 0.0;
    let mut top = 0.0;
    for (k, &f) in floors.iter().enumerate() {
        let l = &locals[k];
        top = bottom - l.h;
        bands.push(Band { node: f, x: left, y: top, w: width, h: l.h });
        for &(i, x, y, depth) in &l.items {
            placed[i] = Placed { visible: true, depth, angle: 0.0, radius: 0.0, x: left + x, y: top + y };
        }
        bottom = top - FLOOR_GAP;
    }
    placed[root] = Placed { visible: true, depth: 0, angle: 0.0, radius: 0.0, x: 0.0, y: top - ROOF_H * 0.42 };
    Floors { placed, bands, bbox: (left - 34.0, top - ROOF_H, -left + 34.0, 0.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_stack_upwards_without_overlap() {
        let mut nodes = tree(&[6, 5, 11], 0.0);
        // Open the rooms of every other floor; fold the last floor.
        let floors = nodes[0].children.clone();
        for (k, &f) in floors.iter().enumerate() {
            for &r in &nodes[f].children.clone() {
                nodes[r].expanded = k % 2 == 0;
            }
        }
        nodes[floors[5]].expanded = false;
        let widths: Vec<f64> = (0..nodes.len()).map(|i| 60.0 + (i % 7) as f64 * 25.0).collect();
        let out = layout_floors(&nodes, &widths, 0);

        assert_eq!(out.bands.len(), 6);
        for w in out.bands.windows(2) {
            assert!(w[1].y + w[1].h <= w[0].y, "floor above must end before the floor below starts");
            assert_eq!(w[0].w, w[1].w);
        }
        assert_eq!(out.bands[0].y + out.bands[0].h, 0.0, "ground floor sits on y = 0");
        assert!(out.placed[0].y < out.bands[5].y, "root sits in the roof");

        // Every visible node lies inside its floor's band, and no two label
        // boxes intersect.
        let mut boxes: Vec<(f64, f64, f64, f64)> = Vec::new();
        for band in &out.bands {
            let mut stack = vec![band.node];
            while let Some(i) = stack.pop() {
                let p = out.placed[i];
                if !p.visible {
                    continue;
                }
                let extra = if p.depth < 3 { COUNT_W } else { 0.0 };
                let b = (p.x - 8.0, p.y - ROW / 2.0 + 1.0, p.x - 8.0 + DOT + widths[i] + extra, p.y + ROW / 2.0 - 1.0);
                assert!(b.0 >= band.x && b.2 <= band.x + band.w, "node {i} outside band horizontally");
                assert!(b.1 >= band.y && b.3 <= band.y + band.h, "node {i} outside band vertically");
                boxes.push(b);
                stack.extend(nodes[i].children.iter().copied());
            }
        }
        for a in 0..boxes.len() {
            for b in (a + 1)..boxes.len() {
                let (p, q) = (boxes[a], boxes[b]);
                let apart = p.2 <= q.0 || q.2 <= p.0 || p.3 <= q.1 || q.3 <= p.1;
                assert!(apart, "labels overlap: {p:?} {q:?}");
            }
        }
        // The folded floor shows no rooms.
        for &r in &nodes[floors[5]].children {
            assert!(!out.placed[r].visible);
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
