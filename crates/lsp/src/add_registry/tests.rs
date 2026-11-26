use std::path::PathBuf;

use super::*;

const NPM: &str = "https://registry.npmjs.org/";

/// A folder for one test's project, with `files`.
fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "compmd-lsp-add-registry-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a folder");
    for (path, text) in files {
        fs::write(dir.join(path), text).expect("written");
    }
    dir
}

/// The text of `file` in `root` once `add`'s edit is made to it.
fn edited(add: &AddRegistry, root: &Path, file: &str) -> String {
    let path = root.join(file);
    let text = fs::read_to_string(&path).expect("the file");
    let uri = uri_of(&path).expect("a URI");
    let (_, edits) = add
        .edit
        .changes
        .iter()
        .flatten()
        .find(|(changed, _)| **changed == uri)
        .expect("the file's edits");
    let mut edits = edits.clone();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    let mut chars: Vec<char> = text.chars().collect();
    for edit in edits {
        let at = Encoding::Utf16.offset(&text, edit.range.start);
        chars.splice(at..at, edit.new_text.chars());
    }
    chars.into_iter().collect()
}

#[test]
fn a_configuration_without_registries_gets_them_and_direct_without_a_proxy() {
    let root = project(
        "direct",
        &[("compmd.config.json", "{\n  \"imports\": \"any\"\n}\n")],
    );
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!((add.file, add.fetch), (CONFIG_FILE, Fetch::Direct));
    assert_eq!(
        edited(&add, &root, CONFIG_FILE),
        "{\n  \"registries\": { \"@comp-markdown\": \"https://registry.npmjs.org/\" },\n  \"fetch\": \"direct\",\n  \"imports\": \"any\"\n}\n"
    );
    assert!(add.question().contains("compmd.config.json names it"));
    assert!(add.question().contains("which sees their addresses"));
    assert_eq!(
        add.title(),
        "Add npm for `@comp-markdown` to compmd.config.json"
    );
}

#[test]
fn registries_already_named_get_one_more_and_their_host_fetches_it() {
    let config = "{ \"registries\": { \"@kit\": \"https://npm.kit.example/\" } }";
    let root = project("named", &[("compmd.config.json", config)]);
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!(add.fetch, Fetch::Host);
    assert_eq!(
        edited(&add, &root, CONFIG_FILE),
        "{ \"registries\": { \"@comp-markdown\": \"https://registry.npmjs.org/\", \"@kit\": \"https://npm.kit.example/\" } }"
    );
    assert!(
        add.question()
            .contains("your site's host then fetches them")
    );
}

#[test]
fn a_configuration_in_package_json_is_edited_there_and_vite_proxies() {
    let manifest = "{\n  \"name\": \"site\",\n  \"devDependencies\": { \"@compmd/vite\": \"^0.11.0\" },\n  \"compmd\": {}\n}\n";
    let root = project("manifest", &[("package.json", manifest)]);
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!((add.file, add.fetch), ("package.json", Fetch::Host));
    assert!(
        edited(&add, &root, "package.json").contains(
            "\"compmd\": {\n  \"registries\": { \"@comp-markdown\": \"https://registry.npmjs.org/\" }\n}"
        )
    );
}

#[test]
fn what_the_configuration_says_of_fetch_stays() {
    let root = project(
        "stated",
        &[("compmd.config.json", "{\"fetch\": \"direct\"}")],
    );
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!(add.fetch, Fetch::Direct);
    assert_eq!(
        edited(&add, &root, CONFIG_FILE),
        "{\"registries\": { \"@comp-markdown\": \"https://registry.npmjs.org/\" },\"fetch\": \"direct\"}"
    );
    let root = project("host", &[("compmd.config.json", "{\"fetch\": \"host\"}")]);
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!(add.fetch, Fetch::Host);
}

#[test]
fn without_a_configuration_one_is_created() {
    let manifest = r#"{"dependencies": {"@compmd/vite": "^0.11.0"}}"#;
    let root = project("created", &[("package.json", manifest)]);
    let add = AddRegistry::of(&root, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!((add.file, add.fetch), (CONFIG_FILE, Fetch::Host));
    let Some(DocumentChanges::Operations(operations)) = &add.edit.document_changes else {
        panic!("operations");
    };
    let [
        DocumentChangeOperation::Op(ResourceOp::Create(created)),
        DocumentChangeOperation::Edit(edit),
    ] = operations.as_slice()
    else {
        panic!("a file created and written");
    };
    assert_eq!(created.uri, uri_of(&root.join(CONFIG_FILE)).expect("a URI"));
    let [OneOf::Left(written)] = edit.edits.as_slice() else {
        panic!("one edit");
    };
    assert_eq!(
        written.new_text,
        "{\n  \"registries\": { \"@comp-markdown\": \"https://registry.npmjs.org/\" }\n}\n"
    );
    let bare = project("bare", &[]);
    let add = AddRegistry::of(&bare, NPM, Encoding::Utf16).expect("an edit");
    assert_eq!(add.fetch, Fetch::Direct, "no proxy");
}

#[test]
fn a_configuration_that_isnt_an_object_has_no_edit() {
    let root = project("broken", &[("compmd.config.json", "[]")]);
    assert!(AddRegistry::of(&root, NPM, Encoding::Utf16).is_none());
}
