//! What the tables' tests share.

use super::table_text::TableText;
use super::tables_of;
use crate::encoding::Encoding;
use crate::features::drawing::Source;
use crate::features::testing::processed;

/// The first table of `text`.
pub(crate) fn table_of(text: &str) -> TableText {
    let processor = processed(text);
    let source = Source::new(text, Encoding::Utf16);
    tables_of(&processor.tree().children, &source)
        .into_iter()
        .next()
        .expect("a table")
}
