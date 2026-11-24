//! A document's data imports (spec §8.6): what a `from` may interpolate (F031, F014), and where remote data may come
//! from (F028). A pack's imports too, since its data is loaded for its definitions; only the processed document's
//! `from` sees inputs.

use super::Checker;
use super::scope::Scope;
use crate::AllowedOrigins;
use crate::diagnostic::{Code, Location};
use crate::parse::params::interpolations;

impl Checker<'_, '_> {
    pub(super) fn data(&mut self, allowed_origins: &AllowedOrigins) {
        let resolved = self.resolved;
        let settings = &resolved.document(self.id).settings;
        let scope = Scope::body(resolved, self.id);
        for (index, import) in settings.data.iter().enumerate() {
            let chars: Vec<char> = import.from.value.chars().collect();
            let literal = import.source.literal_prefix(&chars);
            let mut loadable = true;
            for (at_char, path) in interpolations(&chars) {
                let at = Location::new(import.text_at.line, import.text_at.column + at_char);
                loadable &= self.data_path(&path, at_char < literal, at, &scope);
            }
            let from = &import.from.value;
            if let Some(origin) = import
                .source
                .origin(from)
                .filter(|origin| !allowed_origins.allows(origin))
            {
                self.report(
                    Code::F028,
                    import.from.at,
                    format!("this processor isn't configured to fetch from `{origin}`; the data is its `fallback`, or null"),
                );
                loadable = false;
            }
            if !loadable {
                self.refuse_data(index);
            }
        }
    }

    /// An interpolation in a `from`: only `md.input` and `md.doc`, never in a URL's scheme or host, and never an input
    /// a host-language component can change (F031); whether it may be loaded.
    fn data_path(
        &mut self,
        path: &str,
        in_literal_prefix: bool,
        at: Location,
        scope: &Scope,
    ) -> bool {
        let why = if in_literal_prefix {
            "the start of a `from`, a URL's scheme and host, is written as is, so the origin can be checked"
        } else if !(path.starts_with("md.input.") || path.starts_with("md.doc.")) {
            "a `from` may interpolate only `md.input` and `md.doc`, so data from one source can't be sent to another"
        } else if path.strip_prefix("md.input.").is_some_and(|input| {
            self.bound_by_hosts
                .contains(input.split('.').next().unwrap_or(input))
        }) {
            "a host-language component can change this input, so its code could send data out through it"
        } else {
            self.path(path, at, scope);
            return scope.knows(path);
        };
        self.report(Code::F031, at, why);
        false
    }
}
