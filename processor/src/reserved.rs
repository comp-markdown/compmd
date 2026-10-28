//! Names a document may not declare (F013): the keywords of a major version (spec §6.3), `md` (spec §8.1), and the
//! names expressions can't use as variables (spec §3.1).

use crate::named::named;

named! {
    /// A component no document may define, import, export or rename to, for a whole major version: so a name that
    /// is a keyword is always `@compmd/core`'s component.
    pub(crate) enum Keyword {
        If = "if",
        Each = "each",
        Value = "value",
        Box = "box",
    }
}

/// CEL's reserved words and type names: a definition's param, an `as` name or an input can't be one of them.
const CEL_NAMES: [&str; 34] = [
    "as",
    "break",
    "const",
    "continue",
    "else",
    "false",
    "for",
    "function",
    "if",
    "import",
    "in",
    "let",
    "loop",
    "namespace",
    "null",
    "package",
    "return",
    "true",
    "var",
    "void",
    "while",
    "bool",
    "bytes",
    "double",
    "duration",
    "dyn",
    "int",
    "list",
    "map",
    "null_type",
    "string",
    "timestamp",
    "type",
    "uint",
];

/// Why `name` can't name a component, or `None` when it can.
pub(crate) fn as_component(name: &str) -> Option<String> {
    if Keyword::from_name(name).is_some() {
        Some(format!(
            "`{name}` is a keyword of CompMD 1, so no document may define, import, export or rename to it"
        ))
    } else if name == "md" {
        Some("`md` is CompMD's own namespace, so no document may declare it".into())
    } else {
        None
    }
}

/// Why `name` can't be a variable of expressions (a definition's param, an `as` name, an input), or `None` when it
/// can.
pub(crate) fn as_variable(name: &str) -> Option<String> {
    if name == "md" {
        Some("`md` is CompMD's own namespace, so no document may declare it".into())
    } else if CEL_NAMES.contains(&name) {
        Some(format!(
            "`{name}` is a reserved word or type name of CEL, so expressions can't use it as a variable"
        ))
    } else {
        None
    }
}

/// Why `name` can't be an `as` name, which binds a variable of the template and may not be a keyword either.
pub(crate) fn as_binding(name: &str) -> Option<String> {
    as_variable(name).or_else(|| as_component(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_md_and_cel_names_are_refused_where_they_matter() {
        assert!(as_component("each").is_some());
        assert!(as_component("md").is_some());
        assert_eq!(as_component("card"), None);
        assert!(as_variable("type").is_some());
        assert!(as_variable("md").is_some());
        assert_eq!(as_variable("each"), None);
        assert!(as_binding("each").is_some());
        assert!(as_binding("int").is_some());
        assert_eq!(as_binding("t"), None);
    }
}
