//! `font-family`'s value (spec §9.4): a type token, or a font of the document the param is written in.

use super::literal;
use crate::diagnostic::Code;
use crate::resolve::{Fallback, FontBinding};
use crate::tree::Param;
use crate::validate::Checker;
use crate::validate::params::value_at;
use crate::values::Kind;

impl Checker<'_, '_> {
    /// Whether `param`, a `font-family`, names a type token or a font of this document: F009 for a name no font has,
    /// F023 for one two wildcard imports give; a font from an import that failed or is pending is refused without a
    /// diagnostic. A quoted value with interpolations is checked at render.
    pub(super) fn font_family(&mut self, param: &Param) -> bool {
        let resolved = self.resolved;
        let fonts = &resolved.document(self.id).fonts;
        if let Some(written) = literal(&param.value) {
            let written = written.trim();
            if Fallback::from_name(written).is_some() {
                return true;
            }
            match fonts.get(written) {
                Some(FontBinding::Font(_)) => return true,
                Some(FontBinding::Dropped) => return false,
                Some(FontBinding::Ambiguous(sources)) => {
                    self.report(
                        Code::F023,
                        value_at(param),
                        format!(
                            "the font `{written}` is provided by {}; list it in one import's `fonts` to choose",
                            sources.join(" and ")
                        ),
                    );
                    return false;
                }
                None => {}
            }
        }
        let names: Vec<&str> = Fallback::NAMES.iter().copied().chain(fonts.all()).collect();
        if names.len() == Fallback::NAMES.len() {
            return self.takes(param, &Kind::FONT_FAMILY);
        }
        self.takes(param, &Kind::one_of(names))
    }

    /// Whether `param`'s value is of `kind`, F009 otherwise.
    fn takes(&mut self, param: &Param, kind: &Kind) -> bool {
        if kind.accepts(&param.value) {
            return true;
        }
        self.refused(param, "font-family", kind);
        false
    }
}
