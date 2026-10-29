//! The fixed vocabulary against what is written about it elsewhere: `conformance/params.json`, which the renderer
//! checks itself against, the spec's tables and its copy of `box`'s signature, and `llms.txt`. Each fails, naming
//! what is missing, until a param or token added to the processor is added there too.

use compmd::values::{Known, vocabulary};
use std::fs;
use std::path::{Path, PathBuf};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(file: &str) -> String {
    fs::read_to_string(repository().join(file)).unwrap_or_else(|error| panic!("{file}: {error}"))
}

/// The words inside the backticks of `line`: `` `sticky=top|bottom` `` gives `sticky`, `top` and `bottom`.
fn quoted_words(line: &str) -> Vec<&str> {
    line.split('`')
        .skip(1)
        .step_by(2)
        .flat_map(|span| span.split(|c: char| c.is_whitespace() || "|=,".contains(c)))
        .filter(|word| !word.is_empty())
        .collect()
}

/// The lines of `text` that name `param` in backticks.
fn naming<'t>(text: &'t str, param: &str) -> Vec<&'t str> {
    text.lines()
        .filter(|line| quoted_words(line).contains(&param))
        .collect()
}

/// What is wrong with how `text` (the file `file`) writes `param`: it isn't named, or no line naming it lists every
/// token it takes.
fn missing(text: &str, file: &str, param: &Known) -> Option<String> {
    let lines = naming(text, &param.name);
    if lines.is_empty() {
        return Some(format!("{file} doesn't name `{}`", param.name));
    }
    let lists_all = |line: &&str| {
        let words = quoted_words(line);
        param
            .tokens
            .iter()
            .all(|token| words.contains(&token.as_str()))
    };
    (!lines.iter().any(lists_all)).then(|| {
        format!(
            "no line of {file} naming `{}` lists all it takes: {}",
            param.name,
            param.tokens.join(", ")
        )
    })
}

#[test]
fn params_json_is_up_to_date() {
    let path = repository().join("conformance/params.json");
    let written = serde_json::to_string_pretty(&vocabulary()).expect("JSON") + "\n";
    if std::env::var_os("COMPMD_WRITE").is_some() {
        fs::write(&path, &written).expect("written");
    }
    let saved = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        saved == written,
        "conformance/params.json isn't what the processor gives; write it again with `COMPMD_WRITE=1 cargo test --test vocabulary`"
    );
}

#[test]
fn the_spec_lists_every_common_param_with_its_tokens() {
    let spec = read("compmd-spec-v1.md");
    let section = spec
        .split("### 9.2 Common params")
        .nth(1)
        .and_then(|rest| rest.split("\n- ").next())
        .expect("section 9.2");
    let rows = section
        .lines()
        .filter(|line| line.starts_with('|'))
        .collect::<Vec<_>>()
        .join("\n");
    let problems: Vec<String> = vocabulary()
        .common
        .iter()
        .filter_map(|param| missing(&rows, "spec §9.2", param))
        .collect();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_specs_type_grammar_has_every_type_name() {
    let spec = read("compmd-spec-v1.md");
    let grammar = spec
        .split("type-decl = ")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .expect("the type grammar of §7.3");
    let missing: Vec<&str> = compmd::tree::TypeName::NAMES
        .iter()
        .copied()
        .filter(|name| !grammar.contains(&format!("\"{name}\"")))
        .collect();
    assert!(
        missing.is_empty(),
        "spec §7.3's type grammar doesn't name {missing:?}"
    );
}

#[test]
fn the_specs_copy_of_box_is_core_s() {
    let core = read("processor/src/core.comp.md");
    let fence = |text: &str| -> String {
        let start = text
            .find("```compmd component=box signature")
            .expect("box's signature fence");
        let end = text[start..].find("\n```\n").expect("its end");
        text[start..start + end].to_string()
    };
    assert_eq!(
        fence(&read("compmd-spec-v1.md")),
        fence(&core),
        "spec §9 quotes box's signature as `core.comp.md` has it"
    );
}

#[test]
fn llms_txt_lists_every_param_with_its_tokens() {
    let llms = read("llms.txt");
    let vocabulary = vocabulary();
    let problems: Vec<String> = vocabulary
        .common
        .iter()
        .chain(&vocabulary.layout)
        .chain(&vocabulary.attributes)
        .filter_map(|param| missing(&llms, "llms.txt", param))
        .collect();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn words_are_found_inside_backticks() {
    assert_eq!(
        quoted_words("a `sticky=top|bottom` b `x y`"),
        ["sticky", "top", "bottom", "x", "y"]
    );
    assert_eq!(naming("`a`\nb\n`c a`", "a"), ["`a`", "`c a`"]);
}
