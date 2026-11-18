//! What a code block a code renderer renders shows: its `render` param (spec §7.7).

use serde::Serialize;

/// What a rendered code block shows.
///
/// ```
/// use compmd::render::Render;
///
/// assert_eq!(Render::from_name("both"), Some(Render::Both));
/// assert_eq!(Render::default(), Render::Output);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Render {
    /// What the component draws: the default.
    #[default]
    Output,
    /// The code block alone; the component isn't run.
    Source,
    /// The code block, then what the component draws, in one box.
    Both,
}

impl Render {
    /// The values, as a document writes them.
    pub const NAMES: [&'static str; 3] = ["output", "source", "both"];

    /// The value a document writes as `name`, if it is one.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "output" => Some(Self::Output),
            "source" => Some(Self::Source),
            "both" => Some(Self::Both),
            _ => None,
        }
    }

    /// Whether the component runs.
    #[must_use]
    pub fn draws(self) -> bool {
        self != Self::Source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_three_values_and_nothing_else() {
        for name in Render::NAMES {
            assert!(Render::from_name(name).is_some());
        }
        assert_eq!(Render::from_name("code"), None);
        assert!(Render::Both.draws() && !Render::Source.draws());
        assert_eq!(
            serde_json::to_string(&Render::Source).ok().as_deref(),
            Some("\"source\"")
        );
    }
}
