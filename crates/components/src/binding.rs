//! What a binding of props takes: a component's first parameter, or the variable `$props()` is given to.

use compmd::analysis::{Prop, Props};
use oxc_ast::ast::{BindingPattern, Program, TSTypeAnnotation};

use crate::types::{key_name, members_of};

/// What `pattern`, typed by `annotation`, takes. A type the file states lists every prop; without one, the names
/// destructured are the props, and a `...rest` takes any others. `None` when neither says: an untyped `props`.
pub(crate) fn props_of(
    program: &Program<'_>,
    pattern: &BindingPattern<'_>,
    annotation: Option<&TSTypeAnnotation<'_>>,
) -> Option<Props> {
    if let Some(typed) =
        annotation.and_then(|annotation| members_of(program, &annotation.type_annotation))
    {
        return Some(Props {
            props: typed,
            rest: false,
        });
    }
    let BindingPattern::ObjectPattern(object) = pattern else {
        return None;
    };
    let props: Option<Vec<Prop>> = object
        .properties
        .iter()
        .map(|property| {
            let name = (!property.computed)
                .then(|| key_name(&property.key))
                .flatten()?;
            Some(Prop { name, kind: None })
        })
        .collect();
    Some(Props {
        props: props?,
        rest: object.rest.is_some(),
    })
}
