//! What a document's processor read ([editor support], decision 8): each file of its root folder and each pack's
//! release and file, recorded as the processor reads them, so the preview's page is given what the server's processor
//! had and processes the document the same.
//!
//! [editor support]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-support.md

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use compmd::host::{Files, KnownPack, Packs, Release};
use compmd::pack::{PackName, Place};
use serde_json::{Value, json};

/// A release read, and the files read of it.
#[derive(Debug, Default)]
struct ReadRelease {
    integrity: Option<String>,
    files: BTreeMap<String, String>,
}

/// What was read.
#[derive(Debug, Default)]
pub(crate) struct Reads {
    files: BTreeMap<String, String>,
    /// By pack, then version.
    packs: BTreeMap<String, BTreeMap<String, ReadRelease>>,
}

impl Reads {
    /// The files read, by path.
    pub(crate) fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }

    /// The packs read, as a processor's request gives them: each pack's releases with the files read of them.
    pub(crate) fn packs(&self) -> Value {
        let packs = self.packs.iter().map(|(pack, releases)| {
            let releases: Vec<Value> = releases
                .iter()
                .filter(|(_, release)| !release.files.is_empty())
                .map(|(version, release)| {
                    json!({ "version": version, "integrity": release.integrity, "files": release.files })
                })
                .collect();
            (pack.clone(), Value::from(releases))
        });
        Value::Object(
            packs
                .filter(|(_, releases)| {
                    releases
                        .as_array()
                        .is_some_and(|releases| !releases.is_empty())
                })
                .collect(),
        )
    }
}

/// `T`, a processor's files or packs, recording what is read of it in `reads`.
pub(crate) struct Recorded<T> {
    inner: T,
    reads: Rc<RefCell<Reads>>,
}

impl<T> Recorded<T> {
    /// `inner`, recorded into `reads`.
    pub(crate) fn new(inner: T, reads: &Rc<RefCell<Reads>>) -> Self {
        Self {
            inner,
            reads: Rc::clone(reads),
        }
    }
}

impl<T: Files> Files for Recorded<T> {
    fn file(&self, path: &str) -> Option<String> {
        let text = self.inner.file(path)?;
        self.reads
            .borrow_mut()
            .files
            .insert(path.to_owned(), text.clone());
        Some(text)
    }
}

impl<T: Packs> Packs for Recorded<T> {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        let releases = self.inner.releases(pack);
        let mut reads = self.reads.borrow_mut();
        let read = reads.packs.entry(pack.as_str().to_owned()).or_default();
        for release in &releases {
            read.entry(release.version.to_string())
                .or_default()
                .integrity = release.integrity.as_ref().map(ToString::to_string);
        }
        releases
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        let text = self.inner.pack_file(place)?;
        let mut reads = self.reads.borrow_mut();
        let release = &place.release;
        reads
            .packs
            .entry(release.pack.as_str().to_owned())
            .or_default()
            .entry(release.version.to_string())
            .or_default()
            .files
            .insert(place.path.as_str().to_owned(), text.clone());
        Some(text)
    }

    fn pending(&self, pack: &PackName) -> bool {
        self.inner.pending(pack)
    }

    fn known(&self) -> Vec<KnownPack> {
        self.inner.known()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use compmd::host::LoadedRelease;

    use super::*;

    #[test]
    fn the_files_and_the_pack_files_read_are_recorded() {
        let reads = Rc::new(RefCell::new(Reads::default()));
        let files = Recorded::new(
            HashMap::from([("tide.csv".to_owned(), "level\n2\n".to_owned())]),
            &reads,
        );
        assert_eq!(files.file("tide.csv").as_deref(), Some("level\n2\n"));
        assert_eq!(files.file("none.csv"), None);
        let release: LoadedRelease = serde_json::from_value(json!({
            "version": "1.0.0", "integrity": "sha512-abc",
            "files": { "package.json": "{}", "index.comp.md": "# Notes", "unread.md": "x" }
        }))
        .expect("a release");
        let packs = Recorded::new(
            HashMap::from([("@kit/notes".to_owned(), vec![release])]),
            &reads,
        );
        let notes: PackName = "@kit/notes".parse().expect("a name");
        assert_eq!(packs.releases(&notes).len(), 1);
        assert_eq!(packs.releases(&"@kit/none".parse().expect("a name")), []);
        let place = "@kit/notes@1.0.0/index.comp.md".parse().expect("a place");
        assert_eq!(packs.pack_file(&place).as_deref(), Some("# Notes"));
        assert!(!packs.pending(&notes));
        assert_eq!(packs.known().len(), 1);
        let reads = reads.borrow();
        assert_eq!(reads.files().keys().collect::<Vec<_>>(), ["tide.csv"]);
        assert_eq!(
            reads.packs(),
            json!({ "@kit/notes": [{ "version": "1.0.0", "integrity": "sha512-abc", "files": { "index.comp.md": "# Notes" } }] }),
            "a pack whose releases were only listed isn't given"
        );
    }
}
