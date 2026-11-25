//! One way to call a function, as `#[overload(…)]` writes it: `fn(self: String, substring: String) -> Int =
//! strings::index_of`, its params named and typed, with `, size = path` for a result that can be far larger than its
//! arguments.

use proc_macro2::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Path, Token, Type, parenthesized, token};

/// An overload as written.
pub(crate) struct Overload {
    /// A method's receiver's type: `self: String`.
    receiver: Option<Ident>,
    /// Each param's name and type.
    params: Vec<(String, Ident)>,
    /// What it gives: `Int`, or `Same(0)`, which names an argument.
    returns: Ident,
    returns_index: Option<LitInt>,
    /// Its implementation.
    call: Path,
    /// The size of its result, when it declares one.
    size: Option<Path>,
}

impl Parse for Overload {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![fn]>()?;
        let list;
        parenthesized!(list in input);
        let mut receiver = None;
        let mut params = Vec::new();
        while !list.is_empty() {
            if receiver.is_none() && params.is_empty() && list.peek(Token![self]) {
                list.parse::<Token![self]>()?;
                list.parse::<Token![:]>()?;
                receiver = Some(list.parse()?);
            } else {
                let name = list.call(Ident::parse_any)?;
                if name == "self" {
                    return Err(syn::Error::new(
                        name.span(),
                        "`self` comes first, for a method's receiver",
                    ));
                }
                list.parse::<Token![:]>()?;
                params.push((name.to_string(), list.parse()?));
            }
            if !list.is_empty() {
                list.parse::<Token![,]>()?;
            }
        }
        input.parse::<Token![->]>()?;
        let returns = input.parse()?;
        let returns_index = if input.peek(token::Paren) {
            let index;
            parenthesized!(index in input);
            Some(index.parse()?)
        } else {
            None
        };
        input.parse::<Token![=]>()?;
        let call = input.parse()?;
        let size = if input.is_empty() {
            None
        } else {
            input.parse::<Token![,]>()?;
            let key: Ident = input.parse()?;
            if key != "size" {
                return Err(syn::Error::new(key.span(), "expected `size = …`"));
            }
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        };
        Ok(Self {
            receiver,
            params,
            returns,
            returns_index,
            call,
            size,
        })
    }
}

impl Overload {
    /// The `Overload` it makes, its implementation of the type `call`.
    pub(crate) fn tokens(&self, call: &Type) -> TokenStream {
        let receiver = self.receiver.as_ref().map_or_else(
            || quote!(None),
            |ty| quote!(Some(crate::cel::library::Param::#ty)),
        );
        let types = self
            .params
            .iter()
            .map(|(_, ty)| quote!(crate::cel::library::Param::#ty));
        let names = self.params.iter().map(|(name, _)| name);
        let returns = &self.returns;
        let index = self.returns_index.as_ref().map(|index| quote!((#index)));
        let returns = quote!(crate::cel::library::Returns::#returns #index);
        let implementation = &self.call;
        let size = self.size.as_ref().map_or_else(
            || quote!(None),
            |size| quote!(Some(#size as crate::cel::library::Size)),
        );
        quote! {
            crate::cel::library::Overload {
                receiver: #receiver,
                params: &[#(#types),*],
                names: &[#(#names),*],
                returns: #returns,
                call: #implementation as #call,
                size: #size,
            }
        }
    }
}

#[cfg(test)]
mod tests;
