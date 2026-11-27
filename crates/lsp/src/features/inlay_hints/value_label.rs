//! An evaluated value as an inlay hint's label ([a document drawn in the editor], decision 9): `= ` and the value as
//! interpolation writes it into text (spec §8.1, by the processor), short enough to sit beside the text.
//!
//! - A string is itself, a number as JavaScript's `String(n)`, a boolean `true` or `false`, a map with a string `name`
//!   its name, and `null` nothing: the label is `=` alone.
//! - A list is its size, `[3 items]`, and a map without a `name` its keys', `{2 keys}`.
//! - A value longer than [`LONGEST`] code points is cut to that, ending in `…`; a control character, a line break
//!   among them, is a space.
//! - What the render evaluated more than once shows its first value and the count: `= Ada, … (3)`.
//!
//! The tooltip holds what the label doesn't show: the whole text of a value cut short, a list's or a map's JSON, and
//! `null`; each at most [`LONGEST_TOOLTIP`] code points.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::cel::Value;
use compmd::evaluations::Evaluation;

/// The most code points a label's value takes, its `…` among them.
pub(crate) const LONGEST: usize = 40;

/// The most code points a tooltip takes, its `…` among them.
pub(crate) const LONGEST_TOOLTIP: usize = 1000;

/// What an inlay hint shows of a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValueLabel {
    /// The label: `= ` and the value.
    pub(crate) label: String,
    /// What the label doesn't show of the value.
    pub(crate) tooltip: Option<String>,
}

impl ValueLabel {
    /// The label of `evaluation`; `None` for one that failed, which its diagnostic explains.
    pub(crate) fn of(evaluation: &Evaluation) -> Option<Self> {
        let value = evaluation.value.as_ref()?;
        let (shown, tooltip) = shown(value);
        let shown = one_line(&shortened(&shown, LONGEST));
        let label = match evaluation.count {
            1 => format!("= {shown}"),
            count => format!("= {shown}, … ({count})"),
        };
        Some(Self {
            label: label.trim_end().to_owned(),
            tooltip: tooltip.map(|tooltip| shortened(&tooltip, LONGEST_TOOLTIP)),
        })
    }
}

/// What a label shows of `value`, and the tooltip with what it doesn't.
fn shown(value: &Value) -> (String, Option<String>) {
    let json = || Some(value.to_json().to_string());
    match (value, value.written()) {
        (Value::List(items), _) => (counted(items.len(), "item"), json()),
        (Value::Map(_), Ok(Some(name))) => (name, json()),
        (Value::Map(entries), _) => (counted(entries.len(), "key"), json()),
        (_, Ok(Some(text))) if text.chars().count() > LONGEST => (text.clone(), Some(text)),
        (_, Ok(Some(text))) => (text, None),
        (_, _) => (String::new(), Some("null".to_owned())),
    }
}

/// A list's or a map's size: `[3 items]`, `{1 key}`.
fn counted(count: usize, noun: &str) -> String {
    let plural = if count == 1 { "" } else { "s" };
    let (open, close) = if noun == "item" {
        ('[', ']')
    } else {
        ('{', '}')
    };
    format!("{open}{count} {noun}{plural}{close}")
}

/// `text`, cut to `longest` code points with a `…` when it is longer.
fn shortened(text: &str, longest: usize) -> String {
    if text.chars().count() <= longest {
        return text.to_owned();
    }
    let mut cut: String = text.chars().take(longest - 1).collect();
    cut.push('…');
    cut
}

/// `text` on one line: each control character a space.
fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use compmd::diagnostic::Location;
    use serde_json::json;

    use super::*;

    fn evaluated(value: Option<Value>, count: usize) -> Evaluation {
        Evaluation {
            at: Location::new(1, 1),
            value,
            count,
        }
    }

    fn label(value: &serde_json::Value) -> ValueLabel {
        ValueLabel::of(&evaluated(Some(Value::from_json(value)), 1)).expect("a label")
    }

    fn plain(label: &str) -> ValueLabel {
        ValueLabel {
            label: label.to_owned(),
            tooltip: None,
        }
    }

    #[test]
    fn a_value_is_written_as_interpolation_writes_it() {
        assert_eq!(label(&json!("Harbour notes")), plain("= Harbour notes"));
        assert_eq!(label(&json!(12.5)), plain("= 12.5"));
        assert_eq!(label(&json!(3)), plain("= 3"));
        assert_eq!(label(&json!(1e21)), plain("= 1e+21"));
        assert_eq!(label(&json!(true)), plain("= true"));
        let int = ValueLabel::of(&evaluated(Some(Value::Int(-4)), 1));
        assert_eq!(int, Some(plain("= -4")));
        assert_eq!(
            label(&json!(null)),
            ValueLabel {
                label: "=".into(),
                tooltip: Some("null".into())
            },
            "null writes nothing"
        );
        assert_eq!(label(&json!("")), plain("="));
    }

    #[test]
    fn a_record_is_its_name_and_other_lists_and_maps_their_size() {
        let author = json!({ "name": "Ursula K. Le Guin", "born": 1929 });
        assert_eq!(
            label(&author),
            ValueLabel {
                label: "= Ursula K. Le Guin".into(),
                tooltip: Some(author.to_string())
            }
        );
        assert_eq!(
            label(&json!(["Ana", "Kenji", "Priya"])),
            ValueLabel {
                label: "= [3 items]".into(),
                tooltip: Some(r#"["Ana","Kenji","Priya"]"#.into())
            }
        );
        assert_eq!(label(&json!([1])).label, "= [1 item]");
        assert_eq!(label(&json!([])).label, "= [0 items]");
        assert_eq!(label(&json!({ "a": 1, "b": 2 })).label, "= {2 keys}");
        assert_eq!(label(&json!({ "a": 1 })).label, "= {1 key}");
        assert_eq!(
            label(&json!({ "name": 3 })).label,
            "= {1 key}",
            "a name that isn't a string"
        );
    }

    #[test]
    fn a_long_value_is_cut_and_whole_in_the_tooltip() {
        let long = "a".repeat(LONGEST + 5);
        let cut = label(&json!(long));
        assert_eq!(cut.label, format!("= {}…", "a".repeat(LONGEST - 1)));
        assert_eq!(cut.tooltip, Some(long));
        let exact = "b".repeat(LONGEST);
        assert_eq!(label(&json!(exact.clone())), plain(&format!("= {exact}")));
        let huge = label(&json!("c".repeat(LONGEST_TOOLTIP * 2)));
        let tooltip = huge.tooltip.expect("a tooltip");
        assert_eq!(tooltip.chars().count(), LONGEST_TOOLTIP);
        assert!(tooltip.ends_with('…'));
    }

    #[test]
    fn a_label_is_on_one_line() {
        assert_eq!(
            label(&json!("Line one\nline two")),
            plain("= Line one line two")
        );
    }

    #[test]
    fn a_value_evaluated_more_than_once_shows_the_first_and_the_count() {
        let first = Some(Value::string("Ada"));
        assert_eq!(
            ValueLabel::of(&evaluated(first, 3)),
            Some(plain("= Ada, … (3)"))
        );
    }

    #[test]
    fn a_failure_has_no_label() {
        assert_eq!(ValueLabel::of(&evaluated(None, 2)), None);
    }
}
