//! The contrast of a box's text on its background (spec §9.2, W126): WCAG 2's ratio of two colors' relative
//! luminances, for colors written in hex, `rgb()` or `hsl()`, opaque. Any other color (a token, a named color, a
//! function the page resolves) is one the processor can't tell, so it has no ratio.

/// Text contrasting less than this with its background is W126 (WCAG 2, AA, normal text).
pub(super) const ENOUGH: f64 = 4.5;

/// The contrast ratio of `text` on `background`, from 1 to 21; `None` when either isn't a color it can tell.
pub(super) fn ratio(text: &str, background: &str) -> Option<f64> {
    let [one, other] = [luminance(rgb(text)?), luminance(rgb(background)?)];
    let (light, dark) = if one > other {
        (one, other)
    } else {
        (other, one)
    };
    Some((light + 0.05) / (dark + 0.05))
}

/// A color's red, green and blue, from 0 to 1.
type Rgb = [f64; 3];

/// WCAG 2's relative luminance.
fn luminance([red, green, blue]: Rgb) -> f64 {
    let linear = |channel: f64| {
        if channel <= 0.040_45 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
}

/// An opaque color written in hex, `rgb()` or `hsl()`.
fn rgb(color: &str) -> Option<Rgb> {
    let color = color.trim().to_ascii_lowercase();
    if let Some(hex) = color.strip_prefix('#') {
        return from_hex(hex);
    }
    let (name, inner) = color.strip_suffix(')')?.split_once('(')?;
    let parts = arguments(inner)?;
    match name.trim() {
        "rgb" | "rgba" => from_rgb(&parts),
        "hsl" | "hsla" => from_hsl(&parts),
        _ => None,
    }
}

fn from_hex(hex: &str) -> Option<Rgb> {
    let digits: Vec<u8> = hex
        .chars()
        .map(|digit| {
            digit
                .to_digit(16)
                .and_then(|value| u8::try_from(value).ok())
        })
        .collect::<Option<_>>()?;
    let channels: Vec<u8> = match digits.len() {
        3 | 4 => digits.iter().map(|digit| digit * 17).collect(),
        6 | 8 => digits
            .chunks(2)
            .map(|pair| pair[0] * 16 + pair[1])
            .collect(),
        _ => return None,
    };
    if channels.get(3).is_some_and(|alpha| *alpha != 255) {
        return None;
    }
    Some([0, 1, 2].map(|index| f64::from(channels[index]) / 255.0))
}

/// The arguments of a color function, with commas or spaces, an alpha after `/` or as the fourth; `None` when the
/// alpha isn't opaque.
fn arguments(inner: &str) -> Option<Vec<&str>> {
    let (channels, alpha) = match inner.split_once('/') {
        Some((channels, alpha)) => (channels, Some(alpha.trim())),
        None => (inner, None),
    };
    let mut parts: Vec<&str> = channels
        .split([',', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    let alpha = alpha.or_else(|| (parts.len() == 4).then(|| parts.remove(3)));
    if alpha.is_some_and(|alpha| !is_opaque(alpha)) || parts.len() != 3 {
        return None;
    }
    Some(parts)
}

fn is_opaque(alpha: &str) -> bool {
    match alpha.strip_suffix('%') {
        Some(percent) => percent.parse::<f64>() == Ok(100.0),
        None => alpha.parse::<f64>() == Ok(1.0),
    }
}

/// `rgb(255, 0, 0)`, `rgb(100% 0% 0%)`: each channel a number up to 255 or a percentage.
fn from_rgb(parts: &[&str]) -> Option<Rgb> {
    let channel = |part: &str| match part.strip_suffix('%') {
        Some(percent) => percent.parse::<f64>().ok().map(|value| value / 100.0),
        None => part.parse::<f64>().ok().map(|value| value / 255.0),
    };
    let [red, green, blue] = [channel(parts[0])?, channel(parts[1])?, channel(parts[2])?];
    Some([red, green, blue].map(|value| value.clamp(0.0, 1.0)))
}

/// `hsl(120, 100%, 25%)`, `hsl(120deg 100% 25%)`: CSS Color 4's conversion to RGB.
fn from_hsl(parts: &[&str]) -> Option<Rgb> {
    let hue = parts[0]
        .strip_suffix("deg")
        .unwrap_or(parts[0])
        .parse::<f64>()
        .ok()?;
    let percent = |part: &str| {
        part.strip_suffix('%')?
            .parse::<f64>()
            .ok()
            .map(|value| (value / 100.0).clamp(0.0, 1.0))
    };
    let (saturation, lightness) = (percent(parts[1])?, percent(parts[2])?);
    let channel = |offset: f64| {
        let k = (offset + hue.rem_euclid(360.0) / 30.0) % 12.0;
        let a = saturation * lightness.min(1.0 - lightness);
        lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    Some([channel(0.0), channel(8.0), channel(4.0)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(ratio: Option<f64>, expected: f64) -> bool {
        ratio.is_some_and(|ratio| (ratio - expected).abs() < 0.01)
    }

    #[test]
    fn black_on_white_is_21_and_a_color_on_itself_1() {
        assert!(close(ratio("#000", "#fff"), 21.0));
        assert!(close(ratio("#ffffff", "#000000"), 21.0));
        assert!(close(ratio("#777", "#777"), 1.0));
    }

    #[test]
    fn hex_rgb_and_hsl_are_read_alike() {
        // #767676 on white is the lightest grey with 4.5:1, WCAG's own example.
        assert!(close(ratio("#767676", "#fff"), 4.54));
        assert!(close(ratio("rgb(118, 118, 118)", "#fff"), 4.54));
        assert!(close(ratio("rgb(118 118 118)", "#fff"), 4.54));
        assert!(close(ratio("rgb(100% 0% 0%)", "#fff"), 4.0));
        assert!(close(ratio("hsl(0, 100%, 50%)", "#fff"), 4.0));
        assert!(close(
            ratio("hsl(120deg 100% 25%)", "#fff"),
            ratio("rgb(0% 50% 0%)", "#fff").unwrap_or_default()
        ));
    }

    #[test]
    fn what_it_can_t_tell_has_no_ratio() {
        for color in [
            "teal",
            "transparent",
            "currentColor",
            "#0000",
            "rgb(0 0 0 / 50%)",
            "rgba(0, 0, 0, 0.5)",
            "oklch(0.7 0.1 200)",
            "#12",
            "hsl(red, 1, 2)",
        ] {
            assert_eq!(ratio(color, "#fff"), None, "{color}");
        }
        assert!(ratio("rgb(0 0 0 / 1)", "#fff").is_some());
        assert!(ratio("#000000ff", "#fff").is_some());
    }
}
