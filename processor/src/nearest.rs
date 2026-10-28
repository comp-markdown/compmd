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

/// The edits from `a` to `b`, in code points: inserts, deletes, changes, and swaps of two neighbours, the commonest
/// typo (the optimal string alignment distance).
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    // The rows for the two previous letters of `a`, and the current one.
    let mut before: Vec<usize> = Vec::new();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut row = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let change = usize::from(a[i - 1] != b[j - 1]);
            row[j] = (previous[j] + 1)
                .min(row[j - 1] + 1)
                .min(previous[j - 1] + change);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                row[j] = row[j].min(before[j - 2] + 1);
            }
        }
        before = std::mem::replace(&mut previous, row);
    }
    previous[b.len()]
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
        assert_eq!(distance("ab", ""), 2);
        assert_eq!(distance("ab", "ab"), 0);
        assert_eq!(distance("dco", "doc"), 1);
        assert_eq!(distance("ca", "abc"), 3);
    }
}
