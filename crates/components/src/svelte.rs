//! What a Svelte 5 component takes: what its instance script gives `$props()`.

use compmd::analysis::Props;
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::binding::props_of;

/// Whether a `<script>` tag's attributes make it the module script, which runs once per module, not per instance:
/// `module`, or Svelte 4's `context="module"`.
fn is_module(attributes: &str) -> bool {
    attributes.split_whitespace().any(|attribute| {
        matches!(
            attribute,
            "module" | "context=\"module\"" | "context='module'"
        )
    })
}

/// The instance script of a component's source: the content of its first `<script>` that isn't the module script.
/// `Some("")` when it has none; `None` when a script is never closed.
fn instance_script(code: &str) -> Option<&str> {
    let mut rest = code;
    while let Some(start) = rest.find("<script") {
        let after = &rest[start + "<script".len()..];
        if !after.starts_with(|c: char| c.is_whitespace() || c == '>') {
            rest = after;
            continue;
        }
        let open = after.find('>')?;
        let body = &after[open + 1..];
        let close = body.find("</script>")?;
        if !is_module(&after[..open]) {
            return Some(&body[..close]);
        }
        rest = &body[close..];
    }
    Some("")
}

/// What the Svelte component whose source is `code` takes; `None` when it can't be read.
pub(crate) fn props(code: &str) -> Option<Props> {
    let script = instance_script(code)?;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return None;
    }
    for statement in &parsed.program.body {
        let Statement::VariableDeclaration(variables) = statement else {
            continue;
        };
        for declarator in &variables.declarations {
            let Some(Expression::CallExpression(call)) = &declarator.init else {
                continue;
            };
            if matches!(&call.callee, Expression::Identifier(callee) if callee.name == "$props") {
                return props_of(
                    &parsed.program,
                    &declarator.id,
                    declarator.type_annotation.as_deref(),
                );
            }
        }
    }
    Some(Props::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_instance_script_is_the_first_that_isnt_the_modules() {
        assert_eq!(instance_script("<script>let a;</script>"), Some("let a;"));
        assert_eq!(
            instance_script(
                "<script module>export const m = 1;</script>\n<script lang=\"ts\">let b;</script>"
            ),
            Some("let b;")
        );
        assert_eq!(
            instance_script("<script context=\"module\">x</script><script>let c;</script>"),
            Some("let c;")
        );
        assert_eq!(
            instance_script("<scripts>no</scripts><p>static</p>"),
            Some("")
        );
        assert_eq!(instance_script("<script>let d;"), None, "never closed");
        assert_eq!(instance_script("<p><script"), Some(""), "not a tag");
        assert_eq!(
            instance_script("<script lang=\"ts\""),
            None,
            "a tag never ended"
        );
    }
}
