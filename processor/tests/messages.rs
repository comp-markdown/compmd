//! What the diagnostics that refuse a value or a name say: F009 says what the param takes, of a built-in and of a
//! document's own component, and F002, F006, F008 and F009 give the fix their "did you mean" suggests.

use compmd::diagnostic::{Code, Diagnostic, Location};
use compmd::host::LoadedRelease;
use compmd::{Ambient, ParseOptions, Processor};
use std::collections::HashMap;

/// The diagnostics of validating `text`.
fn validated(text: &str) -> Vec<Diagnostic> {
    let ambient = Ambient {
        packs: Vec::new(),
        report: false,
    };
    Processor::new("input.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(
            HashMap::<String, String>::new(),
            HashMap::<String, Vec<LoadedRelease>>::new(),
            &ambient,
        )
        .validate(&[])
        .diagnostics()
        .to_vec()
}

/// The only diagnostic of `text` with `code`.
fn one(text: &str, code: Code) -> Diagnostic {
    let found: Vec<Diagnostic> = validated(text)
        .into_iter()
        .filter(|d| d.code == code)
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    found.into_iter().next().expect("one")
}

/// The fix of `diagnostic`, as `(start, end, text)`.
fn fix(diagnostic: &Diagnostic) -> Option<(Location, Location, String)> {
    let fix = diagnostic.fix.as_ref()?;
    Some((fix.range.start, fix.range.end?, fix.text.clone()))
}

#[test]
fn a_common_param_says_what_it_takes() {
    let refused = one("::: box {padding=huge}\nx\n::: /box\n", Code::F009);
    assert_eq!(
        refused.message,
        "`padding` takes one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l`, so the param is treated as absent"
    );
    assert_eq!(refused.range.start, Location::new(1, 18));
    assert_eq!(fix(&refused), None, "nothing is close to `huge`");
    let background = one("::: box {background=reddish}\nx\n::: /box\n", Code::F009);
    assert!(background.message.starts_with(
        "`background` takes `none`, `subtle`, `strong`, `accent` or a CSS color, such as `#f6f8fa`"
    ));
}

#[test]
fn a_misspelt_token_gets_a_fix() {
    let refused = one("::: box {border=subtl}\nx\n::: /box\n", Code::F009);
    assert_eq!(
        refused.message,
        "`border` takes `none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?"
    );
    assert_eq!(
        fix(&refused),
        Some((Location::new(1, 17), Location::new(1, 22), "subtle".into()))
    );
    // A quoted value is replaced with its quotes.
    let quoted = one(
        "::: box {md:border-radius=\"ful\"}\nx\n::: /box\n",
        Code::F009,
    );
    assert_eq!(
        fix(&quoted),
        Some((Location::new(1, 27), Location::new(1, 32), "full".into()))
    );
}

#[test]
fn box_params_and_their_enums_say_what_they_take() {
    let display = one("::: box {display=inline}\nx\n::: /box\n", Code::F009);
    assert!(
        display
            .message
            .starts_with("`display` takes `block`, `flex`, `grid` or `contents`")
    );
    let areas = one(
        "::: box {display=grid grid-template-areas=\"a b\"}\nx\n::: /box\n",
        Code::F009,
    );
    assert!(
        areas
            .message
            .starts_with("`grid-template-areas` takes quoted rows of area names, all as long, such as `'head head' 'side main'`"),
        "{}",
        areas.message
    );
}

#[test]
fn a_documents_own_component_says_what_its_params_take() {
    let text = "```compmd component=badge\nparams:\n  tone: info | warning | danger = info\n  gap: space = s\n---\nB\n```\n\n:: badge[New] {tone=warnign gap=wide}\n";
    let found: Vec<Diagnostic> = validated(text)
        .into_iter()
        .filter(|d| d.code == Code::F009)
        .collect();
    let messages: Vec<&str> = found.iter().map(|d| d.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "`tone` takes `info`, `warning` or `danger`, so the param is treated as absent; did you mean `warning`?",
            "`gap` takes `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `1rem`, so the param is treated as absent",
        ]
    );
    assert_eq!(
        fix(&found[0]).map(|(_, _, text)| text),
        Some("warning".into())
    );
}

#[test]
fn unknown_names_get_fixes() {
    let param = one("::: box {md:borde=subtle}\nx\n::: /box\n", Code::F008);
    assert_eq!(
        fix(&param),
        Some((Location::new(1, 13), Location::new(1, 18), "border".into()))
    );
    let component = one(":: zzz[A]\n", Code::F002);
    assert_eq!(fix(&component), None, "`zzz` is too far from any name");
    let component = one("::: sstack\nx\n::: /sstack\n", Code::F002);
    assert_eq!(
        fix(&component),
        Some((Location::new(1, 5), Location::new(1, 11), "stack".into()))
    );
    let text = "```compmd component=card\nslots: [default, footer]\n---\n{{md.slot}}\n```\n\n::: card\nBody.\n\n:: #fotter\nF.\n::: /card\n";
    let slot = one(text, Code::F006);
    assert_eq!(
        fix(&slot),
        Some((Location::new(10, 5), Location::new(10, 11), "footer".into()))
    );
}
