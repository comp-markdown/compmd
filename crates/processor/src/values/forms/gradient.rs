//! A gradient in `background` (spec §9.4): `linear-gradient()`, `radial-gradient()` and their `repeating-` forms,
//! checked part by part. Every part is a word of the grammar, a number with a unit, or a CSS color, so a gradient loads
//! nothing and reaches no other CSS: another function, a quote or a `;` in one is refused.

use super::{function, is_color, is_length, split_top, with_unit};

/// The linear gradients.
const LINEAR: [&str; 2] = ["linear-gradient", "repeating-linear-gradient"];

/// The radial gradients.
const RADIAL: [&str; 2] = ["radial-gradient", "repeating-radial-gradient"];

/// The units of a gradient's angle.
const ANGLE_UNITS: [&str; 4] = ["deg", "rad", "grad", "turn"];

/// A radial gradient's sizes by keyword.
const EXTENTS: [&str; 4] = [
    "closest-side",
    "closest-corner",
    "farthest-side",
    "farthest-corner",
];

/// Whether a side follows the text's direction or is a fixed side of the screen.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Frame {
    Logical,
    Physical,
}

/// The axis a side is on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    Block,
    Inline,
}

/// A side word's frame and axis: `block-end` is a logical side on the block axis, `left` a physical one on the inline axis.
fn side(word: &str) -> Option<(Frame, Axis)> {
    match word {
        "block-start" | "block-end" => Some((Frame::Logical, Axis::Block)),
        "inline-start" | "inline-end" => Some((Frame::Logical, Axis::Inline)),
        "top" | "bottom" => Some((Frame::Physical, Axis::Block)),
        "left" | "right" => Some((Frame::Physical, Axis::Inline)),
        _ => None,
    }
}

/// Whether the sides among `words` are of one frame, at most one on each axis; words that aren't sides are left to
/// the caller.
fn sides_agree(words: &[&str]) -> bool {
    let sides: Vec<(Frame, Axis)> = words.iter().filter_map(|word| side(word)).collect();
    match sides[..] {
        [] | [_] => true,
        [(frame, axis), (other_frame, other_axis)] => frame == other_frame && axis != other_axis,
        _ => false,
    }
}

/// `to block-end inline-end`, `to left`: one or two sides after `to`.
fn is_direction(words: &[&str]) -> bool {
    match words {
        ["to", sides @ ..] => {
            (1..=2).contains(&sides.len())
                && sides.iter().all(|word| side(word).is_some())
                && sides_agree(sides)
        }
        _ => false,
    }
}

/// A linear gradient's first part: an angle, or a direction.
fn is_linear_head(words: &[&str]) -> bool {
    match words {
        [angle] => with_unit(angle, &ANGLE_UNITS),
        _ => is_direction(words),
    }
}

/// `at inline-start block-end`, `at center 40%`: one or two of `center`, a side, a length or a percentage.
fn is_position(words: &[&str]) -> bool {
    (1..=2).contains(&words.len())
        && words
            .iter()
            .all(|word| *word == "center" || side(word).is_some() || is_length(word))
        && sides_agree(words)
}

/// A radial gradient's shape and size, in either order: `circle`, `ellipse`, a size keyword, one length (a circle)
/// or two lengths or percentages (an ellipse).
fn is_shape_and_size(words: &[&str]) -> bool {
    let shapes: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| ["circle", "ellipse"].contains(word))
        .collect();
    let extents = words.iter().filter(|word| EXTENTS.contains(word)).count();
    let lengths: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| is_length(word))
        .collect();
    let shape = match shapes[..] {
        [] => None,
        [shape] => Some(shape),
        _ => return false,
    };
    let sized = match lengths[..] {
        [] => extents <= 1,
        [radius] => extents == 0 && shape != Some("ellipse") && !radius.ends_with('%'),
        [_, _] => extents == 0 && shape != Some("circle"),
        _ => false,
    };
    sized && shapes.len() + extents + lengths.len() == words.len()
}

/// A radial gradient's first part: a shape and size, a position after `at`, or both.
fn is_radial_head(words: &[&str]) -> bool {
    let (sized, position) = match words.iter().position(|word| *word == "at") {
        Some(at) => (&words[..at], Some(&words[at + 1..])),
        None => (words, None),
    };
    (!sized.is_empty() || position.is_some())
        && is_shape_and_size(sized)
        && position.is_none_or(is_position)
}

/// A color stop: a CSS color, then up to two lengths or percentages.
fn is_stop(part: &str) -> bool {
    match split_top(part, false).unwrap_or_default()[..] {
        [color] => is_color(color),
        [color, at] => is_color(color) && is_length(at),
        [color, from, to] => is_color(color) && is_length(from) && is_length(to),
        _ => false,
    }
}

/// Whether `text` is a gradient `background` takes.
pub(super) fn is_gradient(text: &str) -> bool {
    let Some((name, inner)) = function(text) else {
        return false;
    };
    let is_head: fn(&[&str]) -> bool = if LINEAR.contains(&name) {
        is_linear_head
    } else if RADIAL.contains(&name) {
        is_radial_head
    } else {
        return false;
    };
    let Some(parts) = split_top(inner, true) else {
        return false;
    };
    let stops = match parts.split_first() {
        Some((first, rest)) if !is_stop(first) => {
            let words = split_top(first, false).unwrap_or_default();
            if !is_head(&words) {
                return false;
            }
            rest
        }
        _ => &parts[..],
    };
    stops.len() >= 2 && stops.iter().all(|stop| is_stop(stop))
}

#[cfg(test)]
mod tests {
    use super::is_gradient;

    /// Gradients of every part of the grammar, each accepted.
    const ACCEPTED: [&str; 10] = [
        "linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)",
        "linear-gradient(to block-end inline-end, red, blue)",
        "linear-gradient(to bottom left, red 0 2rem, blue 2rem 4rem)",
        "linear-gradient(to block-end, transparent, color-mix(in srgb, currentColor 8%, transparent))",
        "repeating-linear-gradient(90deg, #f6f8fa 0 2rem, #ffffff 2rem 4rem)",
        "radial-gradient(circle at inline-start block-start, #fff5d6, transparent 60%)",
        "radial-gradient(farthest-corner circle at top, red, blue)",
        "radial-gradient(ellipse 4rem 50%, rgb(10 20 30 / 50%), blue)",
        "repeating-radial-gradient(closest-side, red 0 1rem, blue 1rem 2rem)",
        "radial-gradient(at center 40%, red, blue)",
    ];

    /// What would let a gradient load something or reach other CSS.
    const FORBIDDEN: [&str; 9] = [
        "url(x)",
        "var(--x)",
        "calc(1px)",
        "image(x)",
        ";",
        "{",
        "}",
        "\"",
        "\\",
    ];

    #[test]
    fn refuses_a_gradient_with_anything_forbidden_at_any_place() {
        for gradient in ACCEPTED {
            assert!(is_gradient(gradient), "refused {gradient}");
            for forbidden in FORBIDDEN {
                for (at, _) in gradient.char_indices() {
                    let spliced = format!("{}{forbidden}{}", &gradient[..at], &gradient[at..]);
                    assert!(!is_gradient(&spliced), "took {spliced}");
                }
            }
        }
    }

    #[test]
    fn takes_linear_gradients_with_angles_and_sides_of_one_frame() {
        for gradient in [
            "linear-gradient(red, blue)",
            "linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)",
            "linear-gradient(0.25turn, red, blue)",
            "linear-gradient(to block-end, transparent, color-mix(in srgb, currentColor 8%, transparent))",
            "linear-gradient(to block-end inline-end, red, blue)",
            "linear-gradient(to bottom left, red 0 2rem, blue 2rem 4rem)",
            "repeating-linear-gradient(90deg, #f6f8fa 0 2rem, #ffffff 2rem 4rem)",
        ] {
            assert!(is_gradient(gradient), "refused {gradient}");
        }
    }

    #[test]
    fn takes_radial_gradients_with_a_shape_a_size_and_a_position() {
        for gradient in [
            "radial-gradient(red, blue)",
            "radial-gradient(circle, red, blue)",
            "radial-gradient(circle at inline-start block-start, #fff5d6, transparent 60%)",
            "radial-gradient(at center 40%, red, blue)",
            "radial-gradient(farthest-corner circle at top, red, blue)",
            "radial-gradient(4rem, red, blue)",
            "radial-gradient(ellipse 4rem 50%, red, blue)",
            "repeating-radial-gradient(closest-side, red 0 1rem, blue 1rem 2rem)",
        ] {
            assert!(is_gradient(gradient), "refused {gradient}");
        }
    }

    #[test]
    fn refuses_what_isnt_a_linear_or_radial_gradient_of_the_grammar() {
        for gradient in [
            "conic-gradient(red, blue)",
            "linear-gradient(red)",
            "linear-gradient(url(x.png), red)",
            "linear-gradient(var(--x), red)",
            "linear-gradient(red calc(1px + 2px), blue)",
            "linear-gradient(to left block-end, red, blue)",
            "linear-gradient(to left right, red, blue)",
            "linear-gradient(to, red, blue)",
            "linear-gradient(to left top bottom, red, blue)",
            "linear-gradient(sideways, red, blue)",
            "linear-gradient(red; color: blue, blue)",
            "linear-gradient('red', blue)",
            "linear-gradient(red, blue",
            "linear-gradient(red 1px 2px 3px, blue)",
            "linear-gradient(red wide, blue)",
            "radial-gradient(circle ellipse, red, blue)",
            "radial-gradient(closest-side farthest-side, red, blue)",
            "radial-gradient(ellipse 4rem, red, blue)",
            "radial-gradient(circle 4rem 2rem, red, blue)",
            "radial-gradient(50%, red, blue)",
            "radial-gradient(closest-side 4rem, red, blue)",
            "radial-gradient(1rem 2rem 3rem, red, blue)",
            "radial-gradient(circle wide, red, blue)",
            "radial-gradient(at left block-end, red, blue)",
            "radial-gradient(at center center center, red, blue)",
            "radial-gradient(at nowhere, red, blue)",
            "radial-gradient(at, red, blue)",
            "red",
            "gradient",
        ] {
            assert!(!is_gradient(gradient), "took {gradient}");
        }
    }
}
