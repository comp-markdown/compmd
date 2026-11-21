//! A relative destination of a link or an image in the expanded tree (spec §8.6, §12.3): written in another file of
//! the root folder, it is made relative to the processed document; in a pack, it is the file's place, marked as a
//! pack's.

use crate::resolve::Origin;

/// A destination as the expanded tree writes it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Destination {
    /// The URL.
    pub(super) url: String,
    /// Whether `url` is the place of a pack's file.
    pub(super) pack: bool,
}

/// `url`, written in the file `base`, as the expanded tree of the document `main` writes it; `None` when its path
/// leaves `base`'s root (F026). A URL with a scheme, one from the root of its host (`/x`), a fragment or a query
/// alone, and one written in `main` itself are kept.
pub(super) fn resolved(url: &str, base: &Origin, main: &Origin) -> Option<Destination> {
    let kept = || Destination {
        url: url.to_string(),
        pack: false,
    };
    if base == main || !is_relative(url) {
        return Some(kept());
    }
    let split = url.find(['?', '#']).unwrap_or(url.len());
    let (path, rest) = url.split_at(split);
    let slash = if path.ends_with('/') { "/" } else { "" };
    Some(match (base.join(path)?, main) {
        (Origin::Pack(place), _) => Destination {
            url: format!("{place}{slash}{rest}"),
            pack: true,
        },
        (Origin::File(file), Origin::File(main)) => Destination {
            url: format!("{}{slash}{rest}", from_folder_of(main, &file)),
            pack: false,
        },
        // A pack's file can't name a file of a root folder: `join` stays in the pack.
        (Origin::File(_), Origin::Pack(_)) => kept(),
    })
}

/// Whether `url` is a relative path: no scheme, not from the root of its host, not a fragment or a query alone.
fn is_relative(url: &str) -> bool {
    let has_scheme = url.split_once(':').is_some_and(|(scheme, _)| {
        let mut chars = scheme.chars();
        chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    !(url.is_empty() || has_scheme || url.starts_with(['/', '#', '?']))
}

/// The path of `file` seen from the folder of `from`, both `/`-separated paths from the same root.
fn from_folder_of(from: &str, file: &str) -> String {
    let folder: Vec<&str> = from.split('/').collect();
    let folder = &folder[..folder.len() - 1];
    let file: Vec<&str> = file.split('/').collect();
    let shared = folder
        .iter()
        .zip(&file[..file.len() - 1])
        .take_while(|(a, b)| a == b)
        .count();
    let up = folder.len() - shared;
    let mut parts = vec![".."; up];
    parts.extend(&file[shared..]);
    let path = parts.join("/");
    if up == 0 { format!("./{path}") } else { path }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> Origin {
        Origin::File(path.to_string())
    }

    fn pack(place: &str) -> Origin {
        Origin::Pack(place.parse().expect("a place"))
    }

    fn url(written: &str, base: &Origin, main: &Origin) -> Option<(String, bool)> {
        resolved(written, base, main).map(|destination| (destination.url, destination.pack))
    }

    #[test]
    fn a_pack_s_file_is_its_place() {
        let kit = pack("@acme/kit@1.4.2/cards/card.comp.md");
        let main = file("post.comp.md");
        assert_eq!(
            url("./icons/star.svg", &kit, &main),
            Some(("@acme/kit@1.4.2/cards/icons/star.svg".into(), true))
        );
        assert_eq!(
            url("../guide.comp.md#start", &kit, &main),
            Some(("@acme/kit@1.4.2/guide.comp.md#start".into(), true))
        );
        assert_eq!(
            url("docs/", &kit, &main),
            Some(("@acme/kit@1.4.2/cards/docs/".into(), true))
        );
        assert_eq!(url("../../x.svg", &kit, &main), None, "leaves the pack");
    }

    #[test]
    fn a_file_of_the_root_folder_is_seen_from_the_processed_document() {
        let main = file("posts/harbour.comp.md");
        assert_eq!(
            url("./icons/star.svg", &file("parts/cards.comp.md"), &main),
            Some(("../parts/icons/star.svg".into(), false))
        );
        assert_eq!(
            url("star.svg?v=2", &file("posts/parts/cards.comp.md"), &main),
            Some(("./parts/star.svg?v=2".into(), false))
        );
        assert_eq!(
            url("../star.svg", &file("posts/a/b.comp.md"), &main),
            Some(("./star.svg".into(), false))
        );
        assert_eq!(url("../../x.svg", &file("a/b.comp.md"), &main), None);
    }

    #[test]
    fn what_isn_t_a_relative_path_from_another_file_is_kept() {
        let main = file("post.comp.md");
        let kit = pack("@acme/kit@1.4.2/kit.comp.md");
        for written in [
            "https://example.com/a.png",
            "mailto:a@b.c",
            "/root.png",
            "#top",
            "?q",
            "",
        ] {
            assert_eq!(url(written, &kit, &main), Some((written.into(), false)));
        }
        assert_eq!(
            url("./x/../a.png", &main, &main),
            Some(("./x/../a.png".into(), false))
        );
        assert_eq!(
            url("./a.png", &file("x.comp.md"), &kit),
            Some(("./a.png".into(), false))
        );
    }
}
