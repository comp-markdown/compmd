//! The preview's files: its page, and its bundle (the sandbox's frames and the manifest), which `build.rs` embeds in the
//! binary, or a test gives.

use std::borrow::Cow;
use std::collections::BTreeMap;

use compmd_host::Bundle;

/// Files by path, `/`-separated, and their bytes.
type Embedded = &'static [(&'static str, &'static [u8])];

/// What `build.rs` embedded: the page's files and the bundle's; `None` in a build without them.
const EMBEDDED: Option<(Embedded, Embedded)> = include!(concat!(env!("OUT_DIR"), "/preview.rs"));

/// The preview's page and bundle.
pub(crate) struct PreviewFiles {
    page: BTreeMap<String, Cow<'static, [u8]>>,
    /// The bundle the frames are served from.
    pub(crate) bundle: Bundle,
}

impl PreviewFiles {
    /// The files embedded in the binary: `None` in a build without them, an error when their bundle can't be used.
    pub(crate) fn embedded() -> Option<Result<Self, String>> {
        let (page, bundle) = EMBEDDED?;
        let page = page
            .iter()
            .map(|(path, bytes)| ((*path).to_owned(), Cow::Borrowed(*bytes)))
            .collect();
        Some(
            Bundle::embedded(bundle)
                .map(|bundle| Self { page, bundle })
                .map_err(|error| format!("the preview's bundle can't be used: {error}")),
        )
    }

    /// The page's `files`, by path, and `bundle`: a test's preview.
    #[cfg(test)]
    pub(crate) fn new<P: Into<String>>(
        files: impl IntoIterator<Item = (P, Vec<u8>)>,
        bundle: Bundle,
    ) -> Self {
        let page = files
            .into_iter()
            .map(|(path, bytes)| (path.into(), Cow::Owned(bytes)))
            .collect();
        Self { page, bundle }
    }

    /// The page's file at `path`.
    pub(crate) fn page_file(&self, path: &str) -> Option<&[u8]> {
        self.page.get(path).map(AsRef::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_build_rs_embedded_is_a_page_and_a_bundle_or_nothing() {
        match PreviewFiles::embedded() {
            None => assert!(EMBEDDED.is_none(), "a build without the preview"),
            Some(files) => {
                let files = files.expect("a bundle that can be used");
                assert!(files.page_file("index.html").is_some());
                assert_eq!(files.page_file("none.html"), None);
            }
        }
    }
}
