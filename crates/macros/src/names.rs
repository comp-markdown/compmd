//! A variant's names: the function's, as expressions write it, and its tests'.

/// The name a variant gives its function: `GetFullYear` is `getFullYear`.
pub(crate) fn function(variant: &str) -> String {
    let mut chars = variant.chars();
    chars
        .next()
        .map(|first| first.to_lowercase().chain(chars).collect())
        .unwrap_or_default()
}

/// A Rust item's name in snake case: `GetFullYear` is `get_full_year`.
pub(crate) fn snake(name: &str) -> String {
    let mut snake = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            snake.push('_');
        }
        snake.extend(c.to_lowercase());
    }
    snake
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_function_is_its_variant_in_lower_camel_case() {
        assert_eq!(function("GetFullYear"), "getFullYear");
        assert_eq!(function("Size"), "size");
        assert_eq!(function(""), "");
    }

    #[test]
    fn tests_are_in_snake_case() {
        assert_eq!(snake("GetFullYear"), "get_full_year");
        assert_eq!(snake("MdFunction"), "md_function");
        assert_eq!(snake("Size"), "size");
    }
}
