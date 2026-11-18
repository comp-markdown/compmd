//! Param lists (spec §3.1, §5): `{key=value …}` on a component line, after an inline component and on a definition
//! fence, and the bare list of a continuation line. Indexes count code points of the line.

use crate::tree::{Interpolation, Literal, ParamValue, StringPart};

/// A param as written, with where it is on its line.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Written {
    pub(crate) key: Option<String>,
    pub(crate) breakpoint: Option<String>,
    pub(crate) value: ParamValue,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Written {
    /// See [`identity`].
    pub(crate) fn identity(&self) -> Option<(&str, Option<&str>)> {
        identity(self.key.as_deref(), self.breakpoint.as_deref(), &self.value)
    }
}

/// What makes two params the same for F017: the key and breakpoint, where `#name` is `id`; classes add up.
pub(crate) fn identity<'a>(
    key: Option<&'a str>,
    breakpoint: Option<&'a str>,
    value: &ParamValue,
) -> Option<(&'a str, Option<&'a str>)> {
    match value {
        ParamValue::Class(_) => None,
        ParamValue::Id(_) => Some(("id", None)),
        _ => Some((key.unwrap_or(""), breakpoint)),
    }
}

/// The params of a list, and where it ends or broke.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct List {
    /// Every param before the list ended or broke, repeats too.
    pub(crate) params: Vec<Written>,
    /// The index past the list: past its `}`, or the end of the text.
    pub(crate) end: usize,
    /// Where the list broke (F011): a delimiter that doesn't close, a character a bare value can't hold, or an `=`
    /// with no value.
    pub(crate) broke_at: Option<usize>,
}

/// The list whose `{` is at `open`.
pub(crate) fn braced(text: &[char], open: usize) -> List {
    Reader { text, braced: true }.list(open + 1, open)
}

/// The list from `start` to the end of the text, without braces: a continuation line's.
pub(crate) fn unbraced(text: &[char], start: usize) -> List {
    Reader {
        text,
        braced: false,
    }
    .list(start, start)
}

/// The length of the name (spec §3.1) at `at`.
pub(crate) fn name_len(text: &[char], at: usize) -> Option<usize> {
    if !text.get(at)?.is_ascii_alphabetic() {
        return None;
    }
    let rest = text[at + 1..]
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
        .count();
    Some(1 + rest)
}

/// The name at `at`.
pub(crate) fn name_at(text: &[char], at: usize) -> Option<String> {
    name_len(text, at).map(|len| text[at..at + len].iter().collect())
}

/// Whether `text` is a name, as a whole.
pub(crate) fn is_name(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    name_len(&chars, 0) == Some(chars.len())
}

pub(crate) fn is_space(c: Option<&char>) -> bool {
    matches!(c, Some(' ' | '\t'))
}

/// The interpolation `{{ path }}` at `at` (spec §8.1): its path and its length.
pub(crate) fn interpolation_at(text: &[char], at: usize) -> Option<(String, usize)> {
    if text.get(at..at + 2) != Some(&['{', '{']) {
        return None;
    }
    let skip_space = |mut i: usize| {
        while is_space(text.get(i)) {
            i += 1;
        }
        i
    };
    let path_start = skip_space(at + 2);
    let mut i = path_start;
    loop {
        if !text
            .get(i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_')
        {
            return None;
        }
        i += 1 + text[i + 1..]
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric() || **c == '_')
            .count();
        if text.get(i) != Some(&'.') {
            break;
        }
        i += 1;
    }
    let path = text[path_start..i].iter().collect();
    let close = skip_space(i);
    (text.get(close..close + 2) == Some(&['}', '}'])).then(|| (path, close + 2 - at))
}

/// Each interpolation in `text` as written, with the index where its path starts; `\{{` is no interpolation.
pub(crate) fn interpolations(text: &[char]) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if text[at] == '\\' {
            at += 2;
        } else if let Some((path, len)) = interpolation_at(text, at) {
            let path_start = at
                + 2
                + text[at + 2..]
                    .iter()
                    .take_while(|c| is_space(Some(c)))
                    .count();
            found.push((path_start, path));
            at += len;
        } else {
            at += 1;
        }
    }
    found
}

/// The text and interpolation parts of a URL or a data import's `from` (spec §12.2).
pub(crate) fn string_parts(text: &str) -> Vec<StringPart> {
    let chars: Vec<char> = text.chars().collect();
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut at = 0;
    while at < chars.len() {
        if let Some((path, len)) = interpolation_at(&chars, at) {
            flush(&mut parts, &mut buffer);
            parts.push(inline_interpolation(path));
            at += len;
        } else {
            buffer.push(chars[at]);
            at += 1;
        }
    }
    flush(&mut parts, &mut buffer);
    parts
}

fn inline_interpolation(path: String) -> StringPart {
    StringPart::Interpolation(Interpolation {
        path,
        block: false,
        position: None,
    })
}

fn flush(parts: &mut Vec<StringPart>, buffer: &mut String) {
    if !buffer.is_empty() {
        parts.push(StringPart::Text(Literal {
            value: std::mem::take(buffer),
            position: None,
        }));
    }
}

/// Where a list broke.
struct Broke(usize);

struct Reader<'a> {
    text: &'a [char],
    braced: bool,
}

impl Reader<'_> {
    /// The params from `start`; a braced list that doesn't close breaks at `open`.
    fn list(&self, start: usize, open: usize) -> List {
        let mut params = Vec::new();
        let mut at = start;
        let ended = loop {
            while is_space(self.text.get(at)) {
                at += 1;
            }
            match self.text.get(at) {
                None if self.braced => break Err(Broke(open)),
                None => break Ok(at),
                Some('}') if self.braced => break Ok(at + 1),
                Some(_) => {}
            }
            match self.param(at) {
                Ok(param) if self.ends_param(param.end) => {
                    at = param.end;
                    params.push(param);
                }
                Ok(param) => break Err(Broke(param.end)),
                Err(broke) => break Err(broke),
            }
        };
        match ended {
            Ok(end) => List {
                params,
                end,
                broke_at: None,
            },
            Err(Broke(at)) => List {
                params,
                end: self.text.len(),
                broke_at: Some(at),
            },
        }
    }

    fn ends_param(&self, at: usize) -> bool {
        let c = self.text.get(at);
        c.is_none() || is_space(c) || (self.braced && c == Some(&'}'))
    }

    fn param(&self, at: usize) -> Result<Written, Broke> {
        let written = |key, breakpoint, value, end| Written {
            key,
            breakpoint,
            value,
            start: at,
            end,
        };
        if let marker @ ('.' | '#') = self.text[at] {
            let name = name_at(self.text, at + 1).ok_or(Broke(at))?;
            let end = at + 1 + name.len();
            let value = if marker == '.' {
                ParamValue::Class(name)
            } else {
                ParamValue::Id(name)
            };
            return Ok(written(None, None, value, end));
        }
        let mut key = name_at(self.text, at).ok_or(Broke(at))?;
        let mut next = at + key.len();
        let mut breakpoint = None;
        if self.text.get(next) == Some(&':') {
            let name = name_at(self.text, next + 1).ok_or(Broke(next))?;
            next += 1 + name.len();
            breakpoint = Some(std::mem::replace(&mut key, name));
        }
        if self.text.get(next) != Some(&'=') {
            return Ok(written(Some(key), breakpoint, ParamValue::Flag, next));
        }
        let (value, end) = self.value(next + 1, next)?;
        Ok(written(Some(key), breakpoint, value, end))
    }

    /// The value at `at`, after the `=` at `equals`.
    fn value(&self, at: usize, equals: usize) -> Result<(ParamValue, usize), Broke> {
        match self.text.get(at) {
            Some('"') => self.string(at),
            Some('(') => {
                let end = self.expression_end(at)?;
                Ok((ParamValue::Expr(self.slice(at + 1, end - 1)), end))
            }
            Some('[' | '{') => {
                let end = self.json_end(at)?;
                let json = self.slice(at, end);
                serde_json::from_str::<serde_json::Value>(&json).map_err(|_| Broke(at))?;
                Ok((ParamValue::Json(json), end))
            }
            _ if self.ends_param(at) => Err(Broke(equals)),
            _ => {
                let mut end = at;
                while !self.ends_param(end) {
                    if matches!(self.text[end], '"' | '(' | ')' | '[' | ']' | '{' | '}') {
                        return Err(Broke(end));
                    }
                    end += 1;
                }
                Ok((ParamValue::Bare(self.slice(at, end)), end))
            }
        }
    }

    fn slice(&self, start: usize, end: usize) -> String {
        self.text[start..end].iter().collect()
    }

    /// A quoted string: escapes of ASCII punctuation resolved, interpolations as parts.
    fn string(&self, open: usize) -> Result<(ParamValue, usize), Broke> {
        let mut parts = Vec::new();
        let mut buffer = String::new();
        let mut at = open + 1;
        while let Some(&c) = self.text.get(at) {
            if c == '\\'
                && self
                    .text
                    .get(at + 1)
                    .is_some_and(char::is_ascii_punctuation)
            {
                buffer.push(self.text[at + 1]);
                at += 2;
            } else if c == '"' {
                flush(&mut parts, &mut buffer);
                return Ok((ParamValue::String(parts), at + 1));
            } else if let Some((path, len)) = interpolation_at(self.text, at) {
                flush(&mut parts, &mut buffer);
                parts.push(inline_interpolation(path));
                at += len;
            } else {
                buffer.push(c);
                at += 1;
            }
        }
        Err(Broke(open))
    }

    /// The index past the `)` that matches the `(` at `open`, counting outside CEL string and bytes literals.
    fn expression_end(&self, open: usize) -> Result<usize, Broke> {
        let mut depth = 0;
        let mut at = open;
        while let Some(&c) = self.text.get(at) {
            match c {
                '"' | '\'' => at = self.cel_string_end(at).ok_or(Broke(open))?,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(at + 1);
                    }
                }
                _ => {}
            }
            at += 1;
        }
        Err(Broke(open))
    }

    /// The index of the quote that closes the CEL literal opening at `at`: single or triple quoted, raw (`r"…"`,
    /// no escapes) or not.
    fn cel_string_end(&self, at: usize) -> Option<usize> {
        let quote = self.text[at];
        let raw = at > 0 && matches!(self.text[at - 1], 'r' | 'R');
        let triple = self.text.get(at + 1) == Some(&quote) && self.text.get(at + 2) == Some(&quote);
        let mut i = at + if triple { 3 } else { 1 };
        while let Some(&c) = self.text.get(i) {
            if c == '\\' && !raw {
                i += 2;
                continue;
            }
            if c == quote
                && (!triple
                    || (self.text.get(i + 1) == Some(&quote)
                        && self.text.get(i + 2) == Some(&quote)))
            {
                return Some(if triple { i + 2 } else { i });
            }
            i += 1;
        }
        None
    }

    /// The index past the bracket that matches the JSON opener at `open`, counting outside JSON strings.
    fn json_end(&self, open: usize) -> Result<usize, Broke> {
        let mut depth = 0;
        let mut at = open;
        while let Some(&c) = self.text.get(at) {
            match c {
                '"' => {
                    at += 1;
                    while self.text.get(at).is_some_and(|c| *c != '"') {
                        at += if self.text[at] == '\\' { 2 } else { 1 };
                    }
                }
                '[' | '{' => depth += 1,
                ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(at + 1);
                    }
                }
                _ => {}
            }
            at += 1;
        }
        Err(Broke(open))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn values(text: &str) -> Vec<ParamValue> {
        let list = braced(&chars(text), 0);
        assert_eq!(list.broke_at, None, "{text}");
        list.params.into_iter().map(|param| param.value).collect()
    }

    fn broke_at(text: &str) -> Option<usize> {
        braced(&chars(text), 0).broke_at
    }

    fn text(value: &str) -> StringPart {
        StringPart::Text(Literal {
            value: value.into(),
            position: None,
        })
    }

    #[test]
    fn reads_every_kind() {
        assert_eq!(
            values(r#"{icon=rocket title="A \"b\"" n=(f(1)) tags=["a"] open .wide #intro}"#),
            [
                ParamValue::Bare("rocket".into()),
                ParamValue::String(vec![text(r#"A "b""#)]),
                ParamValue::Expr("f(1)".into()),
                ParamValue::Json(r#"["a"]"#.into()),
                ParamValue::Flag,
                ParamValue::Class("wide".into()),
                ParamValue::Id("intro".into()),
            ]
        );
    }

    #[test]
    fn reads_breakpoints_and_positions() {
        let list = braced(&chars("{ md:gap=l }"), 0);
        let param = &list.params[0];
        assert_eq!(
            (param.key.as_deref(), param.breakpoint.as_deref()),
            (Some("gap"), Some("md"))
        );
        assert_eq!((param.start, param.end, list.end), (2, 10, 12));
    }

    #[test]
    fn strings_hold_interpolations_and_keep_other_backslashes() {
        assert_eq!(
            values(r#"{a="Step {{ n }}!" b="C:\dir" c="{{a b}}"}"#),
            [
                ParamValue::String(vec![
                    text("Step "),
                    inline_interpolation("n".into()),
                    text("!")
                ]),
                ParamValue::String(vec![text(r"C:\dir")]),
                ParamValue::String(vec![text("{{a b}}")]),
            ]
        );
        assert_eq!(values(r#"{a=""}"#), [ParamValue::String(Vec::new())]);
    }

    #[test]
    fn expressions_count_parentheses_outside_cel_literals() {
        assert_eq!(
            values(r#"{a=(f(")") + '(' + """)""" + r"\" + b'\'')}"#),
            [ParamValue::Expr(
                r#"f(")") + '(' + """)""" + r"\" + b'\''"#.into()
            )]
        );
    }

    #[test]
    fn json_counts_brackets_outside_strings() {
        assert_eq!(
            values(r#"{a={"b":"}"}}"#),
            [ParamValue::Json(r#"{"b":"}"}"#.into())]
        );
        assert_eq!(
            values(r#"{a=["\"]"]}"#),
            [ParamValue::Json(r#"["\"]"]"#.into())]
        );
    }

    #[test]
    fn breaks_where_spec_10_2_says() {
        assert_eq!(broke_at(r#"{title="x}"#), Some(7));
        assert_eq!(broke_at("{a=(b}"), Some(3));
        assert_eq!(broke_at("{a=[1}"), Some(3));
        assert_eq!(broke_at("{a=[1,]}"), Some(3));
        assert_eq!(broke_at("{a=repeat(3)}"), Some(9));
        assert_eq!(broke_at("{a= b}"), Some(2));
        assert_eq!(broke_at("{a=}"), Some(2));
        assert_eq!(broke_at("{a=1"), Some(0));
        assert_eq!(broke_at(r#"{a="b"c}"#), Some(6));
        assert_eq!(broke_at("{md:=1}"), Some(3));
        assert_eq!(broke_at("{. x}"), Some(1));
        assert_eq!(broke_at("{=x}"), Some(1));
        assert_eq!(broke_at(r#"{a=(f("x)}"#), Some(3));
        assert_eq!(broke_at("{a=[[1]}"), Some(3));
        assert_eq!(broke_at(r#"{a=["x}"#), Some(3));
    }

    #[test]
    fn keeps_params_before_a_break() {
        let list = braced(&chars("{a=1 b=(}"), 0);
        assert_eq!(list.params.len(), 1);
        assert_eq!(list.end, 9);
    }

    #[test]
    fn unbraced_lists_run_to_the_end_of_the_line() {
        let list = unbraced(&chars(" a=1  b"), 0);
        assert_eq!(list.params.len(), 2);
        assert_eq!(list.end, 7);
        assert_eq!(unbraced(&chars("a=}"), 0).broke_at, Some(2));
    }

    #[test]
    fn identity_is_key_and_breakpoint_and_id_for_hash_names() {
        let list = braced(&chars("{id=a #b .c md:gap=s gap=s}"), 0);
        let identities: Vec<_> = list.params.iter().map(Written::identity).collect();
        assert_eq!(
            identities,
            [
                Some(("id", None)),
                Some(("id", None)),
                None,
                Some(("gap", Some("md"))),
                Some(("gap", None))
            ]
        );
    }

    #[test]
    fn interpolations_are_paths_of_identifiers() {
        let at = |text: &str| interpolation_at(&chars(text), 0);
        assert_eq!(at("{{ md.doc.title }}x"), Some(("md.doc.title".into(), 18)));
        assert_eq!(at("{{_a1}}"), Some(("_a1".into(), 7)));
        for text in ["{{1}}", "{{a b}}", "{{a.}}", "{{a}", "{a}}", "{{a-b}}"] {
            assert_eq!(at(text), None, "{text}");
        }
    }

    #[test]
    fn urls_split_into_text_and_interpolations() {
        assert_eq!(
            string_parts("./{{md.doc.slug}}.html"),
            [
                text("./"),
                inline_interpolation("md.doc.slug".into()),
                text(".html")
            ]
        );
        assert_eq!(string_parts(""), []);
    }

    #[test]
    fn names_start_with_a_letter() {
        assert!(is_name("habit-heatmap_2"));
        assert!(!is_name("2x"));
        assert!(!is_name("a b"));
        assert!(!is_name(""));
    }
}
