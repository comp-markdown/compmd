//! How the app runs: showing its document, or checking itself (docs/tauri-plugin.md, decision 9).

use std::path::PathBuf;

/// How the app runs, from its arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Shows the document.
    Show,
    /// Renders it, checks what it drew, and exits `0` or `1`.
    Check,
    /// Serves frames whose runtime is missing, and checks that the sandbox says so.
    CheckBroken,
}

impl Mode {
    /// The mode `args` ask for: `--check`, `--check-broken`, or neither.
    #[must_use]
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        let mut mode = Self::Show;
        for arg in args {
            match arg.as_str() {
                "--check" => mode = Self::Check,
                "--check-broken" => mode = Self::CheckBroken,
                _ => {}
            }
        }
        mode
    }

    /// Its name, as the page reads it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Check => "check",
            Self::CheckBroken => "check-broken",
        }
    }

    /// Whether the app checks itself, and so must exit even if the page never reports.
    #[must_use]
    pub const fn checks(self) -> bool {
        !matches!(self, Self::Show)
    }

    /// The plugin for this mode: the embedded frames, as an app adds it in one call; or, for `CheckBroken`, a build
    /// of the app's own whose runtime is missing (`pnpm build` writes it).
    #[must_use]
    pub fn plugin<R: tauri::Runtime>(self) -> tauri::plugin::TauriPlugin<R> {
        match self {
            Self::CheckBroken => tauri_plugin_compmd::Builder::new()
                .frames_dir(broken())
                .build(),
            Self::Show | Self::Check => tauri_plugin_compmd::init(),
        }
    }
}

/// Where `scripts/broken.mts` writes the broken frames.
fn broken() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("broken")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arguments_choose_the_mode() {
        let mode = |args: &[&str]| Mode::from_args(args.iter().map(|arg| (*arg).to_owned()));
        assert_eq!(mode(&["app"]), Mode::Show);
        assert_eq!(mode(&["app", "--check"]), Mode::Check);
        assert_eq!(mode(&["app", "--check-broken"]), Mode::CheckBroken);
        assert_eq!(Mode::Check.name(), "check");
        assert!(Mode::CheckBroken.checks());
        assert!(!Mode::Show.checks());
    }
}
