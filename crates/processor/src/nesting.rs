//! How deep a document nests (spec §8.5): blocks in its tree, containers, quotes and list items, and invocations in
//! its expansion. Past it, F038, so a deep document is refused instead of exhausting the stack.

/// The depth every processor renders, of blocks and of invocations (spec §8.5).
pub(crate) const NESTING: usize = 100;
