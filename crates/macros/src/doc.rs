//! A function's documentation, read from its variant's doc: the first paragraph is its description, and a ` ```cel `
//! block holds its examples, each `expression // result`, after `let name = value` lines for the variables it reads.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Expr, ExprLit, Ident, Lit, Meta};

/// What a function's doc says.
#[derive(Debug)]
pub(crate) struct Doc {
    pub(crate) description: String,
    pub(crate) examples: Vec<Example>,
}

/// An example: a call, the variables it reads, and what it gives.
#[derive(Debug)]
pub(crate) struct Example {
    expression: String,
    given: Vec<(String, Given)>,
    /// What it gives, as JSON.
    result: String,
}

/// A variable's value: JSON, or the name of a constant that holds it.
#[derive(Debug, PartialEq, Eq)]
enum Given {
    Json(String),
    Constant(String),
}

/// The lines of the doc attributes among `attrs`, without the space `///` leaves before each.
pub(crate) fn lines(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| match &attr.meta {
            Meta::NameValue(doc) if doc.path.is_ident("doc") => match &doc.value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(text),
                    ..
                }) => Some(text.value()),
                _ => None,
            },
            _ => None,
        })
        .flat_map(|text| {
            // `split`, not `lines`: an empty `///` line is a paragraph's end, which `lines` would drop.
            text.split('\n')
                .map(|line| line.strip_prefix(' ').unwrap_or(line).to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// What the doc `lines` say.
///
/// # Errors
///
/// What is missing or can't be read, as a phrase about the function: "has no description".
pub(crate) fn read(lines: &[String]) -> Result<Doc, String> {
    let description = lines
        .iter()
        .map(|line| line.trim())
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if description.is_empty() || description.starts_with("```") {
        return Err("has no description: its doc's first paragraph".to_string());
    }
    let start = lines
        .iter()
        .position(|line| line.trim() == "```cel")
        .ok_or("has no example: a ```cel block in its doc")?;
    let block = &lines[start + 1..];
    let end = block
        .iter()
        .position(|line| line.trim() == "```")
        .ok_or("has a ```cel block that doesn't end")?;
    let examples = examples(&block[..end])?;
    if examples.is_empty() {
        return Err("has no example in its ```cel block".to_string());
    }
    Ok(Doc {
        description,
        examples,
    })
}

/// The examples of a ` ```cel ` block's lines.
fn examples(lines: &[String]) -> Result<Vec<Example>, String> {
    let mut examples = Vec::new();
    let mut given = Vec::new();
    for line in lines.iter().map(|line| line.trim()) {
        if line.is_empty() {
            continue;
        }
        if let Some(binding) = line.strip_prefix("let ") {
            let (name, value) = binding
                .split_once(" = ")
                .ok_or_else(|| format!("has `{line}`, which isn't `let name = value`"))?;
            given.push((name.to_string(), Given::of(value)));
            continue;
        }
        let (expression, result) = line
            .rsplit_once(" // ")
            .ok_or_else(|| format!("has `{line}`, which isn't `expression // result`"))?;
        examples.push(Example {
            expression: expression.to_string(),
            given: std::mem::take(&mut given),
            result: result.to_string(),
        });
    }
    if given.is_empty() {
        Ok(examples)
    } else {
        Err("has a `let` with no example after it".to_string())
    }
}

impl Given {
    /// A value as written: a constant's name is in upper case, which no JSON value is.
    fn of(value: &str) -> Self {
        let constant = value.starts_with(|c: char| c.is_ascii_uppercase())
            && value
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if constant {
            Self::Constant(value.to_string())
        } else {
            Self::Json(value.to_string())
        }
    }
}

impl Example {
    /// The `Example` it makes.
    pub(crate) fn tokens(&self) -> TokenStream {
        let expression = &self.expression;
        let result = &self.result;
        let given = self.given.iter().map(|(name, value)| match value {
            Given::Json(json) => quote!((#name, #json)),
            Given::Constant(constant) => {
                let constant = Ident::new(constant, Span::call_site());
                quote!((#name, #constant))
            }
        });
        quote! {
            crate::cel::library::Example {
                expression: #expression,
                given: &[#(#given),*],
                result: #result,
            }
        }
    }
}

#[cfg(test)]
mod tests;
