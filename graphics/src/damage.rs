//! Compute the damage between frames.
use crate::core::{Point, Rectangle};

/// Diffs the damage regions given some previous and current primitives.
pub fn diff<T>(
    previous: &[T],
    current: &[T],
    bounds: impl Fn(&T) -> Vec<Rectangle>,
    diff: impl Fn(&T, &T) -> Vec<Rectangle>,
) -> Vec<Rectangle> {
    let damage = previous.iter().zip(current).flat_map(|(a, b)| diff(a, b));

    if previous.len() == current.len() {
        damage.collect()
    } else {
        let (smaller, bigger) = if previous.len() < current.len() {
            (previous, current)
        } else {
            (current, previous)
        };

        // Extend damage by the added/removed primitives
        damage
            .chain(bigger[smaller.len()..].iter().flat_map(bounds))
            .collect()
    }
}

/// Computes the damage regions given some previous and current primitives.
pub fn list<T>(
    previous: &[T],
    current: &[T],
    bounds: impl Fn(&T) -> Vec<Rectangle>,
    are_equal: impl Fn(&T, &T) -> bool,
) -> Vec<Rectangle> {
    diff(previous, current, &bounds, |a, b| {
        if are_equal(a, b) {
            vec![]
        } else {
            bounds(a).into_iter().chain(bounds(b)).collect()
        }
    })
}

/// Groups the given damage regions that are close together inside the given
/// bounds, or returns the bounds whole once the regions cover a quarter of it.
pub fn group(mut damage: Vec<Rectangle>, bounds: Rectangle) -> Vec<Rectangle> {
    const AREA_THRESHOLD: f32 = 20_000.0;

    damage.sort_by(|a, b| {
        a.center()
            .distance(Point::ORIGIN)
            .total_cmp(&b.center().distance(Point::ORIGIN))
    });

    let mut output = Vec::new();
    let mut scaled = damage
        .into_iter()
        .filter_map(|region| region.intersection(&bounds))
        .filter(|region| region.width >= 1.0 && region.height >= 1.0);

    if let Some(mut current) = scaled.next() {
        for region in scaled {
            let union = current.union(&region);

            if union.area() - current.area() - region.area() <= AREA_THRESHOLD {
                current = union;
            } else {
                output.push(current);
                current = region;
            }
        }

        output.push(current);
    }

    // Regions overlap and each one is redrawn in full, so a frame that moves
    // most of the window (a scrolled list) costs several full repaints. Past
    // a quarter of the bounds, one repaint of everything is the cheaper frame.
    let damaged: f32 = output.iter().map(Rectangle::area).sum();

    if damaged * 4.0 >= bounds.area() {
        return vec![bounds];
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Size;

    #[test]
    fn a_small_region_stays_a_region() {
        let bounds = Rectangle::with_size(Size::new(100.0, 100.0));
        let row = Rectangle::new(Point::new(0.0, 10.0), Size::new(100.0, 20.0));

        assert_eq!(group(vec![row], bounds), vec![row]);
    }

    #[test]
    fn regions_covering_a_quarter_become_one_full_repaint() {
        let bounds = Rectangle::with_size(Size::new(100.0, 100.0));
        let rows = (0..10)
            .map(|i| {
                Rectangle::new(
                    Point::new(0.0, i as f32 * 10.0),
                    Size::new(100.0, 3.0),
                )
            })
            .collect();

        assert_eq!(group(rows, bounds), vec![bounds]);
    }
}
