//! A relative destination of a link or an image in the expanded tree (spec §8.6, §12.3): written in another file of
//! the root folder, it is made relative to the processed document; in a pack, it is the file's place, marked as a
//! pack's.

use crate::resolve::Origin;
use compmd_pack::FilePath;

/// A destination as the expanded tree writes it.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Destination {
    /// The URL.
    pub(super) url: String,
    /// Whether `url` is the place of a pack's file.
    pub(super) pack: bool,
}

/// `url`, written in the file `base`, as the expanded tree of the document `main` writes it; `None` when its path
/// leaves `base`'s root or isn't a path (F026). A URL with a scheme, one from the root of its host (`/x`), a fragment
/// or a query alone, and one written in `main` itself are kept.
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
    match (base, main) {
        (Origin::File(file), Origin::File(main)) => in_root_folder(file, main, path, rest),
        (Origin::Pack(place), _) => {
            let place = place.join(path)?;
            let slash = if path.ends_with('/') { "/" } else { "" };
            Some(Destination {
                url: format!("{place}{slash}{rest}"),
                pack: true,
            })
        }
        _ => base.join(path).ok().map(|_| kept()),
    }
}

/// The destination `path` (with `rest`, its query and fragment), written in `file` of the root folder, seen from
/// `main`: a file, or a folder (`./`, `../guide/`), which a link may name in a root folder.
fn in_root_folder(file: &FilePath, main: &FilePath, path: &str, rest: &str) -> Option<Destination> {
    let target = if names_a_folder(path) {
        let folder = file.join_folder(path)?;
        if folder.is_empty() {
            folder
        } else {
            format!("{folder}/")
        }
    } else {
        file.join(path)?.as_str().to_string()
    };
    Some(Destination {
        url: format!("{}{rest}", from_folder_of(main.as_str(), &target)),
        pack: false,
    })
}

/// Whether the relative `path` names a folder: it ends with `/`, `.` or `..`.
fn names_a_folder(path: &str) -> bool {
    path.ends_with('/') || matches!(path.rsplit('/').next(), Some("." | ".."))
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

/// The path of `file` seen from the folder of `from`, both `/`-separated paths from the same root; a folder `file`
/// ends with `/`, and is empty for the root.
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
        Origin::main(path)
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
        assert_eq!(url("./..\\x.svg", &file("a/b.comp.md"), &main), None);
    }

    #[test]
    fn a_folder_of_the_root_folder_is_seen_from_the_processed_document() {
        let main = file("posts/harbour.comp.md");
        let part = file("parts/cards.comp.md");
        assert_eq!(url("./", &part, &main), Some(("../parts/".into(), false)));
        assert_eq!(
            url("../#top", &part, &main),
            Some(("../#top".into(), false))
        );
        assert_eq!(url("..", &part, &main), Some(("../".into(), false)));
        assert_eq!(url("../posts/.", &part, &main), Some(("./".into(), false)));
        assert_eq!(
            url("./", &part, &file("index.comp.md")),
            Some(("./parts/".into(), false))
        );
        assert_eq!(
            url("../", &part, &file("index.comp.md")),
            Some(("./".into(), false))
        );
        assert_eq!(url("../../", &part, &main), None, "it leaves the root");
        assert_eq!(url("./a:b/", &part, &main), None, "a name with `:`");
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
