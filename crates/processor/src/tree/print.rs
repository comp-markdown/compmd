//! What a use of a component shows on paper (spec §7.3, §14).

use serde::Serialize;

/// What a use of a component shows on paper: a signature's `print` (spec §7.3, §14).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Print {
    /// The component as it draws itself.
    #[default]
    Drawn,
    /// Its fallback (spec §4.5), for a component that means nothing on paper.
    Fallback,
}

impl Print {
    /// The value a signature's `print` names: `Some` for `drawn` and `fallback`, `None` for any other (W122).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "drawn" => Some(Self::Drawn),
            "fallback" => Some(Self::Fallback),
            _ => None,
        }
    }

    /// Whether it prints as drawn, the default, which the tree leaves out.
    #[must_use]
    pub fn is_drawn(&self) -> bool {
        *self == Self::Drawn
    }
}

#[cfg(test)]
mod tests {
    use super::Print;

    #[test]
    fn names_drawn_and_fallback_only() {
        assert_eq!(Print::from_name("drawn"), Some(Print::Drawn));
        assert_eq!(Print::from_name("fallback"), Some(Print::Fallback));
        assert_eq!(Print::from_name("Fallback"), None);
        assert_eq!(Print::from_name("hidden"), None);
    }

    #[test]
    fn is_left_out_of_the_tree_when_drawn() {
        assert!(Print::default().is_drawn());
        assert!(!Print::Fallback.is_drawn());
        assert_eq!(
            serde_json::to_value(Print::Fallback).ok(),
            Some(serde_json::json!("fallback"))
        );
    }
}
