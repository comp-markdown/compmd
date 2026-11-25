use super::expand;
use quote::quote;

fn expanded(item: proc_macro2::TokenStream) -> Result<String, String> {
    expand(quote!(prefix = "md.", call = Call), item)
        .map(|tokens| tokens.to_string())
        .map_err(|error| error.to_string())
}

#[test]
fn a_library_gets_its_names_overloads_entries_and_tests() {
    let tokens = expanded(quote! {
        /// The functions.
        pub(crate) enum MdFunction {
            /// An amount written in a currency.
            ///
            /// ```cel
            /// let price = 12
            /// md.currency(price * 10, "USD") // "$120.00"
            /// ```
            #[overload(fn(n: Number, code: String) -> String = currency)]
            Currency,
            /// The first position of a substring.
            ///
            /// ```cel
            /// "banana".indexOf("an") // 1
            /// "banana".indexOf("an", 2) // 3
            /// ```
            #[overload(fn(self: String, substring: String) -> Int = index_of)]
            #[overload(fn(self: String, substring: String, start: Int) -> Int = index_of)]
            #[counts]
            IndexOf,
        }
    })
    .expect("a library");
    for part in [
        "# [derive (Clone , Copy , Debug , PartialEq , Eq)]",
        "# [doc = r\" The functions.\"] pub (crate) enum MdFunction",
        "Currency , # [doc",
        r#"pub (crate) const NAMES : & 'static [& 'static str] = & ["currency" , "indexOf"]"#,
        "impl crate :: cel :: library :: Library for MdFunction",
        "type Call = Call ;",
        r#"const PREFIX : & 'static str = "md." ;"#,
        r#"& crate :: cel :: library :: Definition { name : "currency" , description : "An amount written in a currency.""#,
        r#"names : & ["n" , "code"]"#,
        "counts : false",
        "counts : true",
        "mod md_function_examples",
        "fn currency_1 ()",
        "fn index_of_2 ()",
        ". examples [1usize]",
    ] {
        assert!(tokens.contains(part), "{part} in {tokens}");
    }
    assert!(
        !tokens.contains("# [overload"),
        "the overloads are taken off"
    );
    assert!(!tokens.contains("# [counts]"));
}

#[test]
fn what_a_function_lacks_stops_the_build() {
    let error =
        |variant: proc_macro2::TokenStream| expanded(quote!(enum Function { #variant })).err();
    assert_eq!(
        error(quote!(
            #[overload(fn() -> Int = f)]
            Size
        ))
        .as_deref(),
        Some("`Size` has no description: its doc's first paragraph")
    );
    assert_eq!(
        error(quote! {
            /// The length.
            ///
            /// ```cel
            /// size("a") // 1
            /// ```
            Size
        })
        .as_deref(),
        Some("`Size` has no overload: an #[overload(fn(…) -> … = …)] for each way to call it")
    );
    assert_eq!(
        error(quote!(Size(u8))).as_deref(),
        Some("`Size` is a function: it has no fields or value")
    );
    assert_eq!(
        error(quote!(Size = 1)).as_deref(),
        Some("`Size` is a function: it has no fields or value")
    );
    assert_eq!(
        error(quote!(
            #[overload(fn(n) -> Int = f)]
            Size
        ))
        .as_deref(),
        Some("expected `:`")
    );
}

#[test]
fn the_attribute_names_a_prefix_and_a_call() {
    let item = quote!(
        enum Function {}
    );
    let error = |attr: proc_macro2::TokenStream| {
        expand(attr, item.clone())
            .err()
            .map(|error| error.to_string())
    };
    assert_eq!(
        error(quote!(call = Call)).as_deref(),
        Some("#[library] needs `prefix = …`")
    );
    assert_eq!(
        error(quote!(prefix = "")).as_deref(),
        Some("#[library] needs `call = …`")
    );
    assert_eq!(
        error(quote!(prefix = "", call = Call, other = 1)).as_deref(),
        Some("expected `prefix` or `call`")
    );
    assert!(
        expand(
            quote!(prefix = "", call = Call),
            quote!(
                struct Function;
            )
        )
        .is_err()
    );
}
