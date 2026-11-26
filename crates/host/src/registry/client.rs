//! The registry rules' network steps over npm's protocol, with `ureq` ([the language server], decision 6): a host in
//! Rust that fetches from registries itself answers [`super::Step::Versions`] and [`super::Step::Tarball`] with these.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::time::Duration;

use compmd_pack::PackName;
use ureq::Agent;

use super::metadata::{ABBREVIATED, VersionInfo, answered_versions, metadata_url, off_origin};
use super::search::{search_path, searched};
use super::step::Answer;

/// A client of npm registries: blocking, so a host runs it on a thread of its own.
#[derive(Clone, Debug)]
pub struct Client {
    agent: Agent,
}

/// A registry's answer to a request for a pack's metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Its status: 404 when it doesn't have the pack.
    pub status: u16,
    /// How long it may be kept, in seconds: its `Cache-Control`'s `max-age`, 0 without one.
    pub max_age: u64,
    /// Its body.
    pub body: String,
}

/// The `max-age` of a `Cache-Control` header, in seconds: 0 without one.
fn max_age_of(cache_control: Option<&str>) -> u64 {
    cache_control
        .into_iter()
        .flat_map(|header| header.split(','))
        .find_map(|directive| directive.trim().strip_prefix("max-age=")?.parse().ok())
        .unwrap_or(0)
}

impl Client {
    /// A client whose every request, its body read whole, takes at most `timeout`.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(timeout))
            .http_status_as_error(false)
            .build();
        Self {
            agent: config.into(),
        }
    }

    /// `pack`'s abbreviated metadata at `registry`, as it answered.
    ///
    /// # Errors
    ///
    /// When the registry gives no answer, or its body can't be read.
    pub fn metadata(&self, registry: &str, pack: &PackName) -> Result<Metadata, String> {
        let url = metadata_url(registry, pack);
        self.agent
            .get(&url)
            .header("Accept", ABBREVIATED)
            .call()
            .and_then(|mut response| {
                let status = response.status().as_u16();
                let cache_control = response
                    .headers()
                    .get("cache-control")
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                let body = response.body_mut().read_to_string()?;
                Ok(Metadata {
                    status,
                    max_age: max_age_of(cache_control.as_deref()),
                    body,
                })
            })
            .map_err(|error| format!("{url}: {error}"))
    }

    /// `pack`'s versions at `registry`, from its abbreviated metadata: [`Answer::Versions`], or [`Answer::Failed`].
    #[must_use]
    pub fn versions(&self, registry: &str, pack: &PackName) -> Answer {
        let read = self.metadata(registry, pack).and_then(|metadata| {
            let versions = answered_versions(registry, pack, metadata.status, &metadata.body)?;
            Ok((versions, metadata.max_age))
        });
        match read {
            Ok((versions, max_age)) => Answer::Versions { versions, max_age },
            Err(message) => Answer::Failed { message },
        }
    }

    /// The packs of `scope` (`@kit`) that `registry`'s search lists ([`search_path`]), in its order.
    ///
    /// # Errors
    ///
    /// When the registry gives no answer, answers with an error (a registry without search does), or its answer
    /// isn't a search's.
    pub fn search(&self, registry: &str, scope: &str) -> Result<Vec<PackName>, String> {
        let url = format!("{registry}{}", search_path(scope));
        let mut response = self
            .agent
            .get(&url)
            .call()
            .map_err(|error| format!("{url}: {error}"))?;
        let status = response.status().as_u16();
        if !(200..=299).contains(&status) {
            return Err(format!("{url} answered {status}"));
        }
        let body = response
            .body_mut()
            .read_to_string()
            .map_err(|error| format!("{url}: {error}"))?;
        searched(&body, scope)
    }

    /// `version`'s tarball from `registry`'s origin, at most `max_bytes` of it: [`Answer::Tarball`], or
    /// [`Answer::Failed`].
    #[must_use]
    pub fn tarball(&self, registry: &str, version: &VersionInfo, max_bytes: u64) -> Answer {
        let url = &version.tarball;
        if let Some(message) = off_origin(url, registry) {
            return Answer::Failed { message };
        }
        let fetched = self.agent.get(url).call().and_then(|mut response| {
            let status = response.status().as_u16();
            let bytes = response
                .body_mut()
                .with_config()
                .limit(max_bytes)
                .read_to_vec()?;
            Ok((status, bytes))
        });
        match fetched {
            Ok((200..=299, bytes)) => Answer::Tarball { bytes },
            Ok((status, _)) => Answer::Failed {
                message: format!("{url} answered {status}"),
            },
            Err(error) => Answer::Failed {
                message: format!("{url}: {error}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cache_control_gives_its_max_age() {
        assert_eq!(max_age_of(Some("public, max-age=300")), 300);
        assert_eq!(max_age_of(Some("no-cache")), 0);
        assert_eq!(max_age_of(Some("max-age=soon")), 0);
        assert_eq!(max_age_of(None), 0);
    }

    #[test]
    fn a_scopes_search_lists_its_packs_with_the_keyword() {
        use crate::registry::{LocalRegistry, ServedPack};

        let manifest = |keywords: &str| format!(r#"{{"name": "x", "keywords": {keywords}}}"#);
        let (pack, other) = (manifest(r#"["compmd-pack"]"#), manifest("[]"));
        let served = [
            ServedPack::new("@kit/timeline").version("1.0.0", &[("package.json", &pack)]),
            ServedPack::new("@kit/tool").version("1.0.0", &[("package.json", &other)]),
            ServedPack::new("@acme/maps").version("1.0.0", &[("package.json", &pack)]),
        ];
        let registry = LocalRegistry::serve(&served).expect("served");
        let client = Client::new(Duration::from_secs(10));
        let names = |scope| {
            client
                .search(registry.url(), scope)
                .map(|found| found.iter().map(ToString::to_string).collect::<Vec<_>>())
        };
        assert_eq!(names("@kit"), Ok(vec!["@kit/timeline".to_owned()]));
        assert_eq!(names("@acme"), Ok(vec!["@acme/maps".to_owned()]));
        assert!(names("@none").is_err_and(|message| message.ends_with("answered 404")));
        drop(registry);
        let gone = LocalRegistry::serve(&[]).expect("served");
        let url = gone.url().to_owned();
        drop(gone);
        assert!(client.search(&url, "@kit").is_err(), "no answer");
    }
}
