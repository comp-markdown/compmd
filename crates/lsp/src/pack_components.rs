//! The components a document's packs export, for auto-import ([the language server], *Completion*, IN-1): for each
//! pack its project has (a folder pack or an installed one), a document importing it with all its names, processed,
//! lists them, as the processor reads the pack.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use std::path::Path;
use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::{Ambient, ParseOptions, Processor};
use compmd_host::ChainedPacks;
use compmd_host::pack::ReleaseId;

use crate::project::Project;

/// A component a pack exports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PackComponent {
    /// The name it is exported as.
    pub(crate) name: String,
    /// What an import writes to take it: the pack and its major version, `@kit/notes@1`.
    pub(crate) from: String,
}

/// The components the packs of the project at `root` export; none when it can't be read.
pub(crate) fn pack_components(root: &Path) -> Vec<PackComponent> {
    let Ok(Project {
        listed, installed, ..
    }) = Project::read(root)
    else {
        return Vec::new();
    };
    let releases: Vec<ReleaseId> = listed
        .packs()
        .iter()
        .chain(installed.packs())
        .map(|pack| pack.release.clone())
        .collect();
    let packs = Rc::new(ChainedPacks::new(vec![
        Box::new(listed),
        Box::new(installed),
    ]));
    releases
        .iter()
        .flat_map(|release| exported(release, &packs))
        .collect()
}

/// What an import writes to take `release`: its major version, or its minor one below 1.0.0, as the spec's imports
/// write them.
fn from_of(release: &ReleaseId) -> String {
    let version = &release.version;
    if version.major == 0 {
        format!("{}@0.{}", release.pack, version.minor)
    } else {
        format!("{}@{}", release.pack, version.major)
    }
}

/// The components `release` exports, read from `packs`.
fn exported(release: &ReleaseId, packs: &Rc<ChainedPacks>) -> Vec<PackComponent> {
    let text = format!("---\nmd:\n  import:\n    - from: \"{release}\"\n      names: \"*\"\n---\n");
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let processor = Processor::new("probe.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve((), Rc::clone(packs), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&compmd::expand::Options::default());
    let place = format!("{release}/");
    let from = from_of(release);
    processor
        .components()
        .groups
        .into_iter()
        .filter(|group| {
            group
                .file
                .as_deref()
                .is_some_and(|file| file.starts_with(&place))
        })
        .flat_map(|group| group.components)
        .map(|component| PackComponent {
            name: component.name,
            from: from.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests;
