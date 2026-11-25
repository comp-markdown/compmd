//! The functions an expression can call (spec §8.2), each with its signatures, what it gives and examples: the `md.`
//! functions, the functions and methods the profile takes from CEL, and its macros. One catalog, for an editor's
//! completion and hover, a reference page and a language server; the processor's own hints read it too.
//!
//! ```
//! let functions = compmd::functions::all();
//! let currency = functions.iter().find(|f| f.name == "md.currency").unwrap();
//! assert_eq!(currency.signatures[0].written, "md.currency(n: number, code: string) → string");
//! assert_eq!(currency.examples[0].expression, r#"md.currency(price * 10, "USD")"#);
//! assert_eq!(currency.examples[0].result, serde_json::json!("$120.00"));
//! ```

#[cfg(test)]
pub(crate) mod examples;
mod macros;
#[cfg(test)]
mod tests;

use crate::cel::functions::Function as CelFunction;
use crate::cel::overload::{self, Library};
use crate::md::MdFunction;
use serde::Serialize;
use serde_json::{Map, Value as Json};

/// Where the catalog's examples are evaluated: at 2026-10-21T10:00:00Z, in UTC, in English (`md.lang` `en`), so
/// those that read the render's time or language give what they say.
pub const EXAMPLE_RENDER: ExampleRender = ExampleRender {
    now: 1_792_576_800 * 1_000_000_000,
    tz: "UTC",
    lang: "en",
};

/// A render's time, time zone and language, for the catalog's examples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExampleRender {
    /// The render time, in nanoseconds since 1970-01-01T00:00:00Z.
    pub now: i128,
    /// The render's time zone (`md.tz`).
    pub tz: &'static str,
    /// The document's language (`md.lang`).
    pub lang: &'static str,
}

/// A function an expression can call.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Function {
    /// How it is called: `md.currency`, `size`, `contains` (a method, `s.contains(…)`), `map` (a macro).
    pub name: String,
    /// Where it comes from.
    pub kind: Kind,
    /// Each way to call it.
    pub signatures: Vec<Signature>,
    /// What it gives, in one line.
    pub description: String,
    /// Calls of it, and what each gives at [`EXAMPLE_RENDER`].
    pub examples: Vec<Example>,
    /// The spec's section that defines it: `8.2`.
    pub section: String,
}

/// Where a function comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// One of CompMD's, called `md.<name>(…)`.
    Md,
    /// One the profile takes from CEL's standard definitions or its strings extension, called `name(…)` or on a
    /// value, `value.name(…)`.
    Cel,
    /// A macro: `has(…)`, and the comprehensions on a list or a map (`list.map(x, …)`), whose first argument names
    /// each element.
    Macro,
}

/// One way to call a function.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Signature {
    /// The type of the value it is called on, for a method: `string`, `list(string)`; `None` for a function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Its params, in order.
    pub params: Vec<Param>,
    /// The type it gives: `string`, or `dyn` where it depends on what it is given.
    pub result: String,
    /// The signature as an editor shows it: `string.indexOf(substring: string, start: int) → int`.
    pub written: String,
}

/// A param of a signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Param {
    /// Its name: the spec's, for the `md.` functions.
    pub name: String,
    /// Its type, as CEL writes it (`number` is an `int`, a `uint` or a `double`; `name` is a macro's element).
    #[serde(rename = "type")]
    pub ty: String,
}

/// A call of a function, and what it gives at [`EXAMPLE_RENDER`].
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Example {
    /// The expression.
    pub expression: String,
    /// The variables it reads, by name (`price`, `md.doc`), with their values.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub given: Map<String, Json>,
    /// What it gives, as JSON: a date or a timestamp as the string it is written as.
    pub result: Json,
}

/// The spec's section of every function and macro.
const SECTION: &str = "8.2";

/// Every function an expression can call: the `md.` functions, then CEL's, then the macros, each in the order the
/// spec lists it.
#[must_use]
pub fn all() -> Vec<Function> {
    let md = MdFunction::ALL
        .iter()
        .map(|function| of_library(*function, function.name(), Kind::Md));
    let cel = CelFunction::ALL
        .iter()
        .map(|function| of_library(*function, function.name(), Kind::Cel));
    md.chain(cel).chain(macros::all()).collect()
}

/// The function written `name`, `md.` and all (`md.currency`, `trim`, `map`).
pub(crate) fn named(name: &str) -> Option<Function> {
    if let Some(name) = name.strip_prefix("md.") {
        return MdFunction::from_name(name).map(|function| of_library(function, name, Kind::Md));
    }
    CelFunction::from_name(name)
        .map(|function| of_library(function, name, Kind::Cel))
        .or_else(|| macros::named(name))
}

/// `function` of a [`Library`], written `name`, from its overloads and its entry.
fn of_library<L: Library>(function: L, name: &str, kind: Kind) -> Function {
    let written_name = format!("{}{name}", L::PREFIX);
    let entry = function.entry();
    let signatures = function
        .overloads()
        .iter()
        .map(|overload| {
            let params = overload
                .names
                .iter()
                .zip(overload.params)
                .map(|(name, param)| Param {
                    name: (*name).to_string(),
                    ty: param.written().to_string(),
                })
                .collect();
            signature(
                &written_name,
                overload
                    .receiver
                    .map(|receiver| receiver.written().to_string()),
                params,
                overload.returns_written(),
            )
        })
        .collect();
    Function {
        name: written_name,
        kind,
        signatures,
        description: entry.description.to_string(),
        examples: entry.examples.iter().map(example).collect(),
        section: SECTION.to_string(),
    }
}

/// A signature, written as an editor shows it; the macros' are written so too.
pub(crate) fn signature(
    name: &str,
    receiver: Option<String>,
    params: Vec<Param>,
    result: &str,
) -> Signature {
    let on = receiver
        .as_ref()
        .map(|receiver| format!("{receiver}."))
        .unwrap_or_default();
    let list: Vec<String> = params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.ty))
        .collect();
    Signature {
        written: format!("{on}{name}({}) → {result}", list.join(", ")),
        receiver,
        params,
        result: result.to_string(),
    }
}

/// An entry's example, its JSON read.
pub(crate) fn example(example: &overload::Example) -> Example {
    let json = |text: &str| serde_json::from_str(text).unwrap_or(Json::Null);
    Example {
        expression: example.expression.to_string(),
        given: example
            .given
            .iter()
            .map(|(name, value)| ((*name).to_string(), json(value)))
            .collect(),
        result: json(example.result),
    }
}
