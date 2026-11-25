//! What a React component (JSX, TSX) takes: its default export's first parameter.

use compmd::analysis::Props;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, Expression, FormalParameters, Program, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::binding::props_of;

/// The parameters of the function `expression` is: an arrow, a function, one in parentheses, or one the file names.
fn function_of<'p, 'a>(
    program: &'p Program<'a>,
    expression: &'p Expression<'a>,
) -> Option<&'p FormalParameters<'a>> {
    match expression.without_parentheses() {
        Expression::ArrowFunctionExpression(arrow) => Some(&arrow.params),
        Expression::FunctionExpression(function) => Some(&function.params),
        Expression::Identifier(identifier) => named(program, &identifier.name),
        _ => None,
    }
}

/// The parameters of the function the file declares at its top level as `name`: a function, or a variable whose
/// value is one.
fn named<'p, 'a>(program: &'p Program<'a>, name: &str) -> Option<&'p FormalParameters<'a>> {
    for statement in &program.body {
        let declaration = match statement {
            Statement::FunctionDeclaration(function) => {
                if function.id.as_ref().is_some_and(|id| id.name == name) {
                    return Some(&function.params);
                }
                continue;
            }
            Statement::VariableDeclaration(variables) => variables,
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::FunctionDeclaration(function)) => {
                    if function.id.as_ref().is_some_and(|id| id.name == name) {
                        return Some(&function.params);
                    }
                    continue;
                }
                Some(Declaration::VariableDeclaration(variables)) => variables,
                _ => continue,
            },
            _ => continue,
        };
        for declarator in &declaration.declarations {
            if declarator
                .id
                .get_identifier_name()
                .is_some_and(|id| id == name)
                && let Some(init) = &declarator.init
            {
                return function_of(program, init);
            }
        }
    }
    None
}

/// The parameters of the function the file exports as its default.
fn default_export<'p, 'a>(program: &'p Program<'a>) -> Option<&'p FormalParameters<'a>> {
    program.body.iter().find_map(|statement| match statement {
        Statement::ExportDefaultDeclaration(export) => match &export.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(function) => Some(&*function.params),
            declaration => function_of(program, declaration.as_expression()?),
        },
        _ => None,
    })
}

/// What the React component whose source is `code` takes; `None` when it can't be read.
pub(crate) fn props(code: &str, tsx: bool) -> Option<Props> {
    let allocator = Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::jsx()
    };
    let parsed = Parser::new(&allocator, code, source_type).parse();
    if parsed.panicked {
        return None;
    }
    let params = default_export(&parsed.program)?;
    let Some(first) = params.items.first() else {
        return Some(Props::default());
    };
    props_of(
        &parsed.program,
        &first.pattern,
        first.type_annotation.as_deref(),
    )
}
