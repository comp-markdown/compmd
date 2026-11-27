//! What a client declares of the drawing in `initialize` ([a document drawn in the editor], decision 3), in its
//! capabilities' `experimental`: `compmd.drawing: { primitives, refreshSupport }`. A client that declares nothing is
//! never sent a drawing message.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::ClientCapabilities;
use serde_json::Value;

use super::primitive::{Primitive, Primitives};

/// The drawing a client declared.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClientDrawing {
    /// The primitives it draws; a name that is no primitive is let go.
    pub(crate) primitives: Primitives,
    /// Whether it takes `compmd/drawing/refresh`.
    pub(crate) refresh_support: bool,
}

impl ClientDrawing {
    /// What a client of `capabilities` declared of the drawing; `None` when it declared nothing.
    pub(crate) fn of(capabilities: &ClientCapabilities) -> Option<Self> {
        let declared = capabilities
            .experimental
            .as_ref()?
            .pointer("/compmd/drawing")
            .filter(|declared| declared.is_object())?;
        let primitives = declared
            .get("primitives")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(Primitive::named)
            .collect();
        let refresh_support = declared.get("refreshSupport").and_then(Value::as_bool) == Some(true);
        Some(Self {
            primitives,
            refresh_support,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn declared(experimental: Value) -> Option<ClientDrawing> {
        ClientDrawing::of(&ClientCapabilities {
            experimental: Some(experimental),
            ..ClientCapabilities::default()
        })
    }

    #[test]
    fn a_client_declares_its_primitives_and_whether_it_takes_refreshes() {
        let drawing = declared(json!({ "compmd": { "drawing": { "primitives": ["hide", "line", "image", 3], "refreshSupport": true } } }))
            .expect("declared");
        assert!(drawing.primitives.contains(Primitive::Hide));
        assert!(drawing.primitives.contains(Primitive::Line));
        assert!(!drawing.primitives.contains(Primitive::Style));
        assert!(drawing.refresh_support);
        assert_eq!(
            declared(json!({ "compmd": { "drawing": {} } })),
            Some(ClientDrawing::default()),
            "nothing drawn, no refreshes"
        );
    }

    #[test]
    fn a_client_that_declares_nothing_has_no_drawing() {
        assert_eq!(ClientDrawing::of(&ClientCapabilities::default()), None);
        assert_eq!(declared(json!({ "compmd": {} })), None);
        assert_eq!(declared(json!({ "compmd": { "drawing": true } })), None);
    }
}
