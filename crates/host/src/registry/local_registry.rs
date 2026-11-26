//! A registry on the machine, for tests ([the language server]'s testing): it serves the packs it is given over npm's
//! protocol, their abbreviated metadata and their tarballs, from `http://127.0.0.1`, and records what it was asked.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use miniz_oxide::deflate::compress_to_vec;
use serde_json::{Map, Value, json};

use super::integrity::integrity_of;

/// A tar block.
const BLOCK: usize = 512;

/// A pack the registry serves: its name, each version with its files by path in the package.
#[derive(Clone, Debug, Default)]
pub struct ServedPack {
    name: String,
    versions: BTreeMap<String, BTreeMap<String, Vec<u8>>>,
}

impl ServedPack {
    /// The pack `name`, `@kit/mermaid`, with no versions yet.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            versions: BTreeMap::new(),
        }
    }

    /// The pack with `version` too, `1.2.0`, of `files`, each a path in the package and its text.
    #[must_use]
    pub fn version(mut self, version: &str, files: &[(&str, &str)]) -> Self {
        let files = files
            .iter()
            .map(|(path, text)| ((*path).to_owned(), text.as_bytes().to_vec()))
            .collect();
        self.versions.insert(version.to_owned(), files);
        self
    }
}

/// What the registry answers a path with.
type Routes = BTreeMap<String, (&'static str, Vec<u8>)>;

/// A registry serving packs on the machine until it is dropped.
///
/// ```
/// use compmd_host::registry::{LocalRegistry, ServedPack};
///
/// let mermaid = ServedPack::new("@kit/mermaid").version("1.2.0", &[("package.json", "{}")]);
/// let registry = LocalRegistry::serve(&[mermaid])?;
/// assert!(registry.url().starts_with("http://127.0.0.1:"));
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Debug)]
pub struct LocalRegistry {
    url: String,
    asked: Arc<Mutex<Vec<String>>>,
    stopped: Arc<AtomicBool>,
}

/// `files` as a pack's tarball: a tar of them under `package/`, gzipped, as npm publishes one.
#[must_use]
pub fn tarball(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut archive = Vec::new();
    for (path, bytes) in files {
        let mut header = vec![0u8; BLOCK];
        let name = format!("package/{path}");
        header[..name.len()].copy_from_slice(name.as_bytes());
        header[124..135].copy_from_slice(format!("{:011o}", bytes.len()).as_bytes());
        header[156] = b'0';
        archive.extend(header);
        archive.extend(bytes);
        archive.resize(archive.len().div_ceil(BLOCK) * BLOCK, 0);
    }
    archive.extend([0u8; BLOCK * 2]);
    let mut gzipped = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
    gzipped.extend(compress_to_vec(&archive, 6));
    gzipped.extend([0; 8]);
    gzipped
}

/// The routes of `packs` at `url`: each pack's metadata at its escaped name, each version's tarball under it.
fn routes_of(packs: &[ServedPack], url: &str) -> Routes {
    let mut routes = Routes::new();
    for pack in packs {
        let file_name = pack.name.rsplit('/').next().unwrap_or_default();
        let mut versions = Map::new();
        for (version, files) in &pack.versions {
            let path = format!("{}/-/{file_name}-{version}.tgz", pack.name);
            let bytes = tarball(files);
            let size: usize = files.values().map(Vec::len).sum();
            let dist = json!({ "tarball": format!("{url}{path}"), "integrity": integrity_of(&bytes), "unpackedSize": size });
            versions.insert(version.clone(), json!({ "dist": dist }));
            routes.insert(format!("/{path}"), ("application/octet-stream", bytes));
        }
        let metadata = json!({ "name": pack.name, "versions": Value::Object(versions) });
        let escaped = pack.name.replacen('/', "%2f", 1);
        routes.insert(
            format!("/{escaped}"),
            (
                "application/vnd.npm.install-v1+json",
                metadata.to_string().into_bytes(),
            ),
        );
    }
    routes
}

/// Answers one request on `stream` from `routes`, recording its path in `asked`.
fn answer(stream: TcpStream, routes: &Routes, asked: &Mutex<Vec<String>>) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let path = line.split(' ').nth(1).unwrap_or_default().to_owned();
    let mut header = String::new();
    while reader.read_line(&mut header)? > 2 {
        header.clear();
    }
    asked
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(path.clone());
    let (status, (media_type, body)) = routes
        .get(&path)
        .map_or(("404 Not Found", ("text/plain", Vec::new())), |route| {
            ("200 OK", route.clone())
        });
    let mut stream = reader.into_inner();
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {media_type}\r\nContent-Length: {}\r\nCache-Control: public, max-age=300\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(&body)
}

impl LocalRegistry {
    /// A registry serving `packs` on a port of its own.
    ///
    /// # Errors
    ///
    /// When no port can be had.
    pub fn serve(packs: &[ServedPack]) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}/", listener.local_addr()?);
        let routes = routes_of(packs, &url);
        let asked = Arc::new(Mutex::new(Vec::new()));
        let stopped = Arc::new(AtomicBool::new(false));
        let (recorded, stopping) = (Arc::clone(&asked), Arc::clone(&stopped));
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                if stopping.load(Ordering::Relaxed) {
                    break;
                }
                drop(answer(stream, &routes, &recorded));
            }
        });
        Ok(Self {
            url,
            asked,
            stopped,
        })
    }

    /// Its URL, ending in `/`, as a configuration's `registries` names one.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// The paths it was asked for, in order.
    #[must_use]
    pub fn asked(&self) -> Vec<String> {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Stops serving: the port is closed, so what asks it next can't connect.
impl Drop for LocalRegistry {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        drop(TcpStream::connect(
            self.url.trim_start_matches("http://").trim_end_matches('/'),
        ));
    }
}
