use crate::features::code_actions::tests::{actions_of, applied, caret_at};

/// The document `text` with its signature written by the action at `line`, and the action's title; `None` when it
/// offers none.
fn signed(text: &str, line: u32) -> Option<(String, String)> {
    let [action] = actions_of("refactor.rewrite", text, caret_at(line, 0))
        .try_into()
        .ok()?;
    Some((action.title.clone(), applied(text, &action)))
}

const BADGE: &str = "```tsx component=badge\nexport default function Badge({ label, size = 3, tone, children }: { label: string; size: number; tone?: 'info' | 'warn'; children: React.ReactNode }) { return null; }\n```";

#[test]
fn a_definition_without_a_signature_gets_one_after_it() {
    let text = format!("# A\n\n{BADGE}\n\nText.\n");
    let (title, signed) = signed(&text, 3).expect("offered");
    assert_eq!(title, "Generate the signature of `badge`");
    assert_eq!(
        signed,
        format!(
            "# A\n\n{BADGE}\n\n```compmd component=badge signature\nparams:\n  label: string\n  size: number = 3\n  tone: info | warn?\nslots: [default]\n```\n\nText.\n"
        )
    );
}

#[test]
fn a_signature_is_brought_up_to_date_keeping_what_the_writer_wrote() {
    let text = format!(
        "{BADGE}\n\n```compmd component=badge signature\nparams:\n  # What it says.\n  label:\n    type: string\n    doc: The badge's text\n  gone: number\nslots: [default]\n```\n"
    );
    let (title, signed) = signed(&text, 1).expect("offered");
    assert_eq!(title, "Update the signature of `badge`");
    assert_eq!(
        signed,
        format!(
            "{BADGE}\n\n```compmd component=badge signature\nparams:\n  # What it says.\n  label:\n    type: string\n    doc: The badge's text\n  size: number = 3\n  tone: info | warn?\nslots: [default]\n```\n"
        )
    );
    // Brought up to date, it offers nothing more.
    assert_eq!(
        super::super::tests::actions_of("refactor.rewrite", &signed, caret_at(1, 0)),
        []
    );
}

#[test]
fn only_on_a_host_language_definition_it_can_read() {
    let text = format!("# A\n\n{BADGE}\n");
    assert_eq!(signed(&text, 0), None, "away from it");
    let plain = "```js component=chart\nexport default function mount() {}\n```\n";
    assert_eq!(signed(plain, 1), None, "plain JavaScript takes no props");
    let unread = "```jsx component=chart\nexport default memo(Chart);\n```\n";
    assert_eq!(signed(unread, 1), None, "props it can't read");
    let compmd = "```compmd component=note\n---\nA note.\n```\n";
    assert_eq!(signed(compmd, 1), None, "a compmd definition");
}
