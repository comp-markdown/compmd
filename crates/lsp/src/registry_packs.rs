//! The server's thread for registry packs ([the language server], decisions 2 and 6): each document's worker sends it
//! the packs its document imports that nothing holds, and it resolves them on the registry rules, from the user's cache
//! or the registries the document's configuration names, then sends the worker what it got. One thread for the
//! server, so a release is fetched once however many documents import it.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use compmd_host::pack::PackName;
use compmd_host::registry::{Client, FolderCache, Needed, Options, resolve};
use crossbeam_channel::{Sender, unbounded};

use crate::worker::Job;

/// How long one request to a registry may take, its body read whole.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Packs a worker asks for.
struct Asked {
    /// How its document's configuration resolves them.
    options: Options,
    /// The packs.
    packs: Vec<Needed>,
    /// Where the worker takes what was got.
    reply: Sender<Job>,
}

/// The fetch thread, as the server and its workers reach it.
#[derive(Clone)]
pub(crate) struct RegistryPacks {
    asked: Sender<Asked>,
    /// Whether the thread may reach the registries, or only the cache (the `compmd.packs.network` setting).
    network: Arc<AtomicBool>,
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

impl RegistryPacks {
    /// Starts the thread, its cache in `cache`.
    pub(crate) fn start(cache: PathBuf) -> Self {
        let (asked, received) = unbounded::<Asked>();
        let network = Arc::new(AtomicBool::new(true));
        let online = Arc::clone(&network);
        thread::spawn(move || {
            let (client, cache) = (Client::new(TIMEOUT), FolderCache::new(cache));
            for Asked {
                options,
                packs,
                reply,
            } in received
            {
                let names: Vec<PackName> = packs.iter().map(|needed| needed.pack.clone()).collect();
                let reachable = online.load(Ordering::Relaxed).then_some(&client);
                let mut first = Some(packs);
                let resolved = resolve(options, reachable, &cache, |_, _| {
                    first.take().unwrap_or_default()
                });
                let _ = reply.send(Job::Fetched {
                    packs: names,
                    resolved,
                });
            }
        });
        Self { asked, network }
    }

    /// Asks for `packs`, resolved with `options`; what was got comes to `reply` as [`Job::Fetched`].
    pub(crate) fn fetch(&self, options: Options, packs: Vec<Needed>, reply: Sender<Job>) {
        let _ = self.asked.send(Asked {
            options,
            packs,
            reply,
        });
    }

    /// Lets the thread reach the registries, or only the cache.
    pub(crate) fn set_network(&self, network: bool) {
        self.network.store(network, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_is_compmds_packs_folder_in_the_users_cache() {
        assert!(user_cache().ends_with("compmd/packs"));
    }
}
