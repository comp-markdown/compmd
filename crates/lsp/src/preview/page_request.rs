//! The document's request as the preview's page is given it (`request.json`; docs/editor-support.md, decision 8):
//! everything a processor needs but the text, which the events bring: its path, the files and the packs' files the
//! server's processor read, its policy, its ambient packs and the origins its data may come from.

use std::cell::RefCell;
use std::rc::Rc;

use compmd::pack::PackName;
use compmd::policy::Policy;
use serde_json::{Value, json};

use crate::processor_reads::Reads;

/// What the page's request is made of, as a document was last processed.
pub(crate) struct PageRequest {
    /// The document's path in its root.
    pub(crate) path: String,
    /// Whether it is opened as plain Markdown, a `.md` file.
    pub(crate) plain_markdown: bool,
    /// What its processor read, which grows as it reads more.
    pub(crate) reads: Rc<RefCell<Reads>>,
    pub(crate) policy: Policy,
    pub(crate) ambient: Vec<PackName>,
    /// The origins its data may come from: a list, or `"any"`.
    pub(crate) allowed_origins: Value,
}

impl PageRequest {
    /// The request, as JSON a processor's request reads.
    pub(crate) fn json(&self) -> Value {
        let reads = self.reads.borrow();
        json!({
            "path": self.path,
            "plainMarkdown": self.plain_markdown,
            "files": reads.files(),
            "packs": reads.packs(),
            "policy": self.policy,
            "ambient": self.ambient,
            "allowedOrigins": self.allowed_origins,
        })
    }
}

#[cfg(test)]
mod tests {
    use compmd::host::Files;

    use super::*;
    use crate::processor_reads::Recorded;

    #[test]
    fn the_request_has_what_was_read_with_the_policy() {
        let reads = Rc::new(RefCell::new(Reads::default()));
        let files = Recorded::new(
            std::collections::HashMap::from([("a.comp.md".to_owned(), "# A".to_owned())]),
            &reads,
        );
        let request = PageRequest {
            path: "a.comp.md".into(),
            plain_markdown: false,
            reads: Rc::clone(&reads),
            policy: Policy::default(),
            ambient: vec!["@kit/notes".parse().expect("a name")],
            allowed_origins: json!("any"),
        };
        assert_eq!(request.json()["files"], json!({}));
        files.file("a.comp.md");
        let json = request.json();
        assert_eq!(json["files"], json!({ "a.comp.md": "# A" }), "read since");
        assert_eq!(json["ambient"], json!(["@kit/notes"]));
        assert_eq!(json["plainMarkdown"], false);
        assert_eq!(json["allowedOrigins"], "any");
        assert_eq!(json["policy"]["imports"], "any");
    }
}
