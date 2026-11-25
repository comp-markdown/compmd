//! The functions a value of a type is called with, `value.f(…)` (spec §8.2): what an editor offers after `x.`.

use super::{Function, Kind, macros, of_library};
use crate::cel::Type;
use crate::cel::functions::Function as CelFunction;
use crate::cel::library::Library;

/// The functions a value of type `ty` is called with as `value.f(…)`: CEL's methods whose receiver takes it, then
/// the comprehensions, for a list, a map or a record. A `date` takes what a `string` takes, and `dyn`, whose type is
/// known only when evaluated, takes them all; a number, a `bool` or `null` takes none.
///
/// ```
/// use compmd::cel::Type;
///
/// let names = |ty: Type| -> Vec<String> {
///     compmd::functions::of_type(&ty).into_iter().map(|f| f.name).collect()
/// };
/// assert!(names(Type::String).contains(&"startsWith".to_string()));
/// assert!(names(Type::list(Type::Int)).contains(&"map".to_string()));
/// assert!(names(Type::Int).is_empty());
/// ```
#[must_use]
pub fn of_type(ty: &Type) -> Vec<Function> {
    let methods = CelFunction::ALL
        .iter()
        .copied()
        .filter(|function| {
            function.definition().overloads.iter().any(|overload| {
                overload
                    .receiver
                    .is_some_and(|receiver| receiver.takes_type(ty))
            })
        })
        .map(|function| of_library(function, Kind::Cel));
    let ranged = matches!(
        ty.cel(),
        Type::List(_) | Type::Map(..) | Type::Record(_) | Type::Dyn
    );
    let comprehensions = if ranged {
        macros::comprehensions()
    } else {
        Vec::new()
    };
    methods.chain(comprehensions).collect()
}
