//! Where a JSON object's members are in its text, so the server can add members to a file (a project's configuration)
//! without writing the rest of it again: its order and its spacing stay as their writer left them.

/// A member of a JSON object, as it is written.
#[derive(Debug, PartialEq, Eq)]
struct Member {
    /// Its key.
    key: String,
    /// The byte offset of its key's opening quote.
    at: usize,
    /// The byte offset of its value.
    value: usize,
}

/// A JSON object in a text.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct JsonObject {
    /// The byte offset of its `{`.
    open: usize,
    /// Its members, in order.
    members: Vec<Member>,
}

/// The offset of the first byte from `from` on that isn't JSON's white space.
fn skip_space(bytes: &[u8], from: usize) -> usize {
    let mut at = from;
    while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
        at += 1;
    }
    at
}

/// The offset just past the string whose opening quote is at `at`.
fn string_end(bytes: &[u8], at: usize) -> Option<usize> {
    if bytes.get(at) != Some(&b'"') {
        return None;
    }
    let mut i = at + 1;
    loop {
        match bytes.get(i)? {
            b'\\' => i += 2,
            b'"' => return Some(i + 1),
            _ => i += 1,
        }
    }
}

/// The offset just past the value at `at`: a string, an object, an array, or a number, `true`, `false` or `null`.
fn value_end(bytes: &[u8], at: usize) -> Option<usize> {
    match bytes.get(at)? {
        b'"' => string_end(bytes, at),
        b'{' | b'[' => {
            let (mut depth, mut i) = (0_usize, at);
            loop {
                match bytes.get(i)? {
                    b'"' => {
                        i = string_end(bytes, i)?;
                        continue;
                    }
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(i + 1);
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }
        _ => {
            let mut i = at;
            while bytes.get(i).is_some_and(|byte| {
                !matches!(byte, b',' | b'}' | b']') && !byte.is_ascii_whitespace()
            }) {
                i += 1;
            }
            Some(i)
        }
    }
}

impl JsonObject {
    /// The object `text` is; `None` when it isn't one.
    pub(crate) fn top(text: &str) -> Option<Self> {
        Self::at(text, skip_space(text.as_bytes(), 0))
    }

    /// The object that the value of its member `key` is; `None` when it has no such member, or its value isn't an
    /// object.
    pub(crate) fn member(&self, text: &str, key: &str) -> Option<Self> {
        let member = self.members.iter().find(|member| member.key == key)?;
        Self::at(text, member.value)
    }

    /// Where to insert `members` (each written `"key": value`) as its first ones, and the text to insert there: before
    /// its first member, each followed by the spacing that precedes that member; in an empty object, each on a line
    /// of its own.
    pub(crate) fn adding(&self, text: &str, members: &[String]) -> (usize, String) {
        match self.members.first() {
            Some(first) => {
                let spacing = &text[self.open + 1..first.at];
                let written = members.join(&format!(",{spacing}"));
                (first.at, format!("{written},{spacing}"))
            }
            None => (self.open + 1, format!("\n  {}\n", members.join(",\n  "))),
        }
    }

    /// The object whose `{` is at `open` in `text`.
    fn at(text: &str, open: usize) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.get(open) != Some(&b'{') {
            return None;
        }
        let mut members = Vec::new();
        let mut i = skip_space(bytes, open + 1);
        if bytes.get(i) == Some(&b'}') {
            return Some(Self { open, members });
        }
        loop {
            let at = i;
            let key_end = string_end(bytes, at)?;
            let key = serde_json::from_str(&text[at..key_end]).ok()?;
            i = skip_space(bytes, key_end);
            if bytes.get(i) != Some(&b':') {
                return None;
            }
            let value = skip_space(bytes, i + 1);
            members.push(Member { key, at, value });
            i = skip_space(bytes, value_end(bytes, value)?);
            match bytes.get(i)? {
                b',' => i = skip_space(bytes, i + 1),
                b'}' => return Some(Self { open, members }),
                _ => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `text` with `inserted` put in at `at`.
    fn inserted(text: &str, (at, inserted): (usize, String)) -> String {
        format!("{}{inserted}{}", &text[..at], &text[at..])
    }

    #[test]
    fn an_objects_members_are_found_past_strings_objects_arrays_and_scalars() {
        let text = r#" { "a": "x \" }", "b": {"c": [1, {"d": "]"}]}, "e": -1.5e3, "f": true }"#;
        let top = JsonObject::top(text).expect("an object");
        let keys: Vec<&str> = top
            .members
            .iter()
            .map(|member| member.key.as_str())
            .collect();
        assert_eq!(keys, ["a", "b", "e", "f"]);
        let b = top.member(text, "b").expect("an object");
        assert_eq!(b.members[0].key, "c");
        assert_eq!(top.member(text, "a"), None, "a string");
        assert_eq!(top.member(text, "z"), None);
        assert_eq!(JsonObject::top("{}").expect("empty").members, []);
    }

    #[test]
    fn what_isnt_an_object_is_none() {
        for text in [
            "[]",
            "",
            "{\"a\" 1}",
            "{\"a\": 1 \"b\": 2}",
            "{\"a\": \"open",
            "{\"a\": [1",
        ] {
            assert_eq!(JsonObject::top(text), None, "{text}");
        }
    }

    #[test]
    fn members_are_added_first_with_the_spacing_the_object_has() {
        let text = "{\n  \"fetch\": \"host\"\n}\n";
        let top = JsonObject::top(text).expect("an object");
        let added = top.adding(text, &["\"a\": 1".into(), "\"b\": 2".into()]);
        assert_eq!(
            inserted(text, added),
            "{\n  \"a\": 1,\n  \"b\": 2,\n  \"fetch\": \"host\"\n}\n"
        );
        let empty = "{}";
        let added = JsonObject::top(empty)
            .expect("an object")
            .adding(empty, &["\"a\": 1".into()]);
        assert_eq!(inserted(empty, added), "{\n  \"a\": 1\n}");
    }
}
