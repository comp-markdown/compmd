//! What a binding of props takes: a component's first parameter, or the variable `$props()` is given to.

use compmd::analysis::{Prop, Props};
use oxc_ast::ast::{BindingPattern, Expression, ObjectPattern, Program, TSTypeAnnotation};
use serde_json::Value;

use crate::types::{key_name, members_of};

/// What `pattern`, typed by `annotation`, takes. A type the file states lists every prop; without one, the names
/// destructured are the props, and a `...rest` takes any others. `None` when neither says: an untyped `props`. A prop
/// the pattern gives a default is optional, with that default when it is a literal.
pub(crate) fn props_of(
    program: &Program<'_>,
    pattern: &BindingPattern<'_>,
    annotation: Option<&TSTypeAnnotation<'_>>,
) -> Option<Props> {
    let object = match pattern {
        BindingPattern::ObjectPattern(object) => Some(&**object),
        _ => None,
    };
    if let Some(typed) =
        annotation.and_then(|annotation| members_of(program, &annotation.type_annotation))
    {
        return Some(Props {
            props: with_defaults(typed, object),
            rest: false,
        });
    }
    let object = object?;
    let props: Option<Vec<Prop>> = object
        .properties
        .iter()
        .map(|property| {
            let name = (!property.computed)
                .then(|| key_name(&property.key))
                .flatten()?;
            Some(Prop {
                name,
                ..Prop::default()
            })
        })
        .collect();
    Some(Props {
        props: with_defaults(props?, Some(object)),
        rest: object.rest.is_some(),
    })
}

/// `props` with the defaults `object` destructures them with.
fn with_defaults(mut props: Vec<Prop>, object: Option<&ObjectPattern<'_>>) -> Vec<Prop> {
    for property in object.into_iter().flat_map(|object| &object.properties) {
        let BindingPattern::AssignmentPattern(assignment) = &property.value else {
            continue;
        };
        let Some(name) = (!property.computed)
            .then(|| key_name(&property.key))
            .flatten()
        else {
            continue;
        };
        if let Some(prop) = props.iter_mut().find(|prop| prop.name == name) {
            prop.optional = true;
            prop.default = literal(&assignment.right);
        }
    }
    props
}

/// The value `expression` writes when it is a literal: a string, a number (negative too), a boolean, a template
/// without expressions.
fn literal(expression: &Expression<'_>) -> Option<Value> {
    match expression.without_parentheses() {
        Expression::StringLiteral(string) => Some(Value::String(string.value.to_string())),
        Expression::BooleanLiteral(boolean) => Some(Value::Bool(boolean.value)),
        Expression::NumericLiteral(number) => number_value(number.value),
        Expression::UnaryExpression(unary) if unary.operator.as_str() == "-" => {
            match unary.argument.without_parentheses() {
                Expression::NumericLiteral(number) => number_value(-number.value),
                _ => None,
            }
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .map(|text| Value::String(text.to_string())),
        _ => None,
    }
}

/// `number` as JSON: an integer when it is one.
fn number_value(number: f64) -> Option<Value> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "only a whole number within i64's range is cast"
    )]
    let whole = (number.fract() == 0.0 && number.abs() < 9.0e15).then_some(number as i64);
    whole.map_or_else(
        || serde_json::Number::from_f64(number).map(Value::Number),
        |whole| Some(Value::from(whole)),
    )
}
