//! The preview's server-sent events (docs/editor-support.md, decision 8): `change`, the document's text and the line
//! being edited, and `reload`, when what it reads changed. Every page listening gets each one; a page that starts
//! listening gets the last change first, so it shows the document as it is.

use std::io::Write;
use std::sync::{Mutex, PoisonError};

use serde_json::json;

/// What starts an event stream on a connection.
const HEAD: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";

/// A page's connection the events are written to.
type Stream = Box<dyn Write + Send>;

/// The pages listening, and the last change.
#[derive(Default)]
struct Listening {
    streams: Vec<Stream>,
    last_change: Option<String>,
}

/// The preview's events, to the pages listening.
#[derive(Default)]
pub(crate) struct Events {
    listening: Mutex<Listening>,
}

/// The event `name` with `data`, one line of JSON, as a stream writes it.
fn event(name: &str, data: &serde_json::Value) -> String {
    format!("event: {name}\ndata: {data}\n\n")
}

/// Writes `text` to `stream` at once; gives whether it could.
fn written(stream: &mut Stream, text: &str) -> bool {
    stream
        .write_all(text.as_bytes())
        .and_then(|()| stream.flush())
        .is_ok()
}

impl Events {
    /// A page starts listening on `stream`, a connection whose request asked for the events: it gets the last change
    /// first.
    pub(crate) fn listen(&self, mut stream: Stream) {
        let mut listening = self.lock();
        let opened = written(&mut stream, HEAD)
            && listening
                .last_change
                .as_deref()
                .is_none_or(|change| written(&mut stream, change));
        if opened {
            listening.streams.push(stream);
        }
    }

    /// The document changed: its `text`, and the `line` being edited, from 1.
    pub(crate) fn change(&self, text: &str, line: usize) {
        let change = event("change", &json!({ "text": text, "line": line }));
        let mut listening = self.lock();
        listening.last_change = Some(change.clone());
        Self::send(&mut listening, &change);
    }

    /// What the document reads changed.
    pub(crate) fn reload(&self) {
        Self::send(&mut self.lock(), &event("reload", &json!({})));
    }

    /// How many pages listen.
    #[cfg(test)]
    pub(crate) fn listening(&self) -> usize {
        self.lock().streams.len()
    }

    /// Sends `text` to each page listening; a page gone stops listening.
    fn send(listening: &mut Listening, text: &str) {
        listening.streams.retain_mut(|stream| written(stream, text));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Listening> {
        self.listening
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;

    /// A stream that keeps what is written, until it is gone: then every write fails.
    #[derive(Clone, Default)]
    struct Kept(Arc<Mutex<Vec<u8>>>, Arc<AtomicBool>);

    impl Write for Kept {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.1.load(Ordering::Relaxed) {
                return Err(std::io::Error::other("gone"));
            }
            self.0.lock().expect("kept").extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Kept {
        fn gone() -> Self {
            Self(Arc::default(), Arc::new(AtomicBool::new(true)))
        }

        fn text(&self) -> String {
            String::from_utf8(self.0.lock().expect("kept").clone()).expect("UTF-8")
        }
    }

    #[test]
    fn each_page_gets_each_event_and_a_new_one_the_last_change_first() {
        let events = Events::default();
        let first = Kept::default();
        events.listen(Box::new(first.clone()));
        events.change("# A", 1);
        events.change("# AB", 1);
        let late = Kept::default();
        events.listen(Box::new(late.clone()));
        events.reload();
        let change = "event: change\ndata: {\"text\":\"# AB\",\"line\":1}\n\n";
        assert!(first.text().starts_with(HEAD));
        assert!(
            first
                .text()
                .ends_with(&format!("{change}event: reload\ndata: {{}}\n\n")),
            "{}",
            first.text()
        );
        assert_eq!(
            late.text(),
            format!("{HEAD}{change}event: reload\ndata: {{}}\n\n")
        );
    }

    #[test]
    fn a_page_gone_stops_listening() {
        let events = Events::default();
        events.listen(Box::new(Kept::default()));
        events.listen(Box::new(Kept::gone()));
        assert_eq!(
            events.listening(),
            1,
            "a stream that can't be written never listens"
        );
        let going = Kept::default();
        events.listen(Box::new(going.clone()));
        assert_eq!(events.listening(), 2);
        going.1.store(true, Ordering::Relaxed);
        events.reload();
        assert_eq!(events.listening(), 1, "gone at the next event");
    }
}
