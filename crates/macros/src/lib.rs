//! The `compmd` crate's own macros, for its use only.
//!
//! [`macro@library`] writes a family of expression functions (spec §8.2): each function is one variant of an enum,
//! with its documentation, its examples and its overloads beside it, so none of them can go missing or apart from the
//! others.

mod doc;
mod library;
mod names;
mod overload;

use proc_macro::TokenStream;

/// Makes an enum of functions into a library of them, for `compmd`'s `cel::overload::Library`.
///
/// Each variant is a function, named as its variant in lower camel case (`GetFullYear` is `getFullYear`), with:
///
/// - its doc's first paragraph as its description;
/// - a ` ```cel ` block in its doc, each line `expression // result` an example, after `let name = value` lines for
///   the variables it reads (a value is JSON, or the name of a constant that holds it);
/// - an `#[overload(fn(self: String, substring: String) -> Int = strings::index_of)]` for each way to call it, `self`
///   first for a method, with `, size = path` when its result can be far larger than its arguments;
/// - `#[counts]` when its `int` arguments are positions or counts.
///
/// The macro gives the enum `ALL`, `NAMES`, `name()` and `from_name()`, implements `Library` for it, and writes a
/// test for each example. A function without a description, an example or an overload doesn't compile.
///
/// ```ignore
/// #[library(prefix = "", call = Call)]
/// pub(crate) enum Function {
///     /// The length of a string, in characters, a list or a map.
///     ///
///     /// ```cel
///     /// size("héllo") // 5
///     /// ```
///     #[overload(fn(value: Sized) -> Int = size)]
///     #[overload(fn(self: Sized) -> Int = size)]
///     Size,
/// }
/// ```
#[proc_macro_attribute]
pub fn library(attr: TokenStream, item: TokenStream) -> TokenStream {
    library::expand(attr.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
