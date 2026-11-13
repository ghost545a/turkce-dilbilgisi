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

#[cfg(test)]
mod tests {
    use super::*;

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
