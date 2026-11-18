//! What a param's value may be (spec §5.1, §7.3, §9.4), as one definition per kind of value: its tokens, the forms it
//! takes besides them, and an example of those forms. Checking a value, the F009 message that says what a param
//! takes, the suggestion of a misspelt token, and the `params.json` a renderer checks itself against all read it, so
//! a token or a form is written down once.
//!
//! ```
//! use compmd::values::Kind;
//!
//! assert!(Kind::BORDER.accepts_text("subtle") && Kind::BORDER.accepts_text("1px solid #ccc"));
//! assert_eq!(
//!     Kind::BORDER.describe(),
//!     "`none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de`"
//! );
//! assert_eq!(Kind::BORDER.suggest("subtl"), Some("subtle"));
//! ```

pub mod forms;
mod vocabulary;

use crate::nearest::nearest;
use crate::tree::{DeclaredType, ParamValue, StringPart, TypeName};
pub use forms::Form;
use std::borrow::Cow;
pub use vocabulary::{Known, Vocabulary, vocabulary};

/// A kind of value: the words it takes, the forms it also takes, and examples of those forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind<'a> {
    /// The words it takes, in the order a message lists them.
    pub tokens: Cow<'a, [&'a str]>,
    /// The forms it takes besides its tokens.
    pub forms: &'static [Form],
    /// Values of its forms, since a message lists every token but can't list a form's values: what "a CSS border"
    /// looks like. A kind of only tokens has none.
    pub examples: &'static [&'static str],
    /// How many space-separated values it takes, each a token or of a form: 2 for `padding`'s `s l`.
    pub up_to: usize,
    /// Whether a flag, written without a value, is one: `true`.
    pub flag: bool,
    /// Whether a quoted string may hold it; a number, a boolean and a name are written bare (spec §5.1).
    pub quoted: bool,
}

/// A kind, written as the constants are.
const fn kind(
    tokens: &'static [&'static str],
    forms: &'static [Form],
    examples: &'static [&'static str],
) -> Kind<'static> {
    Kind {
        tokens: Cow::Borrowed(tokens),
        forms,
        examples,
        up_to: 1,
        flag: false,
        quoted: true,
    }
}

impl Kind<'static> {
    /// The same kind, for a kind made from a declaration that lives shorter.
    fn lent<'a>(self) -> Kind<'a> {
        Kind {
            tokens: match self.tokens {
                Cow::Borrowed(tokens) => Cow::Borrowed(tokens),
                Cow::Owned(tokens) => Cow::Owned(tokens),
            },
            forms: self.forms,
            examples: self.examples,
            up_to: self.up_to,
            flag: self.flag,
            quoted: self.quoted,
        }
    }
}

const SPACE_TOKENS: &[&str] = &["none", "s", "m", "l"];

impl Kind<'static> {
    /// `gap`, the single paddings: a space token or a length.
    pub const SPACE: Self = kind(SPACE_TOKENS, &[Form::Length], &["1rem"]);
    /// `padding`, `padding-inline`, `padding-block`: one or two spaces.
    pub const SPACES: Self = Kind {
        tokens: Cow::Borrowed(SPACE_TOKENS),
        forms: &[Form::Length],
        examples: &["s l"],
        up_to: 2,
        flag: false,
        quoted: true,
    };
    /// `border-radius`: a radius token or a length.
    pub const RADIUS: Self = kind(&["none", "s", "m", "l", "full"], &[Form::Length], &["6px"]);
    /// `background`: a surface token or a CSS color.
    pub const SURFACE: Self = kind(
        &["none", "subtle", "strong", "accent"],
        &[Form::Color],
        &["#f6f8fa"],
    );
    /// `border`: a border token or a CSS border.
    pub const BORDER: Self = kind(
        &["none", "subtle", "strong"],
        &[Form::Border],
        &["1px solid #d0d7de"],
    );
    /// A CSS color.
    pub const COLOR: Self = kind(&[], &[Form::Color], &["#0b7285", "teal"]);
    /// A length.
    pub const LENGTH: Self = kind(&[], &[Form::Length], &["1.5rem"]);
    /// Sizes and insets: `auto` or a length.
    pub const SIZE: Self = kind(&["auto"], &[Form::Length], &["20rem"]);
    /// Grid tracks.
    pub const TRACKS: Self = kind(
        &[],
        &[Form::Tracks],
        &["1fr 2fr", "repeat(auto-fill, minmax(12rem, 1fr))"],
    );
    /// Grid areas.
    pub const AREAS: Self = kind(&[], &[Form::Areas], &["'head head' 'side main'"]);
    /// An area name.
    pub const AREA_NAME: Self = kind(&[], &[Form::AreaName], &["side"]);
    /// `aspect-ratio`: `none` or a width over a height.
    pub const RATIO: Self = kind(&["none"], &[Form::Ratio], &["16/9"]);
    /// `lang`: a language tag.
    pub const LANGUAGE: Self = kind(&[], &[Form::Language], &["en", "de-CH"]);
    /// A number, written bare.
    pub const NUMBER: Self = Kind {
        tokens: Cow::Borrowed(&[]),
        forms: &[Form::Number],
        examples: &["12", "0.5"],
        up_to: 1,
        flag: false,
        quoted: false,
    };
    /// `true` or `false`, or a flag; written bare.
    pub const BOOLEAN: Self = Kind {
        tokens: Cow::Borrowed(&["true", "false"]),
        forms: &[],
        examples: &[],
        up_to: 1,
        flag: true,
        quoted: false,
    };
    /// A date.
    pub const DATE: Self = kind(&[], &[Form::Date], &["2026-10-29"]);
    /// A JSON array.
    pub const ARRAY: Self = kind(&[], &[Form::Array], &["[1, 2]"]);
    /// A JSON object.
    pub const OBJECT: Self = kind(&[], &[Form::Object], &[r#"{"a": 1}"#]);
    /// A name the author binds, written bare and as is.
    pub const NAME: Self = Kind {
        tokens: Cow::Borrowed(&[]),
        forms: &[Form::Name],
        examples: &["row"],
        up_to: 1,
        flag: false,
        quoted: false,
    };
    /// Any text.
    pub const TEXT: Self = kind(&[], &[Form::Text], &[]);
    /// Any value: a type the spec doesn't define, or `enum` in `input<enum>`.
    pub const ANY: Self = Kind {
        tokens: Cow::Borrowed(&[]),
        forms: &[Form::Any],
        examples: &[],
        up_to: 1,
        flag: true,
        quoted: true,
    };

    /// Every constant kind, for the tests over them and `params.json`.
    pub const ALL: &'static [Self] = &[
        Self::SPACE,
        Self::SPACES,
        Self::RADIUS,
        Self::SURFACE,
        Self::BORDER,
        Self::COLOR,
        Self::LENGTH,
        Self::SIZE,
        Self::TRACKS,
        Self::AREAS,
        Self::AREA_NAME,
        Self::RATIO,
        Self::LANGUAGE,
        Self::NUMBER,
        Self::BOOLEAN,
        Self::DATE,
        Self::ARRAY,
        Self::OBJECT,
        Self::NAME,
        Self::TEXT,
        Self::ANY,
    ];
}

impl<'a> Kind<'a> {
    /// One of `tokens`, such as an enum's members.
    #[must_use]
    pub fn one_of(tokens: impl Into<Cow<'a, [&'a str]>>) -> Self {
        Kind {
            tokens: tokens.into(),
            forms: &[],
            examples: &[],
            up_to: 1,
            flag: false,
            quoted: true,
        }
    }

    /// The kind of a declared type. A bound input's value is its name, which is checked against the inputs instead.
    #[must_use]
    pub fn declared(declared: &'a DeclaredType) -> Self {
        match declared {
            DeclaredType::Name(name) => name.kind().lent(),
            DeclaredType::Enum { members } => {
                Self::one_of(members.iter().map(String::as_str).collect::<Vec<_>>())
            }
            DeclaredType::Other(_) | DeclaredType::Input { .. } => Kind::ANY.lent(),
        }
    }

    /// Whether a value written as `value` is of this kind. A string with interpolations is checked when it is
    /// rendered, and an expression by its type.
    #[must_use]
    pub fn accepts(&self, value: &ParamValue) -> bool {
        match value {
            ParamValue::Flag => self.flag,
            ParamValue::Json(json) => self.forms.iter().any(|form| form.takes_json(json)),
            ParamValue::Bare(text) => self.accepts_text(text),
            ParamValue::String(parts) => {
                let text: Option<String> = parts
                    .iter()
                    .map(|part| match part {
                        StringPart::Text(text) => Some(text.value.as_str()),
                        StringPart::Interpolation(_) => None,
                    })
                    .collect();
                text.is_none_or(|text| self.quoted && self.accepts_text(&text))
            }
            ParamValue::Expr(_) | ParamValue::Class(_) | ParamValue::Id(_) => true,
        }
    }

    /// Whether the text of a value, bare or quoted, is of this kind.
    #[must_use]
    pub fn accepts_text(&self, text: &str) -> bool {
        let one = |word: &str| {
            self.tokens.contains(&word) || self.forms.iter().any(|form| form.accepts(word))
        };
        let text = text.trim();
        if one(text) {
            return true;
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        self.up_to > 1 && (1..=self.up_to).contains(&words.len()) && words.into_iter().all(one)
    }

    /// What it takes, as a message says it: "`none`, `subtle`, `strong` or a CSS border, such as `1px solid
    /// #d0d7de`".
    #[must_use]
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = self
            .tokens
            .iter()
            .map(|token| format!("`{token}`"))
            .collect();
        parts.extend(self.forms.iter().map(|form| form.phrase().to_string()));
        let mut described = listed(&parts, "or");
        if self.up_to == 2 {
            described = format!("one or two of {described}");
        }
        if !self.examples.is_empty() {
            let examples: Vec<String> = self
                .examples
                .iter()
                .map(|example| format!("`{example}`"))
                .collect();
            described = format!("{described}, such as {}", listed(&examples, "or"));
        }
        if self.flag && !self.forms.contains(&Form::Any) {
            described = format!("{described}, or nothing, as a flag");
        }
        described
    }

    /// The token closest to `written`, when it is close enough to be a misspelling of one; for a kind that takes
    /// colors, CSS's color names count too.
    #[must_use]
    pub fn suggest(&self, written: &str) -> Option<&str> {
        let colors = self.forms.iter().any(|form| form.has_color_names());
        let color_names: &'a [&'a str] = if colors { &forms::NAMED_COLORS } else { &[] };
        let candidates = self.tokens.iter().chain(color_names).copied();
        nearest(written.trim(), candidates)
            .filter(|near| !near.eq_ignore_ascii_case(written.trim()))
    }
}

/// `a`, `a or b`, `a, b or c`.
fn listed(parts: &[String], last: &str) -> String {
    match parts.split_last() {
        Some((final_part, rest)) if !rest.is_empty() => {
            format!("{} {last} {final_part}", rest.join(", "))
        }
        _ => parts.concat(),
    }
}

impl TypeName {
    /// What a param of this type takes. Every type name has a kind: a new one doesn't compile until it has.
    #[must_use]
    pub fn kind(self) -> Kind<'static> {
        match self {
            Self::String => Kind::TEXT,
            Self::Number => Kind::NUMBER,
            Self::Boolean => Kind::BOOLEAN,
            Self::Date => Kind::DATE,
            Self::Array => Kind::ARRAY,
            Self::Object => Kind::OBJECT,
            Self::Length => Kind::LENGTH,
            Self::Space => Kind::SPACE,
            Self::Color => Kind::COLOR,
            Self::Tracks => Kind::TRACKS,
            Self::Areas => Kind::AREAS,
            Self::Surface => Kind::SURFACE,
            Self::Border => Kind::BORDER,
            Self::Radius => Kind::RADIUS,
            Self::Name => Kind::NAME,
            Self::Enum => Kind::ANY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_takes_its_examples_and_names_its_tokens() {
        for kind in Kind::ALL {
            let described = kind.describe();
            for token in kind.tokens.iter() {
                assert!(
                    kind.accepts_text(token),
                    "{kind:?} refuses its token {token}"
                );
                assert!(
                    described.contains(&format!("`{token}`")),
                    "{described} misses {token}"
                );
            }
            for example in kind.examples {
                // An example is written as a param would be: JSON as JSON, anything else bare.
                let written = if example.starts_with(['[', '{']) {
                    ParamValue::Json((*example).into())
                } else {
                    ParamValue::Bare((*example).into())
                };
                assert!(
                    kind.accepts(&written),
                    "{kind:?} refuses its example {example}"
                );
                assert!(
                    !kind.tokens.contains(example),
                    "{example} is a token of {kind:?}"
                );
                assert!(
                    described.contains(&format!("`{example}`")),
                    "{described} misses {example}"
                );
            }
            for form in kind.forms {
                assert!(
                    described.contains(form.phrase()),
                    "{described} misses {form:?}"
                );
            }
            let anything = kind
                .forms
                .iter()
                .any(|form| matches!(form, Form::Text | Form::Any));
            assert_eq!(
                kind.examples.is_empty(),
                kind.forms.is_empty() || anything,
                "{kind:?}: a kind with forms has examples, unless it takes any text"
            );
        }
    }

    #[test]
    fn each_token_is_suggested_for_a_typo_of_it() {
        for kind in Kind::ALL {
            for token in kind
                .tokens
                .iter()
                .filter(|token| token.chars().count() >= 3)
            {
                let typo: String = token.chars().skip(1).collect();
                assert_eq!(kind.suggest(&typo), Some(*token), "{typo} for {kind:?}");
            }
        }
        assert_eq!(Kind::BORDER.suggest("subtl"), Some("subtle"));
        assert_eq!(Kind::RADIUS.suggest("ful"), Some("full"));
        assert_eq!(Kind::SURFACE.suggest("blu"), Some("blue"));
        assert_eq!(Kind::SPACE.suggest("huge"), None);
        assert_eq!(
            Kind::SPACE.suggest("x"),
            None,
            "too short to be a likely misspelling"
        );
    }

    #[test]
    fn messages_say_what_each_kind_takes() {
        assert_eq!(
            Kind::SPACES.describe(),
            "one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l`"
        );
        assert_eq!(
            Kind::BOOLEAN.describe(),
            "`true` or `false`, or nothing, as a flag"
        );
        assert_eq!(Kind::one_of(&["a", "b"][..]).describe(), "`a` or `b`");
        assert_eq!(Kind::TEXT.describe(), "text");
        assert_eq!(
            Kind::COLOR.describe(),
            "a CSS color, such as `#0b7285` or `teal`"
        );
    }

    #[test]
    fn values_are_checked_by_how_they_are_written() {
        let quoted = |text: &str| {
            ParamValue::String(vec![StringPart::Text(crate::tree::Literal {
                value: text.into(),
                ..Default::default()
            })])
        };
        assert!(Kind::BORDER.accepts(&quoted("1px solid")));
        assert!(
            !Kind::NUMBER.accepts(&quoted("3")),
            "a number is written bare"
        );
        assert!(Kind::NUMBER.accepts(&ParamValue::Bare("3".into())));
        assert!(Kind::BOOLEAN.accepts(&ParamValue::Flag) && !Kind::TEXT.accepts(&ParamValue::Flag));
        assert!(Kind::ARRAY.accepts(&ParamValue::Json("[1]".into())));
        assert!(!Kind::OBJECT.accepts(&ParamValue::Json("[1]".into())));
        assert!(Kind::ANY.accepts(&ParamValue::Flag));
        assert!(
            Kind::NUMBER.accepts(&ParamValue::String(vec![StringPart::Interpolation(
                crate::tree::Interpolation::default()
            )]))
        );
        assert!(Kind::SPACES.accepts_text("s 2rem") && !Kind::SPACES.accepts_text("s m l"));
        assert!(!Kind::SPACE.accepts_text("s l"), "one space only");
    }

    #[test]
    fn every_type_name_has_its_kind() {
        assert_eq!(TypeName::Space.kind(), Kind::SPACE);
        assert_eq!(TypeName::Areas.kind(), Kind::AREAS);
        let members = DeclaredType::Enum {
            members: vec!["a".into(), "b".into()],
        };
        assert_eq!(Kind::declared(&members).tokens.as_ref(), ["a", "b"]);
        let odd = DeclaredType::named("odd");
        assert_eq!(Kind::declared(&odd).forms, Kind::ANY.forms);
    }
}
