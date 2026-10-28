//! What a component line is (spec §3.1), from its text. Indexes count code points of the line.

use super::params::{self, List, is_space, name_at};

/// The form of a line that starts with `::`.
#[derive(Debug, PartialEq)]
pub(crate) enum Form {
    /// `::: name {params}`
    Opener { name: String, params: List },
    /// `:: name[fallback] {params}`; the fallback's text is `line[start..end]`.
    Leaf {
        name: String,
        fallback: Option<(usize, usize)>,
        params: List,
    },
    /// `::: + params`
    Continuation { params: List },
    /// `::: /name`
    Closer { name: String },
    /// `:: #name`
    Slot { name: String },
    /// No form: F016, with the form the line comes closest to.
    Malformed { expected: &'static str },
}

const OPENER: &str = "`::: name {params}`";
const CLOSER: &str = "`::: /name`";
const CONTINUATION: &str = "`::: + params`";
const LEAF: &str = "`:: name[fallback] {params}`";
const SLOT: &str = "`:: #name`";

/// The form of `line`, which starts with `::`.
pub(crate) fn classify(line: &[char]) -> Form {
    if line.get(2) == Some(&':') {
        if is_space(line.get(3)) {
            container_line(line, skip_space(line, 3))
        } else {
            Form::Malformed { expected: OPENER }
        }
    } else if is_space(line.get(2)) {
        leaf_or_slot_line(line, skip_space(line, 2))
    } else {
        Form::Malformed { expected: LEAF }
    }
}

fn container_line(line: &[char], at: usize) -> Form {
    match line.get(at) {
        Some('/') => match name_at(line, at + 1) {
            Some(name) if blank_from(line, at + 1 + name.len()) => Form::Closer { name },
            _ => Form::Malformed { expected: CLOSER },
        },
        Some('+') if is_space(line.get(at + 1)) => Form::Continuation {
            params: params::unbraced(line, at + 1),
        },
        Some('+') => Form::Malformed {
            expected: CONTINUATION,
        },
        _ => {
            let Some(name) = name_at(line, at) else {
                return Form::Malformed { expected: OPENER };
            };
            let after = at + name.len();
            if blank_from(line, after) {
                return Form::Opener {
                    name,
                    params: List::default(),
                };
            }
            let brace = skip_space(line, after);
            if brace == after {
                return Form::Malformed { expected: OPENER };
            }
            match ending_list(line, brace) {
                Some(params) => Form::Opener { name, params },
                None => Form::Malformed { expected: OPENER },
            }
        }
    }
}

fn leaf_or_slot_line(line: &[char], at: usize) -> Form {
    if line.get(at) == Some(&'#') {
        return match name_at(line, at + 1) {
            Some(name) if blank_from(line, at + 1 + name.len()) => Form::Slot { name },
            _ => Form::Malformed { expected: SLOT },
        };
    }
    let Some(name) = name_at(line, at) else {
        return Form::Malformed { expected: LEAF };
    };
    let mut after = at + name.len();
    let mut fallback = None;
    if line.get(after) == Some(&'[') {
        let Some(close) = matching_bracket(line, after) else {
            return Form::Malformed { expected: LEAF };
        };
        fallback = Some((after + 1, close));
        after = close + 1;
    }
    if blank_from(line, after) {
        return Form::Leaf {
            name,
            fallback,
            params: List::default(),
        };
    }
    match ending_list(line, skip_space(line, after)) {
        Some(params) => Form::Leaf {
            name,
            fallback,
            params,
        },
        None => Form::Malformed { expected: LEAF },
    }
}

/// The param list at `brace`, which must end the line unless it broke (F011 then, not F016).
fn ending_list(line: &[char], brace: usize) -> Option<List> {
    if line.get(brace) != Some(&'{') {
        return None;
    }
    let list = params::braced(line, brace);
    (list.broke_at.is_some() || blank_from(line, list.end)).then_some(list)
}

/// The index of the `]` that matches the `[` at `open` (spec §3.2 rule 9): brackets nest, escaped ones don't count,
/// and a code span binds tighter than the brackets.
pub(crate) fn matching_bracket(text: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut at = open;
    while let Some(&c) = text.get(at) {
        match c {
            '\\' => at += 1,
            '`' => at = code_span_end(text, at) - 1,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// The index past the code span whose backtick run starts at `at`, or past the run when no run of its length
/// closes it.
fn code_span_end(text: &[char], at: usize) -> usize {
    let run = |from: usize| text[from..].iter().take_while(|c| **c == '`').count();
    let open = run(at);
    let mut i = at + open;
    while i < text.len() {
        let close = run(i);
        if close == open {
            return i + close;
        }
        i += close.max(1);
    }
    at + open
}

fn skip_space(line: &[char], mut at: usize) -> usize {
    while is_space(line.get(at)) {
        at += 1;
    }
    at
}

fn blank_from(line: &[char], at: usize) -> bool {
    skip_space(line, at) >= line.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::ParamValue;

    fn form(line: &str) -> Form {
        classify(&line.chars().collect::<Vec<_>>())
    }

    /// The values of a form's params.
    fn values(form: &Form) -> Vec<ParamValue> {
        match form {
            Form::Opener { params, .. }
            | Form::Leaf { params, .. }
            | Form::Continuation { params } => params
                .params
                .iter()
                .map(|param| param.value.clone())
                .collect(),
            _ => Vec::new(),
        }
    }

    #[test]
    fn openers_take_a_param_list_after_a_space() {
        assert_eq!(
            form("::: card"),
            Form::Opener {
                name: "card".into(),
                params: List::default()
            }
        );
        let spaced = form(":::  card   {open}  ");
        assert!(matches!(&spaced, Form::Opener { name, .. } if name == "card"));
        assert_eq!(values(&spaced), [ParamValue::Flag]);
        assert_eq!(form("::: card{open}"), Form::Malformed { expected: OPENER });
        assert_eq!(
            form("::: card {open} x"),
            Form::Malformed { expected: OPENER }
        );
        assert_eq!(form("::: card x"), Form::Malformed { expected: OPENER });
    }

    #[test]
    fn a_list_that_breaks_is_still_the_form() {
        assert!(matches!(
            form(r#"::: card {title="x} y"#),
            Form::Opener { params, .. } if params.broke_at == Some(16)
        ));
    }

    #[test]
    fn closers_continuations_and_slots() {
        assert_eq!(
            form("::: /card "),
            Form::Closer {
                name: "card".into()
            }
        );
        assert_eq!(form("::: /card x"), Form::Malformed { expected: CLOSER });
        assert_eq!(form("::: / card"), Form::Malformed { expected: CLOSER });
        let continuation = form("::: + md:gap=l");
        assert!(matches!(continuation, Form::Continuation { .. }));
        assert_eq!(values(&continuation), [ParamValue::Bare("l".into())]);
        assert_eq!(values(&form("::: /card")), []);
        assert_eq!(
            form("::: +gap=l"),
            Form::Malformed {
                expected: CONTINUATION
            }
        );
        assert_eq!(
            form(":: #footer"),
            Form::Slot {
                name: "footer".into()
            }
        );
        assert_eq!(form(":: #footer x"), Form::Malformed { expected: SLOT });
    }

    #[test]
    fn leaves_take_a_fallback_and_a_list() {
        let chart = form(":: chart[Weekly [x] `]`]{src=a.csv}");
        assert!(
            matches!(&chart, Form::Leaf { name, fallback: Some((9, 23)), .. } if name == "chart")
        );
        assert_eq!(values(&chart), [ParamValue::Bare("a.csv".into())]);
        assert_eq!(
            form(":: toc"),
            Form::Leaf {
                name: "toc".into(),
                fallback: None,
                params: List::default()
            }
        );
        assert_eq!(form(":: chart[open"), Form::Malformed { expected: LEAF });
        assert_eq!(form(":: chart[a] b"), Form::Malformed { expected: LEAF });
    }

    #[test]
    fn other_lines_are_malformed() {
        assert_eq!(form(":::card"), Form::Malformed { expected: OPENER });
        assert_eq!(form("::: 1card"), Form::Malformed { expected: OPENER });
        assert_eq!(form("::card"), Form::Malformed { expected: LEAF });
        assert_eq!(form(":: [x]"), Form::Malformed { expected: LEAF });
    }

    #[test]
    fn brackets_match_around_escapes_and_code_spans() {
        let close = |text: &str| matching_bracket(&text.chars().collect::<Vec<_>>(), 0);
        assert_eq!(close(r"[a\]b]"), Some(5));
        assert_eq!(close("[b `c]` d]"), Some(9));
        assert_eq!(close("[a ``b`` `c]"), Some(11));
        assert_eq!(close("[a `` b]"), Some(7));
        assert_eq!(close("[a"), None);
    }
}
