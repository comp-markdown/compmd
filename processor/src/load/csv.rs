//! CSV as RFC 4180 reads it (spec §8.6): fields separated by a delimiter, `"` quoting a field, `""` writing a
//! quote, quoted fields holding delimiters and line breaks, lines ending in LF or CRLF, a byte order mark ignored.

/// A record: its fields, and the line it starts on.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub(super) cells: Vec<String>,
    pub(super) line: usize,
}

/// The records of `text`; blank lines are skipped.
pub(super) fn rows(text: &str, delimiter: char) -> Vec<Row> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows = Vec::new();
    let mut row = Row {
        cells: Vec::new(),
        line: 1,
    };
    let mut field = String::new();
    let mut quoted = false;
    let mut line = 1;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            '\r' if !quoted && chars.peek() == Some(&'\n') => {}
            '\n' if !quoted => {
                line += 1;
                row.cells.push(std::mem::take(&mut field));
                let next = Row {
                    cells: Vec::new(),
                    line,
                };
                push(&mut rows, std::mem::replace(&mut row, next));
            }
            c if c == delimiter && !quoted => row.cells.push(std::mem::take(&mut field)),
            c => {
                if c == '\n' {
                    line += 1;
                }
                field.push(c);
            }
        }
    }
    row.cells.push(field);
    push(&mut rows, row);
    rows
}

/// Adds `row` unless it is a blank line.
fn push(rows: &mut Vec<Row>, row: Row) {
    if row.cells.iter().any(|cell| !cell.is_empty()) {
        rows.push(row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(rows: &[Row]) -> Vec<Vec<&str>> {
        rows.iter()
            .map(|row| row.cells.iter().map(String::as_str).collect())
            .collect()
    }

    #[test]
    fn quotes_hold_delimiters_quotes_and_line_breaks() {
        let rows = rows(
            "\u{feff}name,note\r\nAna,\"says \"\"hi\"\", twice\"\n\nKenji,\"two\nlines\"\nLi,last",
            ',',
        );
        assert_eq!(
            cells(&rows),
            [
                vec!["name", "note"],
                vec!["Ana", "says \"hi\", twice"],
                vec!["Kenji", "two\nlines"],
                vec!["Li", "last"],
            ]
        );
        let lines: Vec<usize> = rows.iter().map(|row| row.line).collect();
        assert_eq!(lines, [1, 2, 4, 6]);
    }

    #[test]
    fn the_delimiter_is_chosen() {
        assert_eq!(
            cells(&rows("a;b\n1,5;2\n", ';')),
            [vec!["a", "b"], vec!["1,5", "2"]]
        );
        assert_eq!(cells(&rows("a\tb\n", '\t')), [vec!["a", "b"]]);
        assert_eq!(rows("\n\n", ','), []);
    }
}
