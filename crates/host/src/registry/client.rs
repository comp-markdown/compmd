//! The registry rules' network steps over npm's protocol, with `ureq` ([the language server], decision 6): a host in
//! Rust that fetches from registries itself answers [`super::Step::Versions`] and [`super::Step::Tarball`] with these.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::time::Duration;

use compmd_pack::PackName;
use ureq::Agent;

use super::metadata::{ABBREVIATED, VersionInfo, answered_versions, metadata_url, off_origin};
use super::step::Answer;

/// A client of npm registries: blocking, so a host runs it on a thread of its own.
#[derive(Clone, Debug)]
pub struct Client {
    agent: Agent,
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

    /// `pack`'s versions at `registry`, from its abbreviated metadata: [`Answer::Versions`], or [`Answer::Failed`].
    #[must_use]
    pub fn versions(&self, registry: &str, pack: &PackName) -> Answer {
        let url = metadata_url(registry, pack);
        let fetched = self
            .agent
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
                Ok((status, cache_control, body))
            });
        let read = fetched.map_err(|error| format!("{url}: {error}")).and_then(
            |(status, cache_control, body)| {
                let versions = answered_versions(registry, pack, status, &body)?;
                Ok((versions, max_age_of(cache_control.as_deref())))
            },
        );
        match read {
            Ok((versions, max_age)) => Answer::Versions { versions, max_age },
            Err(message) => Answer::Failed { message },
        }
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
}
