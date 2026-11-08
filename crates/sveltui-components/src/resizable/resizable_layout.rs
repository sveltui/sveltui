//! The arithmetic behind resizable panels, with no rendering in it: panel
//! sizes are percentages that sum to 100, and a handle moves space between
//! the two panels either side of it.

/// How far one panel may be resized.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Constraints {
    pub min: f32,
    pub max: f32,
    pub collapsible: bool,
    /// The size a collapsed panel keeps.
    pub collapsed: f32,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            collapsible: false,
            collapsed: 0.0,
        }
    }
}

impl Constraints {
    /// The smallest size the panel can take, counting collapsing.
    fn floor(&self) -> f32 {
        if self.collapsible {
            self.collapsed.min(self.min)
        } else {
            self.min
        }
    }

    /// Sizes between the collapsed size and the minimum aren't allowed: they
    /// snap to whichever is nearer.
    fn snap(&self, size: f32) -> f32 {
        if self.collapsible && size < self.min {
            if size < (self.collapsed + self.min) / 2.0 {
                self.collapsed
            } else {
                self.min
            }
        } else {
            size
        }
    }

    pub(crate) fn is_collapsed(&self, size: f32) -> bool {
        self.collapsible && size <= self.collapsed + EPSILON
    }
}

const EPSILON: f32 = 0.001;

/// Sizes for panels that may or may not state one: the stated sizes are
/// kept and the rest share what remains equally.
pub(crate) fn initial_layout(defaults: &[Option<f32>], constraints: &[Constraints]) -> Vec<f32> {
    let stated: f32 = defaults.iter().flatten().sum();
    let unstated = defaults.iter().filter(|d| d.is_none()).count();
    let share = if unstated == 0 {
        0.0
    } else {
        ((100.0 - stated) / unstated as f32).max(0.0)
    };
    let sizes: Vec<f32> = defaults.iter().map(|d| d.unwrap_or(share)).collect();
    normalize(&sizes, constraints)
}

/// Scales `sizes` to total 100 and holds each within its constraints.
pub(crate) fn normalize(sizes: &[f32], constraints: &[Constraints]) -> Vec<f32> {
    if sizes.is_empty() {
        return Vec::new();
    }
    let total: f32 = sizes.iter().sum();
    let mut out: Vec<f32> = if total <= EPSILON {
        vec![100.0 / sizes.len() as f32; sizes.len()]
    } else {
        sizes.iter().map(|s| s * 100.0 / total).collect()
    };
    // Clamping moves the total off 100; hand the difference to panels that
    // still have room, a few passes being plenty for realistic layouts.
    for _ in 0..sizes.len() + 1 {
        for (size, c) in out.iter_mut().zip(constraints) {
            *size = size.clamp(c.floor(), c.max);
        }
        let gap = 100.0 - out.iter().sum::<f32>();
        if gap.abs() <= EPSILON {
            break;
        }
        let room: Vec<usize> = (0..out.len())
            .filter(|&i| match constraints.get(i) {
                Some(c) if gap > 0.0 => out[i] < c.max - EPSILON,
                Some(c) => out[i] > c.floor() + EPSILON,
                None => true,
            })
            .collect();
        if room.is_empty() {
            break;
        }
        let each = gap / room.len() as f32;
        for i in room {
            out[i] += each;
        }
    }
    out
}

/// Moves `delta` percent from the panel after `handle` to the one before it
/// (negative moves the other way), within both panels' constraints. A
/// collapsible panel pushed below its minimum collapses or holds its
/// minimum, whichever is nearer. Returns `sizes` unchanged when nothing can
/// move.
pub(crate) fn resize(
    sizes: &[f32],
    constraints: &[Constraints],
    handle: usize,
    delta: f32,
) -> Vec<f32> {
    let (i, j) = (handle, handle + 1);
    if j >= sizes.len() || j >= constraints.len() {
        return sizes.to_vec();
    }
    let (ci, cj) = (constraints[i], constraints[j]);
    let total = sizes[i] + sizes[j];
    // How far each side may go either way.
    let grow_i = (ci.max - sizes[i]).min(sizes[j] - cj.floor());
    let shrink_i = (sizes[i] - ci.floor()).min(cj.max - sizes[j]);
    let delta = delta.clamp(-shrink_i.max(0.0), grow_i.max(0.0));

    let mut new_i = sizes[i] + delta;
    let mut new_j = sizes[j] - delta;
    if delta < 0.0 {
        new_i = ci.snap(new_i);
        new_j = total - new_i;
    } else if delta > 0.0 {
        new_j = cj.snap(new_j);
        new_i = total - new_j;
    }
    // Snapping can hand the other panel more than it may take; if so the
    // snap is refused and the panel stays where the clamp left it.
    if new_i > ci.max + EPSILON || new_j > cj.max + EPSILON {
        new_i = (sizes[i] + delta).min(ci.max);
        new_j = total - new_i;
    }
    let mut out = sizes.to_vec();
    out[i] = new_i;
    out[j] = new_j;
    out
}

/// Collapses a collapsible panel next to `handle`, or restores it to
/// `restore` (or its minimum). Prefers the panel before the handle.
/// Returns the new sizes and which panel changed, with the size it had
/// when it was collapsed.
pub(crate) fn toggle_collapse(
    sizes: &[f32],
    constraints: &[Constraints],
    handle: usize,
    restore: Option<f32>,
) -> Option<(Vec<f32>, usize, f32)> {
    let side = [handle, handle + 1]
        .into_iter()
        .find(|&p| constraints.get(p).is_some_and(|c| c.collapsible) && p < sizes.len())?;
    let c = constraints[side];
    let before = sizes[side];
    let target = if c.is_collapsed(before) {
        restore.unwrap_or(c.min).max(c.min)
    } else {
        c.collapsed
    };
    // `resize` works on the pair around a handle; the panel before the
    // handle grows with a positive delta, the one after it with a negative.
    let delta = if side == handle {
        target - before
    } else {
        before - target
    };
    let out = resize(sizes, constraints, handle, delta);
    ((out[side] - before).abs() > EPSILON).then_some((out, side, before))
}
