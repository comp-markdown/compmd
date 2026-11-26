//! The preview's server ([editor support], decision 8; [the language server], decision 13): a document's page in the
//! browser, a host of its own. It listens on `127.0.0.1` alone, on two ports chosen when it starts: the page's, under a
//! random token, with the routes the page reads beside it (`events`, `request.json`, `registry/`); and the frames', so
//! they have an origin of their own, served by `compmd-host`'s routes and headers. A request whose `Host` isn't the
//! port's own is refused, so a page elsewhere can't reach it by naming `127.0.0.1` otherwise.
//!
//! [editor support]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-support.md
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

mod events;
mod files;
mod page_request;
mod page_route;
mod project_store;
#[cfg(test)]
pub(crate) mod testing;
mod token;

use std::io::{self, Cursor};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use compmd_host::registry::Proxy;
use compmd_host::{Store, content_type, respond};
use serde_json::Value;
use tiny_http::{Header, Request, Response, Server};

pub(crate) use files::PreviewFiles;
pub(crate) use page_request::PageRequest;
pub(crate) use project_store::ProjectStore;

use events::Events;
use page_route::{PageRoute, page_route, with_origin};
use token::token;

/// What the page's requests and the frames' are answered from.
struct Served {
    token: String,
    files: PreviewFiles,
    /// The packs the bundle doesn't hold, which the frames load: the project's.
    store: Box<dyn Store>,
    /// The proxy of the registries the document's configuration names; `None` when it names none.
    proxy: Option<Proxy>,
    events: Events,
    /// The document's request but its text, as JSON; `None` until it is given.
    request: Mutex<Option<String>>,
    page_port: u16,
    frames_port: u16,
}

/// A document's preview, served until it is dropped.
pub(crate) struct Preview {
    served: Arc<Served>,
    page: Arc<Server>,
    frames: Arc<Server>,
}

/// A server on `127.0.0.1`, on a port the system chooses, and its port.
fn listening() -> io::Result<(Server, u16)> {
    let server = Server::http("127.0.0.1:0").map_err(io::Error::other)?;
    let port = server
        .server_addr()
        .to_ip()
        .map(|address| address.port())
        .ok_or_else(|| io::Error::other("the preview's server has no port"))?;
    Ok((server, port))
}

/// Whether `request` names the server on `port` as its `Host`, as a browser that opened it at `127.0.0.1` does.
fn is_own_host(request: &Request, port: u16) -> bool {
    let own = format!("127.0.0.1:{port}");
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Host"))
        .is_some_and(|header| header.value.as_str() == own)
}

/// A header, from a name and a value both valid in one.
fn header(name: &str, value: &str) -> Option<Header> {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).ok()
}

/// A plain-text answer with `status`, saying `why`.
fn plain(status: u16, why: &str) -> Response<Cursor<Vec<u8>>> {
    let response = Response::from_data(why.as_bytes().to_vec()).with_status_code(status);
    match header("Content-Type", "text/plain; charset=utf-8") {
        Some(header) => response.with_header(header),
        None => response,
    }
}

/// `body` with its `media_type`.
fn typed(body: Vec<u8>, media_type: &str) -> Response<Cursor<Vec<u8>>> {
    let response = Response::from_data(body);
    match header("Content-Type", media_type) {
        Some(header) => response.with_header(header),
        None => response,
    }
}

/// `response`, `compmd-host`'s, as the server sends it.
fn sent(response: http::Response<Vec<u8>>) -> Response<Cursor<Vec<u8>>> {
    let status = response.status().as_u16();
    let headers: Vec<Header> = response
        .headers()
        .iter()
        .filter_map(|(name, value)| header(name.as_str(), value.to_str().ok()?))
        .collect();
    headers.into_iter().fold(
        Response::from_data(response.into_body()).with_status_code(status),
        Response::with_header,
    )
}

impl Served {
    /// Answers `request` on the page's port.
    fn page(self: &Arc<Self>, request: Request) {
        if !is_own_host(&request, self.page_port) {
            return drop(request.respond(plain(403, "this isn't the preview's host")));
        }
        if request.method().as_str() != "GET" {
            return drop(request.respond(plain(405, "the preview answers GET only")));
        }
        let response = match page_route(request.url(), &self.token) {
            PageRoute::File(path) => self.page_file(path),
            PageRoute::Events => return self.events.listen(request.into_writer()),
            PageRoute::Request => self.request(),
            PageRoute::Registry(path) => {
                let (served, path) = (Arc::clone(self), path.to_owned());
                // A registry can be slow: the page's other requests aren't held up by it.
                thread::spawn(move || {
                    let answer = served.registry(&path);
                    drop(request.respond(answer));
                });
                return;
            }
            PageRoute::Unknown => plain(404, "not the preview's"),
        };
        drop(request.respond(response));
    }

    /// The page's file at `path`: the page itself with the frames' origin.
    fn page_file(&self, path: &str) -> Response<Cursor<Vec<u8>>> {
        let Some(bytes) = self.files.page_file(path) else {
            return plain(404, "no such file in the preview");
        };
        if path == "index.html" {
            let origin = format!("http://127.0.0.1:{}", self.frames_port);
            let page = with_origin(&String::from_utf8_lossy(bytes), &origin);
            return typed(page.into_bytes(), content_type(path));
        }
        typed(bytes.to_vec(), content_type(path))
    }

    /// The document's request but its text, once it is given.
    fn request(&self) -> Response<Cursor<Vec<u8>>> {
        let request = self.request.lock().unwrap_or_else(PoisonError::into_inner);
        match request.as_deref() {
            Some(json) => typed(json.as_bytes().to_vec(), "application/json"),
            None => plain(503, "the document isn't processed yet"),
        }
    }

    /// The registries' proxy's answer to `path`; a 404 when the configuration names no registry.
    fn registry(&self, path: &str) -> Response<Cursor<Vec<u8>>> {
        let Some(proxy) = &self.proxy else {
            return plain(404, "the document's configuration names no registry");
        };
        let base = format!(
            "http://127.0.0.1:{}/{}/registry/",
            self.page_port, self.token
        );
        sent(proxy.answer(path, &base))
    }

    /// Answers `request` on the frames' port, by the serving contract.
    fn frames(&self, request: Request) {
        if !is_own_host(&request, self.frames_port) {
            return drop(request.respond(plain(403, "this isn't the preview's host")));
        }
        let asked = http::Request::builder()
            .method(request.method().as_str())
            .uri(request.url())
            .body(());
        let response = match asked {
            Ok(asked) => sent(respond(&self.files.bundle, self.store.as_ref(), &asked)),
            Err(_) => plain(400, "not a request the preview reads"),
        };
        drop(request.respond(response));
    }
}

impl Preview {
    /// Starts the preview of `files`, the frames loading the packs the bundle doesn't hold from `store`, and the page's
    /// `registry/` passing on to `proxy`'s registries.
    ///
    /// # Errors
    ///
    /// When no port can be had, or the system gives no randomness for the token.
    pub(crate) fn start(
        files: PreviewFiles,
        store: Box<dyn Store>,
        proxy: Option<Proxy>,
    ) -> io::Result<Self> {
        let (page, page_port) = listening()?;
        let (frames, frames_port) = listening()?;
        let served = Arc::new(Served {
            token: token().map_err(io::Error::other)?,
            files,
            store,
            proxy,
            events: Events::default(),
            request: Mutex::new(None),
            page_port,
            frames_port,
        });
        let (page, frames) = (Arc::new(page), Arc::new(frames));
        let (on_page, page_served) = (Arc::clone(&page), Arc::clone(&served));
        thread::spawn(move || {
            for request in on_page.incoming_requests() {
                page_served.page(request);
            }
        });
        let (on_frames, frames_served) = (Arc::clone(&frames), Arc::clone(&served));
        thread::spawn(move || {
            for request in on_frames.incoming_requests() {
                frames_served.frames(request);
            }
        });
        Ok(Self {
            served,
            page,
            frames,
        })
    }

    /// The page's address, with its token, which the browser opens.
    pub(crate) fn url(&self) -> String {
        format!(
            "http://127.0.0.1:{}/{}/",
            self.served.page_port, self.served.token
        )
    }

    /// The document changed: its `text`, and the `line` being edited, from 1.
    pub(crate) fn change(&self, text: &str, line: usize) {
        self.served.events.change(text, line);
    }

    /// The document's request but its text: the page fetches it when it opens, and again, told by a `reload`, when it
    /// changes.
    pub(crate) fn set_request(&self, request: &Value) {
        let json = request.to_string();
        let mut held = self
            .served
            .request
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let changed = held.as_ref().is_some_and(|before| *before != json);
        *held = Some(json);
        drop(held);
        if changed {
            self.served.events.reload();
        }
    }
}

/// Stops serving: both ports close.
impl Drop for Preview {
    fn drop(&mut self) {
        self.page.unblock();
        self.frames.unblock();
    }
}

#[cfg(test)]
mod tests;
