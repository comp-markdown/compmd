//! Signature help ([the language server], *Signature help*): inside an expression's call, the function's signatures
//! with the argument the place is at; inside a component's param list, its params, with the one the place is on.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use compmd::hints::SignatureHelp as Help;
use lsp_types::{
    Documentation, MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};

use super::markdown;

fn markdown_of(value: String) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value,
    })
}

/// The index `index` as LSP counts.
fn index(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// `help` as LSP's signature help.
pub(crate) fn signature_help(help: &Help) -> SignatureHelp {
    match help {
        Help::Call { function, argument } => {
            let signatures: Vec<SignatureInformation> = function
                .signatures
                .iter()
                .map(|signature| SignatureInformation {
                    label: signature.written.clone(),
                    documentation: Some(markdown_of(function.description.clone())),
                    parameters: Some(
                        signature
                            .params
                            .iter()
                            .map(|param| ParameterInformation {
                                label: ParameterLabel::Simple(format!(
                                    "{}: {}",
                                    param.name, param.ty
                                )),
                                documentation: None,
                            })
                            .collect(),
                    ),
                    active_parameter: None,
                })
                .collect();
            // The first signature that takes as many arguments.
            let active = function
                .signatures
                .iter()
                .position(|signature| signature.params.len() > *argument);
            SignatureHelp {
                signatures,
                active_signature: Some(index(active.unwrap_or_default())),
                active_parameter: Some(index(*argument)),
            }
        }
        Help::Params {
            component,
            params,
            active,
        } => {
            let mut label = format!("{component} {{");
            let mut parameters = Vec::new();
            for (at, param) in params.iter().enumerate() {
                if at > 0 {
                    label.push(' ');
                }
                let start = index(label.encode_utf16().count());
                label.push_str(&param.name);
                parameters.push(ParameterInformation {
                    label: ParameterLabel::LabelOffsets([
                        start,
                        index(label.encode_utf16().count()),
                    ]),
                    documentation: Some(markdown_of(markdown::param(param))),
                });
            }
            label.push('}');
            SignatureHelp {
                signatures: vec![SignatureInformation {
                    label,
                    documentation: None,
                    parameters: Some(parameters),
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: active.map(index),
            }
        }
    }
}

#[cfg(test)]
mod tests;
