//! `#[library]`: an enum of functions, each variant with its documentation, examples and overloads, made into the
//! enum, its names, its `Library` implementation and a test for each example.

use crate::doc::{self, Doc};
use crate::names;
use crate::overload::Overload;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{Fields, Ident, ItemEnum, LitStr, Type, Variant};

/// The attribute's arguments: `prefix = "md.", call = Call`.
struct Args {
    /// What the functions' names are written with: `""` or `"md."`.
    prefix: LitStr,
    /// The type of their implementations.
    call: Type,
}

impl Args {
    fn parse(attr: TokenStream) -> syn::Result<Self> {
        let mut prefix = None;
        let mut call = None;
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("prefix") {
                prefix = Some(meta.value()?.parse()?);
                Ok(())
            } else if meta.path.is_ident("call") {
                call = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("expected `prefix` or `call`"))
            }
        });
        syn::parse::Parser::parse2(parser, attr)?;
        let missing = |what: &str| {
            syn::Error::new(Span::call_site(), format!("#[library] needs `{what} = …`"))
        };
        Ok(Self {
            prefix: prefix.ok_or_else(|| missing("prefix"))?,
            call: call.ok_or_else(|| missing("call"))?,
        })
    }
}

/// A function: its variant, what its doc says and its overloads.
struct Function {
    variant: Variant,
    name: String,
    doc: Doc,
    overloads: Vec<Overload>,
    counts: bool,
}

impl Function {
    /// The function `variant` defines, with its `#[overload]`s and `#[counts]` taken off it.
    fn of(mut variant: Variant) -> syn::Result<Self> {
        let ident = variant.ident.clone();
        let fail = |message: String| syn::Error::new(ident.span(), format!("`{ident}` {message}"));
        if !matches!(variant.fields, Fields::Unit) || variant.discriminant.is_some() {
            return Err(fail("is a function: it has no fields or value".to_string()));
        }
        let mut overloads = Vec::new();
        let mut counts = false;
        let mut kept = Vec::new();
        for attr in std::mem::take(&mut variant.attrs) {
            if attr.path().is_ident("overload") {
                overloads.push(attr.parse_args()?);
            } else if attr.path().is_ident("counts") {
                counts = true;
            } else {
                kept.push(attr);
            }
        }
        variant.attrs = kept;
        let doc = doc::read(&doc::lines(&variant.attrs)).map_err(fail)?;
        if overloads.is_empty() {
            return Err(fail(
                "has no overload: an #[overload(fn(…) -> … = …)] for each way to call it"
                    .to_string(),
            ));
        }
        Ok(Self {
            name: names::function(&ident.to_string()),
            variant,
            doc,
            overloads,
            counts,
        })
    }
}

/// The library `item` defines, with the attribute's arguments `attr`.
pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let args = Args::parse(attr)?;
    let mut item: ItemEnum = syn::parse2(item)?;
    let functions = std::mem::take(&mut item.variants)
        .into_iter()
        .map(Function::of)
        .collect::<syn::Result<Vec<_>>>()?;
    item.variants = functions
        .iter()
        .map(|function| function.variant.clone())
        .collect();
    let enumeration = &item.ident;
    let vis = &item.vis;
    let Args { prefix, call } = &args;
    let variants: Vec<&Ident> = functions
        .iter()
        .map(|function| &function.variant.ident)
        .collect();
    let names: Vec<&String> = functions.iter().map(|function| &function.name).collect();
    let overloads = functions.iter().map(|function| {
        let overloads = function
            .overloads
            .iter()
            .map(|overload| overload.tokens(call));
        quote!(const { &[#(#overloads),*] })
    });
    let entries = functions.iter().map(|function| {
        let description = &function.doc.description;
        let examples = function.doc.examples.iter().map(doc::Example::tokens);
        quote! {
            crate::cel::overload::Entry {
                description: #description,
                examples: const { &[#(#examples),*] },
            }
        }
    });
    let counted: Vec<&Ident> = functions
        .iter()
        .filter(|function| function.counts)
        .map(|function| &function.variant.ident)
        .collect();
    let counts = if counted.is_empty() {
        quote!()
    } else {
        quote! {
            fn counts(self) -> bool {
                matches!(self, #(Self::#counted)|*)
            }
        }
    };
    let tests = tests(enumeration, &functions);
    Ok(quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #item

        #[allow(dead_code, reason = "not every library uses every helper")]
        impl #enumeration {
            /// Every function, in declared order.
            #vis const ALL: &'static [Self] = &[#(Self::#variants),*];

            /// Every function's name, in declared order.
            #vis const NAMES: &'static [&'static str] = &[#(#names),*];

            /// The name as it is written.
            #vis fn name(self) -> &'static str {
                match self {
                    #(Self::#variants => #names),*
                }
            }

            /// The function written `name`.
            #vis fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|function| function.name() == name)
            }
        }

        impl crate::cel::overload::Library for #enumeration {
            type Call = #call;
            const PREFIX: &'static str = #prefix;

            fn from_name(name: &str) -> Option<Self> {
                Self::from_name(name)
            }

            fn names() -> &'static [&'static str] {
                Self::NAMES
            }

            #[allow(clippy::too_many_lines, reason = "one arm for each function")]
            fn overloads(self) -> &'static [crate::cel::overload::Overload<#call>] {
                match self {
                    #(Self::#variants => #overloads),*
                }
            }

            #[allow(clippy::too_many_lines, reason = "one arm for each function")]
            fn entry(self) -> crate::cel::overload::Entry {
                match self {
                    #(Self::#variants => #entries),*
                }
            }

            #counts
        }

        #tests
    })
}

/// A test for each example: `md_function_examples::currency_1` evaluates `Currency`'s first.
fn tests(enumeration: &Ident, functions: &[Function]) -> TokenStream {
    let module = format_ident!("{}_examples", names::snake(&enumeration.to_string()));
    let tests = functions.iter().flat_map(|function| {
        let variant = &function.variant.ident;
        let snake = names::snake(&variant.to_string());
        (0..function.doc.examples.len()).map(move |index| {
            let test = format_ident!("{snake}_{}", index + 1);
            quote! {
                #[test]
                fn #test() {
                    crate::functions::examples::assert_gives(
                        &<super::#enumeration as crate::cel::overload::Library>::entry(super::#enumeration::#variant)
                            .examples[#index],
                    );
                }
            }
        })
    });
    quote! {
        #[cfg(test)]
        mod #module {
            #(#tests)*
        }
    }
}

#[cfg(test)]
mod tests;
