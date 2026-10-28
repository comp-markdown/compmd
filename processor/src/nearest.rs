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

/// The fewest edits that turn `a` into `b`, in code points: inserting, deleting or changing a letter, or swapping
/// two neighbours, the commonest typo (the optimal string alignment distance).
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    // d[i][j]: the distance between the first i letters of `a` and the first j letters of `b`.
    let mut d = vec![vec![0; b.len() + 1]; a.len() + 1];
    for i in 0..=a.len() {
        for j in 0..=b.len() {
            d[i][j] = if i == 0 || j == 0 {
                i + j
            } else {
                let delete = d[i - 1][j] + 1;
                let insert = d[i][j - 1] + 1;
                let change = d[i - 1][j - 1] + usize::from(a[i - 1] != b[j - 1]);
                let swapped = i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1];
                let swap = if swapped {
                    d[i - 2][j - 2] + 1
                } else {
                    usize::MAX
                };
                delete.min(insert).min(change).min(swap)
            };
        }
    }
    d[a.len()][b.len()]
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
