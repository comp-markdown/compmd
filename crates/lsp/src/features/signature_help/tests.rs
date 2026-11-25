use lsp_types::ParameterLabel;

use super::*;
use crate::features::testing::{caret, processed};

/// The signature help at the caret `|` of `marked`.
fn helped(marked: &str) -> SignatureHelp {
    let (text, at) = caret(marked);
    signature_help(&processed(&text).signature_at(at).expect("help"))
}

#[test]
fn a_call_gives_its_functions_signatures_and_the_argument() {
    let help = helped("::: box {padding=(md.number(1, |))}\n::: /box\n");
    assert_eq!(help.active_parameter, Some(1));
    let first = &help.signatures[0];
    assert!(first.label.starts_with("md.number("), "{}", first.label);
    let Some(ParameterLabel::Simple(param)) = first
        .parameters
        .as_ref()
        .and_then(|params| params.first())
        .map(|param| &param.label)
    else {
        panic!("a param");
    };
    assert!(
        first.label.contains(param.as_str()),
        "`{param}` is in `{}`",
        first.label
    );
}

#[test]
fn a_param_list_gives_the_components_params_with_the_one_written() {
    let help = helped("::: box {bor|der=subtle}\n::: /box\n");
    let signature = &help.signatures[0];
    assert!(
        signature.label.starts_with("box {") && signature.label.ends_with('}'),
        "{}",
        signature.label
    );
    let params = signature.parameters.as_ref().expect("params");
    let active = help.active_parameter.expect("the one written") as usize;
    let ParameterLabel::LabelOffsets([start, end]) = params[active].label else {
        panic!("offsets");
    };
    assert_eq!(&signature.label[start as usize..end as usize], "border");
    assert_eq!(
        helped("::: box {border=subtle |}\n::: /box\n").active_parameter,
        None
    );
}
