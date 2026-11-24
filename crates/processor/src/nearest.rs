//! The nearest of some names to a misspelt one, for diagnostics that suggest a name.

/// The most letters of a name compared: comparing is quadratic in them, and names have no length limit (audit S7).
const COMPARED: usize = 64;

/// The name in `names` closest to `name` by edit distance, ignoring case, if it is close enough to be a likely
/// misspelling: at most a third of the longer name's letters differ. Only the first [`COMPARED`] letters of each
/// name are compared.
pub(crate) fn nearest<'a>(name: &str, names: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    let name = letters(name);
    names
        .into_iter()
        .filter_map(|candidate| {
            let other = letters(candidate);
            if !within_reach(name.len(), other.len()) {
                return None;
            }
            let distance = distance(&name, &other);
            close_enough(distance, name.len(), other.len()).then_some((distance, candidate))
        })
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate)
}

/// The first [`COMPARED`] letters of `name`, in lower case.
fn letters(name: &str) -> Vec<char> {
    name.chars()
        .flat_map(char::to_lowercase)
        .take(COMPARED)
        .collect()
}

/// Whether a distance between names of these lengths makes a likely misspelling.
fn close_enough(distance: usize, a: usize, b: usize) -> bool {
    distance * 3 <= a.max(b)
}

/// Whether names of these lengths can be close enough at all: their distance is at least the difference in their
/// lengths, so a candidate whose length differs more is never measured.
fn within_reach(a: usize, b: usize) -> bool {
    close_enough(a.abs_diff(b), a, b)
}

/// The fewest edits that turn `a` into `b`: inserting, deleting or changing a letter, or swapping two neighbours,
/// the commonest typo (the optimal string alignment distance). Keeps three rows of the table, not all of it.
fn distance(a: &[char], b: &[char]) -> usize {
    // Rows of d, where d[i][j] is the distance between the first i letters of `a` and the first j of `b`: the row
    // two before this one, the last one, and this one.
    let mut before = vec![0; b.len() + 1];
    let mut last: Vec<usize> = (0..=b.len()).collect();
    let mut row = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        row[0] = i;
        for j in 1..=b.len() {
            let delete = last[j] + 1;
            let insert = row[j - 1] + 1;
            let change = last[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            let swapped = i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1];
            let swap = if swapped {
                before[j - 2] + 1
            } else {
                usize::MAX
            };
            row[j] = delete.min(insert).min(change).min(swap);
        }
        std::mem::swap(&mut before, &mut last);
        std::mem::swap(&mut last, &mut row);
    }
    last[b.len()]
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
        let distance = |a: &str, b: &str| distance(&letters(a), &letters(b));
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("", "ab"), 2);
        assert_eq!(distance("ab", ""), 2);
        assert_eq!(distance("ab", "ab"), 0);
        assert_eq!(distance("dco", "doc"), 1);
        assert_eq!(distance("ca", "abc"), 3);
        assert_eq!(distance("abcdef", "badcfe"), 3);
    }

    #[test]
    fn long_names_are_compared_by_their_first_letters() {
        let (unknown, known) = ("a".repeat(50_000) + "x", "a".repeat(50_000) + "y");
        assert_eq!(nearest(&unknown, [known.as_str()]), Some(known.as_str()));
        assert_eq!(letters(&unknown).len(), COMPARED);
    }

    #[test]
    fn a_candidate_whose_length_differs_too_much_is_never_measured() {
        assert!(within_reach(6, 9));
        assert!(!within_reach(6, 10));
        assert!(!within_reach(2, 64));
        assert_eq!(nearest("ab", ["x".repeat(1_000_000).as_str()]), None);
    }
}
