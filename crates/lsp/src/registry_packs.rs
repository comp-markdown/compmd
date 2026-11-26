//! The server's thread for registry packs ([the language server], decisions 2, 6 and 7): each document's worker sends
//! it the packs its document imports that nothing holds, and its suggested scopes; it resolves them on the registry
//! rules, from the user's cache or the registries the document's configuration names, then sends the worker what it
//! got. One thread for the server, so a release is fetched once however many documents import it.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use compmd_host::pack::PackName;
use compmd_host::registry::{Client, FolderCache, Needed, Options, Resolved, resolve};
use crossbeam_channel::{Sender, unbounded};

use crate::worker::Job;

/// How long one request to a registry may take, its body read whole.
const TIMEOUT: Duration = Duration::from_secs(30);

/// What a worker asks the thread.
enum Asked {
    /// The packs its document imports that nothing holds.
    Packs {
        /// How its document's configuration resolves them.
        options: Options,
        packs: Vec<Needed>,
        /// Where the worker takes what was got.
        reply: Sender<Job>,
    },
    /// The latest release of each pack its suggested scopes list.
    Scopes {
        /// How its document's configuration resolves packs: a scope it names no registry for is npm's.
        options: Options,
        scopes: Vec<String>,
        reply: Sender<Job>,
    },
}

/// The fetch thread, as the server and its workers reach it.
#[derive(Clone)]
pub(crate) struct RegistryPacks {
    asked: Sender<Asked>,
    /// Whether the thread may reach the registries, or only the cache (the `compmd.packs.network` setting).
    network: Arc<AtomicBool>,
    /// The writer's own suggested scopes (the `compmd.packs.suggest` setting).
    suggest: Arc<Mutex<Vec<String>>>,
    /// The registry of CompMD's own packs when a configuration names none for them, and of a suggested scope it names
    /// none for: npm's.
    own: Arc<str>,
}

/// The user's cache folder for packs: `compmd/packs` in the platform's cache directory, from its environment
/// (`XDG_CACHE_HOME` or `~/.cache`, `~/Library/Caches`, `%LOCALAPPDATA%`); in the temporary folder without one.
pub(crate) fn user_cache() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let platform = if cfg!(target_os = "macos") {
        home.map(|home| home.join("Library").join("Caches"))
    } else if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|home| home.join(".cache")))
    };
    platform
        .unwrap_or_else(std::env::temp_dir)
        .join("compmd")
        .join("packs")
}

/// Each scope's packs as its registry's search listed them this session, by the registry's URL and the scope: a
/// scope is searched once however many documents suggest it.
type Searched = HashMap<String, Vec<PackName>>;

/// The latest release of each pack `scopes` list, each from its registry in `options` (else `npm`): listed by the
/// registry's search (once a session, `searched`), or by what `cache` kept when there is no `client` or the search
/// fails. Gives what was got, and why a scope couldn't be searched, for the log.
fn suggested(
    mut options: Options,
    scopes: &[String],
    searched: &mut Searched,
    (client, cache, npm): (Option<&Client>, &FolderCache, &str),
) -> (Resolved, Vec<String>) {
    let mut unsearched = Vec::new();
    let mut needed = Vec::new();
    for scope in scopes {
        let registry = options.registry_of_scope(scope).unwrap_or(npm).to_owned();
        let key = format!("{registry}{scope}");
        let listed = match (searched.get(&key), client) {
            (Some(listed), _) => listed.clone(),
            (None, Some(client)) => match client.search(&registry, scope) {
                Ok(listed) => {
                    searched.insert(key, listed.clone());
                    listed
                }
                Err(why) => {
                    unsearched.push(format!("`{scope}` couldn't be searched: {why}"));
                    cache.packs_in(&registry, scope)
                }
            },
            (None, None) => cache.packs_in(&registry, scope),
        };
        options.registries.entry(scope.clone()).or_insert(registry);
        needed.extend(listed.into_iter().map(|pack| Needed {
            pack,
            range: "*".into(),
            integrity: None,
            library: false,
        }));
    }
    let mut first = Some(needed);
    let resolved = resolve(options, client, cache, |_, _| {
        first.take().unwrap_or_default()
    });
    (resolved, unsearched)
}

impl RegistryPacks {
    /// Starts the thread, its cache in `cache`, CompMD's own packs from `own` when a configuration names no registry
    /// for them.
    pub(crate) fn start(cache: PathBuf, own: &str) -> Self {
        let (asked, received) = unbounded::<Asked>();
        let network = Arc::new(AtomicBool::new(true));
        let online = Arc::clone(&network);
        let npm: Arc<str> = own.into();
        let registry = Arc::clone(&npm);
        thread::spawn(move || {
            let (client, cache) = (Client::new(TIMEOUT), FolderCache::new(cache));
            let mut searched = Searched::new();
            for asked in received {
                let reachable = online.load(Ordering::Relaxed).then_some(&client);
                match asked {
                    Asked::Packs {
                        options,
                        packs,
                        reply,
                    } => {
                        let names: Vec<PackName> =
                            packs.iter().map(|needed| needed.pack.clone()).collect();
                        let mut first = Some(packs);
                        let resolved = resolve(options, reachable, &cache, |_, _| {
                            first.take().unwrap_or_default()
                        });
                        let _ = reply.send(Job::Fetched {
                            packs: names,
                            resolved,
                        });
                    }
                    Asked::Scopes {
                        options,
                        scopes,
                        reply,
                    } => {
                        let (resolved, unsearched) = suggested(
                            options,
                            &scopes,
                            &mut searched,
                            (reachable, &cache, &registry),
                        );
                        let _ = reply.send(Job::Suggested {
                            resolved,
                            unsearched,
                        });
                    }
                }
            }
        });
        Self {
            asked,
            network,
            suggest: Arc::default(),
            own: npm,
        }
    }

    /// The registry of CompMD's own packs when a configuration names none for them.
    pub(crate) fn own(&self) -> &str {
        &self.own
    }

    /// Asks for `packs`, resolved with `options`; what was got comes to `reply` as [`Job::Fetched`].
    pub(crate) fn fetch(&self, options: Options, packs: Vec<Needed>, reply: Sender<Job>) {
        let _ = self.asked.send(Asked::Packs {
            options,
            packs,
            reply,
        });
    }

    /// Asks for the packs `scopes` list, resolved with `options`; what was got comes to `reply` as
    /// [`Job::Suggested`].
    pub(crate) fn suggest(&self, options: Options, scopes: Vec<String>, reply: Sender<Job>) {
        let _ = self.asked.send(Asked::Scopes {
            options,
            scopes,
            reply,
        });
    }

    /// Lets the thread reach the registries, or only the cache.
    pub(crate) fn set_network(&self, network: bool) {
        self.network.store(network, Ordering::Relaxed);
    }

    /// The writer's own suggested scopes, in place of those before; gives whether they changed.
    pub(crate) fn set_writer_scopes(&self, scopes: Vec<String>) -> bool {
        let mut held = self.suggest.lock().unwrap_or_else(PoisonError::into_inner);
        let changed = *held != scopes;
        *held = scopes;
        changed
    }

    /// The writer's own suggested scopes.
    pub(crate) fn writer_scopes(&self) -> Vec<String> {
        self.suggest
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_is_compmds_packs_folder_in_the_users_cache() {
        assert!(user_cache().ends_with("compmd/packs"));
    }

    #[test]
    fn the_writers_scopes_are_kept_saying_whether_they_changed() {
        let registry = RegistryPacks::start(
            std::env::temp_dir().join("compmd-lsp-scopes-cache"),
            "http://127.0.0.1:9/",
        );
        assert_eq!(registry.writer_scopes(), Vec::<String>::new());
        assert!(registry.set_writer_scopes(vec!["@acme".into()]));
        assert!(!registry.set_writer_scopes(vec!["@acme".into()]));
        assert_eq!(registry.writer_scopes(), ["@acme"]);
    }
}
