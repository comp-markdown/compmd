//! The grammars of layout and styling values (spec §9.2, §9.4): a value outside its grammar is F009, so no param
//! can pass raw CSS through. Colors and borders take CSS syntax, checked here part by part.

/// A grammar a param's value must be in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Grammar {
    /// A length: a number with `px`, `rem`, `em` or `%`, or `0`.
    Length,
    /// A length or `auto`: sizes and insets.
    Size,
    /// A space token (`none`, `s`, `m`, `l`) or a length.
    Space,
    /// One or two spaces.
    Spaces,
    /// A radius token (`none`, `s`, `m`, `l`, `full`) or a length.
    Radius,
    /// A surface token (`none`, `subtle`, `strong`, `accent`) or a color.
    Surface,
    /// A border token (`none`, `subtle`, `strong`) or a CSS border: a width, a style and a color, each at most once.
    Border,
    /// A CSS color.
    Color,
    /// Grid tracks: lengths, `fr`, `auto`, `min-content`, `max-content`, `minmax()` and `repeat()`.
    Tracks,
    /// Quoted rows of area names or `.`, all the same length.
    Areas,
    /// An area name.
    AreaName,
    /// Width over height, `16/9`, or `none`.
    Ratio,
    /// A BCP 47 language tag.
    Language,
    /// One of these words.
    Keywords(&'static [&'static str]),
}

impl Grammar {
    /// Whether `value` is in the grammar.
    pub(crate) fn accepts(self, value: &str) -> bool {
        let value = value.trim();
        match self {
            Self::Length => is_length(value),
            Self::Size => value == "auto" || is_length(value),
            Self::Space => is_space(value),
            Self::Spaces => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                (1..=2).contains(&parts.len()) && parts.iter().all(|part| is_space(part))
            }
            Self::Radius => ["none", "s", "m", "l", "full"].contains(&value) || is_length(value),
            Self::Surface => {
                ["none", "subtle", "strong", "accent"].contains(&value) || is_color(value)
            }
            Self::Border => ["none", "subtle", "strong"].contains(&value) || is_border(value),
            Self::Color => is_color(value),
            Self::Tracks => is_tracks(value),
            Self::Areas => is_areas(value),
            Self::AreaName => is_ident(value),
            Self::Ratio => value == "none" || is_ratio(value),
            Self::Language => is_language(value),
            Self::Keywords(words) => words.contains(&value),
        }
    }
}

/// `12`, `-0.5`, `.5`: a CSS number.
fn number(text: &str) -> Option<f64> {
    let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
    let plain = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        && digits.bytes().filter(|b| *b == b'.').count() <= 1
        && digits != ".";
    plain.then(|| text.parse().ok()).flatten()
}

/// A number followed by one of `units`.
fn with_unit(text: &str, units: &[&str]) -> bool {
    units.iter().any(|unit| {
        text.strip_suffix(unit)
            .is_some_and(|amount| number(amount).is_some())
    })
}

fn is_length(text: &str) -> bool {
    text == "0" || with_unit(text, &["px", "rem", "em", "%"])
}

fn is_space(text: &str) -> bool {
    ["none", "s", "m", "l"].contains(&text) || is_length(text)
}

fn is_ratio(text: &str) -> bool {
    let positive = |part: &str| number(part.trim()).is_some_and(|n| n > 0.0);
    match text.split_once('/') {
        Some((width, height)) => positive(width) && positive(height),
        None => positive(text),
    }
}

/// A CSS identifier as area and track names use them.
fn is_ident(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `en`, `de-CH`, `zh-Hant-TW`: letters and digits in parts of up to eight, the first of letters.
fn is_language(text: &str) -> bool {
    let mut parts = text.split('-');
    let first = parts.next().unwrap_or("");
    (2..=8).contains(&first.len())
        && first.bytes().all(|b| b.is_ascii_alphabetic())
        && parts.all(|part| {
            (1..=8).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

/// `'head head' 'side main'`.
fn is_areas(text: &str) -> bool {
    let mut rows = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        let Some(quote) = rest.chars().next().filter(|c| *c == '\'' || *c == '"') else {
            return false;
        };
        let Some(end) = rest[1..].find(quote) else {
            return false;
        };
        rows.push(&rest[1..=end]);
        rest = rest[end + 2..].trim_start();
    }
    let lengths: Vec<usize> = rows
        .iter()
        .map(|row| {
            let cells: Vec<&str> = row.split_whitespace().collect();
            if cells.iter().all(|cell| *cell == "." || is_ident(cell)) {
                cells.len()
            } else {
                0
            }
        })
        .collect();
    lengths
        .first()
        .is_some_and(|first| *first > 0 && lengths.iter().all(|n| n == first))
}

/// Splits `text` at top-level separators: white space, or `,` when `commas`. Parentheses group.
fn split_top(text: &str, commas: bool) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if commas && depth == 0 => {
                parts.push(text[start..i].trim());
                start = i + 1;
            }
            c if !commas && c.is_whitespace() && depth == 0 => {
                if !text[start..i].trim().is_empty() {
                    parts.push(text[start..i].trim());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    if commas || !text[start..].trim().is_empty() {
        parts.push(text[start..].trim());
    }
    Some(parts)
}

/// `name(arguments)`: the name and the text between the parentheses.
fn function(text: &str) -> Option<(&str, &str)> {
    let open = text.find('(')?;
    let inner = text.strip_suffix(')')?.get(open + 1..)?;
    Some((&text[..open], inner))
}

fn is_tracks(text: &str) -> bool {
    split_top(text, false)
        .is_some_and(|tracks| !tracks.is_empty() && tracks.iter().all(|track| is_track(track)))
}

fn is_breadth(text: &str) -> bool {
    ["auto", "min-content", "max-content"].contains(&text)
        || is_length(text)
        || with_unit(text, &["fr"])
}

fn is_track(text: &str) -> bool {
    match function(text) {
        Some(("minmax", inner)) => split_top(inner, true).is_some_and(
            |args| matches!(args[..], [min, max] if is_breadth(min) && is_breadth(max)),
        ),
        Some(("repeat", inner)) => split_top(inner, true).is_some_and(|args| match args[..] {
            [count, tracks] => {
                let count_ok = count == "auto-fill"
                    || count == "auto-fit"
                    || count.parse::<u32>().is_ok_and(|n| n > 0);
                count_ok && is_tracks(tracks)
            }
            _ => false,
        }),
        Some(_) => false,
        None => is_breadth(text),
    }
}

/// The border styles of CSS.
const BORDER_STYLES: [&str; 10] = [
    "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
];

/// A width, a style and a color, each at most once, in any order.
fn is_border(text: &str) -> bool {
    let Some(parts) = split_top(text, false) else {
        return false;
    };
    let (mut width, mut style, mut color) = (false, false, false);
    for part in parts {
        let slot = if !width && (["thin", "medium", "thick"].contains(&part) || is_length(part)) {
            &mut width
        } else if !style && BORDER_STYLES.contains(&part) {
            &mut style
        } else if !color && is_color(part) {
            &mut color
        } else {
            return false;
        };
        *slot = true;
    }
    width || style || color
}

/// The color functions of CSS and how their arguments are read.
const COLOR_FUNCTIONS: [&str; 10] = [
    "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color",
];

fn is_color(text: &str) -> bool {
    if let Some(hex) = text.strip_prefix('#') {
        return [3, 4, 6, 8].contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit());
    }
    match function(text) {
        Some(("color-mix", inner)) => is_color_mix(inner),
        Some((name, inner)) if COLOR_FUNCTIONS.contains(&name) => is_color_arguments(name, inner),
        Some(_) => false,
        None => {
            let lower = text.to_ascii_lowercase();
            lower == "transparent"
                || lower == "currentcolor"
                || NAMED_COLORS.contains(&lower.as_str())
        }
    }
}

/// A component of a color: a number, a percentage, an angle or `none`.
fn is_component(text: &str) -> bool {
    text == "none"
        || number(text).is_some()
        || with_unit(text, &["%", "deg", "rad", "grad", "turn"])
}

/// Three components and an optional alpha, with spaces and `/`, or, for `rgb()` and `hsl()`, with commas.
fn is_color_arguments(name: &str, inner: &str) -> bool {
    let (inner, space) = match name {
        "color" => {
            let Some((space, rest)) = inner.trim().split_once(char::is_whitespace) else {
                return false;
            };
            (rest, Some(space))
        }
        _ => (inner, None),
    };
    if space.is_some_and(|space| !is_ident(space)) {
        return false;
    }
    if inner.contains(',') {
        let legacy = ["rgb", "rgba", "hsl", "hsla"].contains(&name);
        return legacy
            && split_top(inner, true).is_some_and(|parts| {
                (3..=4).contains(&parts.len()) && parts.iter().all(|part| is_component(part))
            });
    }
    let (channels, alpha) = match inner.split_once('/') {
        Some((channels, alpha)) => (channels, Some(alpha.trim())),
        None => (inner, None),
    };
    let channels: Vec<&str> = channels.split_whitespace().collect();
    channels.len() == 3
        && channels.iter().all(|channel| is_component(channel))
        && alpha.is_none_or(is_component)
}

/// `in <space> [<hue> hue], <color> [<pct>], <color> [<pct>]`.
fn is_color_mix(inner: &str) -> bool {
    let Some(parts) = split_top(inner, true) else {
        return false;
    };
    let [method, first, second] = parts[..] else {
        return false;
    };
    let method: Vec<&str> = method.split_whitespace().collect();
    let method_ok = match method[..] {
        ["in", space] => is_ident(space),
        ["in", space, hue, "hue"] => {
            is_ident(space) && ["shorter", "longer", "increasing", "decreasing"].contains(&hue)
        }
        _ => false,
    };
    // Each part comes from a split that found its parentheses balanced, so it splits again.
    let mixed = |part: &str| {
        let words = split_top(part, false).unwrap_or_default();
        match words[..] {
            [color] => is_color(color),
            [color, amount] | [amount, color] if with_unit(amount, &["%"]) => is_color(color),
            _ => false,
        }
    };
    method_ok && mixed(first) && mixed(second)
}

/// The named colors of CSS Color 4.
const NAMED_COLORS: [&str; 148] = [
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn check(grammar: Grammar, good: &[&str], bad: &[&str]) {
        for value in good {
            assert!(grammar.accepts(value), "{grammar:?} refused {value}");
        }
        for value in bad {
            assert!(!grammar.accepts(value), "{grammar:?} took {value}");
        }
    }

    #[test]
    fn lengths_sizes_spaces_and_ratios() {
        check(
            Grammar::Length,
            &["0", "16px", "2.5rem", ".5em", "40%", "-8px"],
            &["", "px", "12", "1.2.3px", "wide", "1vw"],
        );
        check(Grammar::Size, &["auto", "50%"], &["wide"]);
        check(
            Grammar::Space,
            &["none", "s", "m", "l", "2.5rem"],
            &["bigger", "xl"],
        );
        check(
            Grammar::Spaces,
            &["s l", "m", "1rem 2px"],
            &["", "s m l", "s big"],
        );
        check(Grammar::Radius, &["full", "8px"], &["round"]);
        check(
            Grammar::Ratio,
            &["16/9", "1.5", "none", "4 / 3"],
            &["0/1", "16:9", "wide"],
        );
    }

    #[test]
    fn tracks_and_areas() {
        check(
            Grammar::Tracks,
            &[
                "1fr 2fr",
                "12rem 1fr",
                "repeat(auto-fill, minmax(12rem, 1fr))",
                "repeat(3, 1fr auto)",
                "min-content",
            ],
            &[
                "",
                "repeat(0, 1fr)",
                "repeat(3)",
                "minmax(1fr)",
                "calc(1px)",
                "1fr (",
                "wide",
            ],
        );
        check(
            Grammar::Areas,
            &["'head head' 'side main'", "\"a . b\""],
            &["", "'head' 'side main'", "head head", "'a", "'a-b! c'"],
        );
        check(Grammar::AreaName, &["side", "main_2"], &["2col", "a b"]);
    }

    #[test]
    fn colors_surfaces_and_borders() {
        check(
            Grammar::Color,
            &[
                "#f5f5f5",
                "#abc",
                "#abcd",
                "#11223344",
                "RebeccaPurple",
                "transparent",
                "currentColor",
                "rgb(10 20 30)",
                "rgb(10 20 30 / 50%)",
                "rgba(10, 20, 30, 0.5)",
                "hsl(120deg 50% 50%)",
                "oklch(0.7 0.1 200)",
                "color(display-p3 1 0 0)",
                "color-mix(in srgb, currentColor 6%, transparent)",
                "color-mix(in oklch longer hue, red, blue 30%)",
            ],
            &[
                "#ggg",
                "#12345",
                "loud",
                "rgb(10 20)",
                "oklch(0.7, 0.1, 200)",
                "rgb(1 2 3 / x)",
                "url(x)",
                "var(--x)",
                "color-mix(srgb, red, blue)",
                "color-mix(in srgb, red, blue, green)",
                "color-mix(in srgb, red), (blue)",
                "color-mix(in srgb, red wide, blue)",
                "color(1 0 0 0)",
                "color(display-p3)",
            ],
        );
        check(
            Grammar::Surface,
            &["subtle", "accent", "#f6f8fa"],
            &["loud"],
        );
        check(
            Grammar::Border,
            &[
                "subtle",
                "1px solid",
                "1px dashed #d0d7de",
                "thick",
                "red dotted",
            ],
            &[
                "thick-dotted",
                "1px 2px",
                "solid solid",
                "",
                "1px solid red blue",
                "1px (solid",
            ],
        );
    }

    #[test]
    fn languages_and_keywords() {
        check(
            Grammar::Language,
            &["en", "de-CH", "zh-Hant-TW"],
            &["e", "en_US", "de-", "123"],
        );
        check(Grammar::Keywords(&["top", "bottom"]), &["top"], &["left"]);
    }
}
