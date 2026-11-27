//! A heading of the expanded tree (spec §12.3): at its computed depth, with its anchor.

use super::Node;
use serde::Serialize;

/// A heading of the expanded tree: at its computed depth (spec §7.2), with the anchor a link reaches it by (spec
/// §12.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Heading {
    /// The level, 1 to 6, computed: a heading of a body with relative headings is below the heading its invocation
    /// is under.
    pub depth: u8,
    /// The name a link reaches it by: GitHub's for its expanded text, with `-1`, `-2`, … for a repeat in its root.
    pub anchor: String,
    /// The heading's text, expanded.
    pub children: Vec<Node>,
}
