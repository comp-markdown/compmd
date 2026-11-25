//! A size as a configuration writes it: bytes, or text such as `"5 MB"`.

use serde_json::Value;

/// The bytes in a unit, by its name in upper case.
fn unit_bytes(unit: &str) -> Option<f64> {
    match unit.to_ascii_uppercase().as_str() {
        "B" => Some(1.0),
        "KB" => Some(1e3),
        "MB" => Some(1e6),
        "GB" => Some(1e9),
        _ => None,
    }
}

/// Whether `number` is digits, with a decimal part of digits or none: `5`, `1.5`.
fn is_decimal(number: &str) -> bool {
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    match number.split_once('.') {
        Some((whole, fraction)) => digits(whole) && digits(fraction),
        None => digits(number),
    }
}

/// `size` in bytes, or `None` when it isn't a size: a positive number, or a decimal and a unit (`B`, `KB`, `MB` or
/// `GB`, in any case), with spaces or none between them.
pub(crate) fn bytes_of(size: &Value) -> Option<f64> {
    match size {
        Value::Number(number) => number.as_f64().filter(|bytes| *bytes > 0.0),
        Value::String(text) => {
            let text = text.trim();
            let split = text.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
            let (number, unit) = text.split_at(split);
            if !is_decimal(number) {
                return None;
            }
            let bytes: f64 = number.parse().ok()?;
            Some(bytes * unit_bytes(unit.trim_start())?)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_size_is_bytes_or_a_number_and_a_unit() {
        assert_eq!(bytes_of(&json!(5000)), Some(5000.0));
        assert_eq!(bytes_of(&json!("5 MB")), Some(5e6));
        assert_eq!(bytes_of(&json!(" 1.5kb ")), Some(1500.0));
        assert_eq!(bytes_of(&json!("2GB")), Some(2e9));
        assert_eq!(bytes_of(&json!("10 B")), Some(10.0));
    }

    #[test]
    fn anything_else_is_not_a_size() {
        for size in [
            json!(0),
            json!(-1),
            json!("5"),
            json!("MB"),
            json!("5 TB"),
            json!(".5 MB"),
            json!("5. MB"),
            json!("1.2.3 MB"),
            json!(true),
            json!(null),
        ] {
            assert_eq!(bytes_of(&size), None, "{size}");
        }
    }
}
