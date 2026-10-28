//! The host languages of version 1's bindings (spec §7, §7.6), and the runtimes each takes.

use crate::named::named;

named! {
    /// A host language a binding covers: the first word of a definition fence, or a component file's extension.
    pub enum Language {
        /// JSX, for React or Preact.
        Jsx = "jsx",
        /// TSX, for React or Preact.
        Tsx = "tsx",
        /// Svelte 5.
        Svelte = "svelte",
        /// Plain JavaScript.
        Js = "js",
        /// Plain TypeScript.
        Ts = "ts",
    }
}

impl Language {
    /// The runtimes its binding runs, the default first; none for plain JavaScript, which takes no `runtime`.
    #[must_use]
    pub fn runtimes(self) -> &'static [&'static str] {
        match self {
            Self::Jsx | Self::Tsx => &["react@19", "preact@10"],
            Self::Svelte => &["svelte@5"],
            Self::Js | Self::Ts => &[],
        }
    }

    /// Whether its binding runs `runtime`, or the default when it is `None`.
    #[must_use]
    pub fn runs(self, runtime: Option<&str>) -> bool {
        runtime.is_none_or(|runtime| self.runtimes().contains(&runtime))
    }

    /// Whether a component in it should name its runtime when exported, since the default may move (W117).
    #[must_use]
    pub fn has_runtimes(self) -> bool {
        !self.runtimes().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_binding_runs_its_runtimes() {
        assert!(Language::Jsx.runs(None));
        assert!(Language::Tsx.runs(Some("preact@10")));
        assert!(!Language::Svelte.runs(Some("svelte@4")));
        assert!(!Language::Js.runs(Some("react@19")));
        assert!(Language::Svelte.has_runtimes());
        assert!(!Language::Ts.has_runtimes());
        assert_eq!(Language::from_name("vue"), None);
    }
}
