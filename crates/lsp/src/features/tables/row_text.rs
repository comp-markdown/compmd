//! A line of a table as it is written: what is before the row on its line, its cells' text, and its outer pipes.

use std::ops::Range;

use compmd::tree::Node;

use crate::features::drawing::Source;

/// One line of a table: the header row, the delimiter row or a body row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RowText {
    /// What is before the row on its line, kept as it is: a list item's indentation, a block quote's `>`.
    pub(crate) prefix: String,
    /// Each cell's text, without its pipes and the spaces around it; a delimiter cell's dashes and colons.
    pub(crate) cells: Vec<String>,
    /// The column each cell starts at on its line, as it was read; none for a row a change made.
    pub(crate) starts: Vec<usize>,
    /// Whether the row opens with a pipe.
    pub(crate) opens_with_pipe: bool,
    /// Whether the row closes with a pipe.
    pub(crate) closes_with_pipe: bool,
}

impl RowText {
    /// The header or body row on `line` of `source` whose cells are `cells`, as the processor's tree has them: each
    /// cell from its leading pipe, the last one up to the row's end.
    pub(crate) fn of_cells(source: &Source, line: usize, cells: &[Node]) -> Option<Self> {
        let spans: Vec<Range<usize>> = cells
            .iter()
            .map(|cell| {
                let at = cell.position()?;
                Some(Source::place(at.start).1..Source::place(at.end).1)
            })
            .collect::<Option<_>>()?;
        let first = spans.first()?.start;
        let mut texts: Vec<String> = spans
            .iter()
            .map(|span| source.slice(line, span.start, span.end))
            .collect();
        let opens_with_pipe = texts[0].starts_with('|');
        for text in &mut texts {
            if let Some(rest) = text.strip_prefix('|') {
                *text = rest.to_owned();
            }
        }
        let last = texts.len() - 1;
        let closes_with_pipe = closes_with_pipe(&texts[last]);
        if closes_with_pipe {
            let pipe = texts[last].trim_end_matches(is_space).len() - 1;
            texts[last].truncate(pipe);
        }
        Some(Self {
            prefix: source.slice(line, 0, first),
            cells: texts.iter().map(|text| trimmed(text)).collect(),
            starts: spans.iter().map(|span| span.start).collect(),
            opens_with_pipe,
            closes_with_pipe,
        })
    }

    /// The delimiter row on `line` of `source`, which the tree has no node for. It holds only pipes, dashes, colons and
    /// spaces, so it is split at its pipes; it starts at the first character from which the line holds nothing else.
    pub(crate) fn of_delimiter(source: &Source, line: usize) -> Self {
        let chars: Vec<char> = source.line(line).chars().collect();
        let in_delimiter = |c: &char| matches!(c, '|' | '-' | ':') || is_space(*c);
        let kept = chars.iter().rev().take_while(|c| in_delimiter(c)).count();
        let first = (chars.len() - kept..chars.len())
            .find(|&at| !is_space(chars[at]))
            .unwrap_or(chars.len());
        let row: String = chars[first..].iter().collect();
        let row = row.trim_end_matches(is_space);
        let opens_with_pipe = row.starts_with('|');
        let closes_with_pipe = row.len() > 1 && row.ends_with('|');
        let inner = &row[usize::from(opens_with_pipe)..row.len() - usize::from(closes_with_pipe)];
        let (mut cells, mut starts) = (Vec::new(), Vec::new());
        let mut content = first + usize::from(opens_with_pipe);
        for (index, cell) in inner.split('|').enumerate() {
            starts.push(if index == 0 { first } else { content - 1 });
            content += cell.len() + 1;
            cells.push(trimmed(cell));
        }
        Self {
            prefix: chars[..first].iter().collect(),
            cells,
            starts,
            opens_with_pipe,
            closes_with_pipe,
        }
    }

    /// The index of the cell a cursor at `column` is in: the last that starts before it (a cell starts at its leading
    /// pipe, so a cursor just before a pipe is in the cell the pipe closes), the first when none does.
    pub(crate) fn cell_at(&self, column: usize) -> usize {
        self.starts
            .iter()
            .rposition(|&start| start < column)
            .unwrap_or(0)
    }
}

/// Whether a space between cells: a space or a tab.
pub(crate) const fn is_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// `text` without the spaces around it.
fn trimmed(text: &str) -> String {
    text.trim_matches(is_space).to_owned()
}

/// Whether a row's last cell, `text`, ends with a pipe of its own: one that an odd run of backslashes doesn't escape.
fn closes_with_pipe(text: &str) -> bool {
    let Some(before) = text.trim_end_matches(is_space).strip_suffix('|') else {
        return false;
    };
    let backslashes = before.chars().rev().take_while(|&c| c == '\\').count();
    backslashes % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::Encoding;

    #[test]
    fn a_delimiter_row_is_split_at_its_pipes_after_what_is_before_it() {
        let source = Source::new("> |:--- | -: |\n  --|:-:  \n|---", Encoding::Utf16);
        let quoted = RowText::of_delimiter(&source, 0);
        assert_eq!(quoted.prefix, "> ");
        assert_eq!(quoted.cells, [":---", "-:"]);
        assert_eq!(quoted.starts, [2, 8]);
        assert!(quoted.opens_with_pipe && quoted.closes_with_pipe);
        let bare = RowText::of_delimiter(&source, 1);
        assert_eq!(
            (bare.prefix.as_str(), bare.cells.clone()),
            ("  ", vec!["--".to_owned(), ":-:".to_owned()])
        );
        assert!(!bare.opens_with_pipe && !bare.closes_with_pipe);
        let open = RowText::of_delimiter(&source, 2);
        assert_eq!(open.cells, ["---"]);
        assert!(open.opens_with_pipe && !open.closes_with_pipe);
    }

    #[test]
    fn a_cell_is_found_by_its_column() {
        let row = RowText::of_delimiter(&Source::new("|---|---|", Encoding::Utf16), 0);
        assert_eq!(row.starts, [0, 4]);
        assert_eq!(
            [0, 1, 4, 5, 9].map(|column| row.cell_at(column)),
            [0, 0, 0, 1, 1]
        );
    }

    #[test]
    fn a_closing_pipe_is_one_no_backslash_escapes() {
        assert!(closes_with_pipe(" b |  "));
        assert!(closes_with_pipe(" b \\\\|"));
        assert!(!closes_with_pipe(" b \\|"));
        assert!(!closes_with_pipe(" b "));
    }
}
