use super::Overload;
use quote::quote;

fn written(overload: &str) -> String {
    let overload: Overload = syn::parse_str(overload).expect("an overload");
    overload.tokens(&syn::parse_quote!(Call)).to_string()
}

#[test]
fn a_function_names_and_types_its_params() {
    let tokens = written("fn(n: Number, code: String) -> String = currency");
    assert_eq!(
        tokens,
        quote! {
            crate::cel::library::Overload {
                receiver: None,
                params: &[crate::cel::library::Param::Number, crate::cel::library::Param::String],
                names: &["n", "code"],
                returns: crate::cel::library::Returns::String,
                call: currency as Call,
                size: None,
            }
        }
        .to_string()
    );
}

#[test]
fn a_method_takes_self_first() {
    let tokens =
        written("fn(self: String, substring: String, start: Int) -> Int = strings::index_of");
    assert!(tokens.contains("receiver : Some (crate :: cel :: library :: Param :: String)"));
    assert!(tokens.contains(r#"names : & ["substring" , "start"]"#));
    assert!(tokens.contains("call : strings :: index_of as Call"));
    let none = written("fn(self: Timestamp) -> Int = getters::hours");
    assert!(none.contains("params : & []"));
}

#[test]
fn a_result_can_name_an_argument_and_declare_its_size() {
    let tokens =
        written("fn(list: List, field: String) -> Same(0) = lists::sort_by, size = lists::size");
    assert!(tokens.contains("returns : crate :: cel :: library :: Returns :: Same (0)"));
    assert!(tokens.contains("size : Some (lists :: size as crate :: cel :: library :: Size)"));
}

#[test]
fn a_param_may_have_a_keyword_name() {
    let tokens = written("fn(self: String, old: String, new: String) -> String = strings::replace");
    assert!(tokens.contains(r#"names : & ["old" , "new"]"#));
}

#[test]
fn what_isnt_an_overload_says_why() {
    let error = |text: &str| {
        syn::parse_str::<Overload>(text)
            .err()
            .map(|error| error.to_string())
    };
    assert_eq!(
        error("fn(n: Number) -> Int = f, sized = g").as_deref(),
        Some("expected `size = …`")
    );
    assert!(error("(n: Number) -> Int = f").is_some(), "`fn` first");
    assert!(
        error("fn(n Number) -> Int = f").is_some(),
        "a param is `name: Type`"
    );
    assert!(error("fn(n: Number) = f").is_some(), "what it gives");
    assert!(
        error("fn(n: Number, self: String) -> Int = f").is_some(),
        "`self` only first"
    );
}
