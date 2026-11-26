//! The components a document's packs export, for auto-import ([the language server], *Completion*, IN-1, decision 7):
//! for each known pack (a folder pack or an installed one of its project, then those suggested from registries), a
//! document importing it with all its names, processed, lists them with their signatures, as the processor reads the
//! pack.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use std::path::Path;
use std::rc::Rc;

use compmd::context::RenderContext;
use compmd::host::{KnownPack, Packs};
use compmd::tree::Signature;
use compmd::{Ambient, ParseOptions, Processor};
use compmd_host::ChainedPacks;
use compmd_host::pack::ReleaseId;

use crate::own_packs::NPM;
use crate::project::Project;

/// A component a pack exports.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PackComponent {
    /// The name it is exported as.
    pub(crate) name: String,
    /// What an import writes to take it: the pack and its major version, `@kit/notes@1`.
    pub(crate) from: String,
    /// Its pack's description.
    pub(crate) description: Option<String>,
    /// Its signature; `None` for a host-language component without one.
    pub(crate) signature: Option<Signature>,
}

/// The components the packs of the project at `root` export, then those of `suggested`, the packs suggested from
/// registries, which the project's hide; none when the project can't be read.
pub(crate) fn pack_components(root: &Path, suggested: Box<dyn Packs>) -> Vec<PackComponent> {
    let Ok(Project {
        listed, installed, ..
    }) = Project::read(root, NPM)
    else {
        return Vec::new();
    };
    let packs = Rc::new(ChainedPacks::new(vec![
        Box::new(listed),
        Box::new(installed),
        suggested,
    ]));
    packs
        .known()
        .iter()
        .flat_map(|known| exported(known, &packs))
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

/// The components the release `known` exports, read from `packs`, with their signatures.
fn exported(known: &KnownPack, packs: &Rc<ChainedPacks>) -> Vec<PackComponent> {
    let release = &known.release;
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
            description: known.description.clone(),
            signature: component.signature,
        })
        .collect()
}

#[cfg(test)]
mod tests;
