//! The packs a document imports (spec §6.4): their releases, checked against the import's `integrity` (F009, F035),
//! the host's policy (F026) and their `package.json` (F022), and what they need from the host (W121).

use super::{Loader, Refusal};
use crate::diagnostic::{Code, Location};
use crate::host::NeededPack;
use crate::resolve::libraries::Ask;
use crate::resolve::source::PackSpec;
use crate::resolve::{DocumentId, Origin};
use crate::settings::Import;
use compmd_pack::{FilePath, PackName, Place, ReleaseId};
use semver::VersionReq;
use serde_json::Value;

impl Loader<'_, '_> {
    /// The pack `spec` names: its newest matching release, checked against the import's `integrity`.
    pub(super) fn pack(
        &mut self,
        id: DocumentId,
        spec: &PackSpec,
        import: &Import,
    ) -> Result<DocumentId, Refusal> {
        let from = &import.from;
        if let Some(why) = self.policy.refusal(&spec.name, &self.ambient_names) {
            return Err(Refusal::new(
                Code::F026,
                from.at,
                format!("{why}; the import is dropped"),
            ));
        }
        let Some(release) = self
            .store
            .releases(&spec.name)
            .into_iter()
            .filter(|release| spec.range.matches(&release.version))
            .max_by(|a, b| a.version.cmp(&b.version))
        else {
            self.missing(spec, import);
            return Err(Refusal::new(
                Code::F026,
                from.at,
                format!("no release of `{}` matches `{}`", spec.name, from.value),
            ));
        };
        if let Some(why) = self.policy.denial(&spec.name, &release.version) {
            return Err(Refusal::new(
                Code::F026,
                from.at,
                format!("{why}; the import is dropped"),
            ));
        }
        if let Some(integrity) = &import.integrity {
            if !spec.exact {
                self.report(
                    id,
                    Code::F009,
                    integrity.at,
                    "`integrity` needs an exact version, such as `@1.4.2`, since a range takes whichever release is newest; ignored",
                );
            } else if release
                .integrity
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                != Some(integrity.value.as_str())
            {
                return Err(Refusal::new(
                    Code::F035,
                    integrity.at,
                    format!(
                        "`{}@{}` doesn't match this `integrity`; the import is dropped",
                        spec.name, release.version
                    ),
                ));
            }
        }
        if let Some(why) = &release.broken {
            return Err(Refusal::new(
                Code::F035,
                from.at,
                format!(
                    "`{}@{}`: {why}; the import is dropped",
                    spec.name, release.version
                ),
            ));
        }
        let release = ReleaseId::new(spec.name.clone(), release.version);
        let document = self.package(&release, from.at, false)?;
        self.check_needs(id, &release, from.at);
        Ok(document)
    }

    /// Whether the pack `spec` names is pending (spec §6.4): the host is still getting it, holds no release that
    /// matches, and its policy doesn't refuse it, which would make it F026 whenever it came.
    pub(super) fn pending(&self, spec: &PackSpec) -> bool {
        self.store.pending(&spec.name)
            && self
                .policy
                .refusal(&spec.name, &self.ambient_names)
                .is_none()
            && !self
                .store
                .releases(&spec.name)
                .iter()
                .any(|release| spec.range.matches(&release.version))
    }

    /// Records the pack `spec` names as one the host has no release for, once.
    pub(super) fn missing(&mut self, spec: &PackSpec, import: &Import) {
        let needed = NeededPack {
            pack: spec.name.clone(),
            range: spec.written.clone(),
            integrity: import
                .integrity
                .as_ref()
                .filter(|_| spec.exact)
                .and_then(|integrity| integrity.value.parse().ok()),
            library: false,
        };
        if !self.missing.contains(&needed) {
            self.missing.push(needed);
        }
    }

    /// W121: what the pack's `release` needs from the host, its `peerDependencies` (spec §6.4), that the host's
    /// policy can't meet, reported on the import at `at`; the library packs it names are asked for, and chosen once
    /// the document is loaded. A range this crate can't read isn't checked.
    fn check_needs(&mut self, id: DocumentId, release: &ReleaseId, at: Location) {
        let needs = self
            .manifest(release)
            .and_then(|manifest| manifest.get("peerDependencies")?.as_object().cloned())
            .unwrap_or_default();
        let mut unmet = Vec::new();
        for (need, range) in &needs {
            let Some(range) = range.as_str() else {
                continue;
            };
            if self.policy.provides(need) {
                let range = VersionReq::parse(range).ok();
                unmet.extend(range.and_then(|range| self.policy.unmet(need, &range)));
            } else if let Some(ask) = Ask::new(release.clone(), need, range, (id, at))
                && !self.asks.contains(&ask)
            {
                self.asks.push(ask);
            }
        }
        for missing in unmet {
            self.unmet(id, release, at, &missing);
        }
    }

    /// W121 on the import at `at`: the pack's or library's `release` needs `missing`.
    pub(super) fn unmet(
        &mut self,
        id: DocumentId,
        release: &ReleaseId,
        at: Location,
        missing: &str,
    ) {
        self.report(
            id,
            Code::W121,
            at,
            format!(
                "`{release}` needs {missing}, so its components that use it render their fallbacks"
            ),
        );
    }

    /// The `package.json` of `release`, when there is one and it is JSON.
    fn manifest(&self, release: &ReleaseId) -> Option<Value> {
        let path = FilePath::new("package.json").ok()?;
        let text = self
            .store
            .read(&Origin::Pack(Place::new(release.clone(), path)))?;
        serde_json::from_str(&text).ok()
    }

    /// The newest release of the ambient pack `name`.
    pub(super) fn ambient_pack(&mut self, name: &PackName) -> Option<DocumentId> {
        let release = self
            .store
            .releases(name)
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))?;
        let release = ReleaseId::new(name.clone(), release.version);
        self.package(&release, Location::new(1, 1), true).ok()
    }

    /// The document of a pack's `release`, named by its `package.json` (spec §6.4), checked against it (F022).
    fn package(
        &mut self,
        release: &ReleaseId,
        at: Location,
        provided: bool,
    ) -> Result<DocumentId, Refusal> {
        let unloadable = || Refusal::new(Code::F026, at, format!("`{release}` can't be loaded"));
        let manifest = self.manifest(release).ok_or_else(unloadable)?;
        let path = manifest
            .get("compmd")
            .and_then(Value::as_str)
            .map_or("index.comp.md", |path| path.trim_start_matches("./"));
        let path = FilePath::new(path).map_err(|_| unloadable())?;
        let origin = Origin::Pack(Place::new(release.clone(), path));
        let fresh = !self.loaded.contains_key(&origin);
        let id = self.document_at(origin, at, provided)?;
        if fresh {
            self.check_manifest(id, &manifest);
        }
        Ok(id)
    }

    /// F022: a pack's `export.pack` and `export.version` must be its manifest's.
    fn check_manifest(&mut self, id: DocumentId, manifest: &Value) {
        let Some(export) = self.document(id).settings.export.clone() else {
            return;
        };
        let differs = |written: &Option<String>, key: &str| {
            written
                .as_deref()
                .is_some_and(|written| manifest.get(key).and_then(Value::as_str) != Some(written))
        };
        if differs(&export.pack, "name") || differs(&export.version, "version") {
            self.report(
                id,
                Code::F022,
                export.at,
                "`export.pack` and `export.version` must match the pack's `package.json`, which is used",
            );
        }
    }
}
