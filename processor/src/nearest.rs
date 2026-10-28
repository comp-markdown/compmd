//! The nearest of some names to a misspelt one, for diagnostics that suggest a name.

/// The name in `names` closest to `name` by edit distance, ignoring case, if it is close enough to be a likely
/// misspelling: at most a third of the longer name's letters differ.
pub(crate) fn nearest<'a>(name: &str, names: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let lower = name.to_lowercase();
    names
        .into_iter()
        .map(|candidate| (distance(&lower, &candidate.to_lowercase()), candidate))
        .filter(|(distance, candidate)| {
            *distance * 3 <= name.chars().count().max(candidate.chars().count())
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// The Levenshtein distance between `a` and `b`, in code points.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != *cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_closest_likely_misspelling() {
        let keys = ["version", "import", "inputs", "syntax"];
        assert_eq!(nearest("input", keys), Some("inputs"));
        assert_eq!(nearest("Imports", keys), Some("import"));
        assert_eq!(nearest("colour", keys), None);
    }

    #[test]
    fn measures_edits() {
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("", "ab"), 2);
        assert_eq!(distance("ab", "ab"), 0);
    }
}
