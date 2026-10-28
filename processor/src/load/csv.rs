//! CSV as RFC 4180 reads it (spec §8.6), with the `csv` crate: fields separated by a delimiter, `"` quoting a field,
//! `""` writing a quote, quoted fields holding delimiters and line breaks, lines ending in LF or CRLF, a byte order
//! mark ignored. Rows may have any number of fields; the header decides how many they need.

use csv::ReaderBuilder;

/// A record: its fields, and the line it starts on.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub(super) cells: Vec<String>,
    pub(super) line: usize,
}

/// The records of `text`; blank lines are skipped.
///
/// # Errors
///
/// A delimiter that isn't one ASCII character.
pub(super) fn rows(text: &str, delimiter: char) -> Result<Vec<Row>, String> {
    let byte = u8::try_from(delimiter)
        .ok()
        .filter(u8::is_ascii)
        .ok_or_else(|| format!("the delimiter `{delimiter}` isn't one ASCII character"))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(byte)
        .from_reader(text.as_bytes());
    // Reading text in memory fails only on invalid UTF-8, which a `&str` can't hold.
    Ok(reader
        .records()
        .filter_map(Result::ok)
        .filter(|record| record.iter().any(|cell| !cell.is_empty()))
        .map(|record| Row {
            line: record
                .position()
                .and_then(|position| usize::try_from(position.byte()).ok())
                .map_or(1, |byte| line_at(text, byte)),
            cells: record.iter().map(str::to_string).collect(),
        })
        .collect())
}

/// The line of a record the crate places at `byte`, from 1: the crate places a record where the line ending before
/// it starts, so its first byte after line endings counts.
fn line_at(text: &str, byte: usize) -> usize {
    let bytes = text.as_bytes();
    let start = bytes[byte.min(bytes.len())..]
        .iter()
        .position(|b| !matches!(b, b'\r' | b'\n'))
        .map_or(bytes.len(), |skipped| byte + skipped);
    1 + text
        .get(..start)
        .map_or(0, |before| before.matches('\n').count())
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
        )
        .expect("CSV");
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
        let cells_of = |text, delimiter| -> Vec<String> {
            let rows = rows(text, delimiter).expect("CSV");
            rows.into_iter().flat_map(|row| row.cells).collect()
        };
        assert_eq!(cells_of("a;b\n1,5;2\n", ';'), ["a", "b", "1,5", "2"]);
        assert_eq!(cells_of("a\tb\n", '\t'), ["a", "b"]);
        assert_eq!(rows("\n,\n", ','), Ok(Vec::new()));
        assert!(rows("a", '→').is_err());
    }
}
