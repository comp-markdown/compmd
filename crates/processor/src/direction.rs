//! A document's language and direction for the expanded tree's root (spec §2.1, §12.3): `md.lang` as written, and
//! `md.dir` as written or, when it isn't, the direction of the language's script, from CLDR.

use crate::settings::Settings;

/// The scripts written right to left (ISO 15924), as a language tag's script subtag names them.
const RTL_SCRIPTS: [&str; 10] = [
    "arab", "adlm", "hebr", "mand", "nkoo", "rohg", "samr", "syrc", "thaa", "yezi",
];

/// The languages whose script, unless their tag names another, is written right to left (CLDR's likely scripts).
const RTL_LANGUAGES: [&str; 18] = [
    "ar", "arc", "ckb", "dv", "fa", "he", "iw", "ks", "lrc", "mzn", "nqo", "pnb", "ps", "sd",
    "syr", "ug", "ur", "yi",
];

/// The direction text in `lang` is written in: `rtl` for Arabic, Hebrew, Persian, Urdu, …, `ltr` otherwise.
///
/// ```ignore
/// assert_eq!(of("ar-EG"), "rtl");
/// assert_eq!(of("sr-Latn"), "ltr");
/// ```
pub(crate) fn of(lang: &str) -> &'static str {
    let lower = lang.to_ascii_lowercase();
    let mut subtags = lower.split(['-', '_']);
    let language = subtags.next().unwrap_or_default();
    let script = subtags.next().filter(|subtag| subtag.len() == 4);
    let rtl = match script {
        Some(script) => RTL_SCRIPTS.contains(&script),
        None => RTL_LANGUAGES.contains(&language),
    };
    if rtl { "rtl" } else { "ltr" }
}

/// The root's `lang` and `dir` for a document with `settings`: each only when the document writes `md.lang` or
/// `md.dir`, so a document that writes neither keeps its host's; `dir` from `lang` when only `lang` is written.
pub(crate) fn of_document(settings: &Settings) -> (Option<String>, Option<String>) {
    let written = settings
        .dir
        .as_deref()
        .filter(|dir| ["ltr", "rtl", "auto"].contains(dir));
    let dir = written
        .map(str::to_string)
        .or_else(|| settings.lang.as_deref().map(|lang| of(lang).to_string()));
    (settings.lang.clone(), dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_has_its_scripts_direction() {
        for lang in ["ar", "ar-EG", "he", "fa-IR", "ur", "yi", "ps", "dv"] {
            assert_eq!(of(lang), "rtl", "{lang}");
        }
        for lang in ["en", "de-CH", "ja", "zh-Hant", "tr", "sw"] {
            assert_eq!(of(lang), "ltr", "{lang}");
        }
    }

    #[test]
    fn a_script_subtag_decides_over_the_language() {
        assert_eq!(of("pa-Arab"), "rtl");
        assert_eq!(of("az-Arab-IR"), "rtl");
        assert_eq!(of("uz-Latn"), "ltr");
        assert_eq!(of("ar-Latn"), "ltr");
    }

    #[test]
    fn a_document_gives_what_it_writes_and_dir_from_lang() {
        let with = |lang: Option<&str>, dir: Option<&str>| Settings {
            lang: lang.map(str::to_string),
            dir: dir.map(str::to_string),
            ..Settings::default()
        };
        assert_eq!(of_document(&with(None, None)), (None, None));
        assert_eq!(
            of_document(&with(Some("ar"), None)),
            (Some("ar".into()), Some("rtl".into()))
        );
        assert_eq!(
            of_document(&with(Some("ar"), Some("ltr"))),
            (Some("ar".into()), Some("ltr".into()))
        );
        assert_eq!(
            of_document(&with(None, Some("auto"))),
            (None, Some("auto".into()))
        );
        assert_eq!(of_document(&with(None, Some("sideways"))), (None, None));
    }
}
