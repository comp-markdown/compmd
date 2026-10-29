# Incremental processing

**Status: done** (every stage, and the WebAssembly binding). How the processor handles an edit without processing the whole document again.

Today every change runs stages 1 to 4b on the whole text. The WebAssembly build takes about 3 ms for the spec's complete example and 10 ms for a 52 KB body (p95, an Apple M4), and the time grows with the document: a body ten times as long would take about ten times as long. An editor that processes on every key press needs the time to depend on the edit, not on the document.

## Decisions

1. **The unit is a run of top-level blocks** of the processed document (a paragraph, a heading, a whole list, a whole block quote, a whole `:::` container, a fence), ended by a blank line where no container is open. A block nested in a list or a container can't be parsed on its own, because Markdown's block structure depends on what is around it; a run like this can. Blocks with no blank line between them stay in one unit, since they affect each other: a lazy line or a `===` line joins the paragraph above, and W108 reports a component line right after a paragraph line. A continuation line (`::: + params`) stays with the component line before it, even after a blank line.
2. **An edit is classified, and its class decides what is redone** (see [Edits](#edits)). An edit inside one body block redoes that block, through every stage.
3. **The result equals processing from scratch:** the same parse tree, expanded tree and diagnostics. A differential test checks this over random edits.
4. **One more method on the expanded processor**, next to `reload`, that takes only the edit:

   ```rust
   let processor = processor.edit(&TextEdit { start, end, text });
   ```

   It gives the expanded processor back, whatever the edit touched, so a host never has to tell edits apart. The processor keeps the context, `fetch` and `data` its last load used, as it already keeps the host's `files` and `packs` from stage 2, and an edit to the frontmatter loads again with them. It owns them, so a host can keep the processor in a struct of its own (the WebAssembly binding does); a host that needs them too passes an `Rc` and keeps a clone.

   A host with data it fetches ahead (remote or `host:` sources) does one more thing: an edit to the frontmatter may name a source it hasn't fetched yet. That source loads as missing (F026 and its `fallback`), as on any first render without it, and `sources()` lists it. The host fetches it and calls `reload`, as it does today when the context changes.

   A host that renders a document with its editor:

   ```rust
   // Open: as today, then keep the processor.
   let mut processor = Processor::new(path, text)
       .parse(&parse_options)
       .resolve(files, packs, &ambient)
       .validate(&allowed_origins)
       .load(context.clone(), fetched, Rc::clone(&data))
       .expand(&expand_options);
   render(processor.expanded());

   // Each change in the editor: one call, one result.
   processor = processor.edit(&TextEdit { start: 11, end: 11, text: "!".into() });
   render(processor.expanded());

   // Only a host that fetches ahead, after an edit, or when it likes.
   // `sources` uses the context the processor keeps.
   let needed = processor.sources();
   if not_fetched_yet(&needed) {
       let fetched = fetch(&needed).await;
       processor = processor.reload(context.clone(), fetched, Rc::clone(&data));
       render(processor.expanded());
   }
   ```

   A host whose `fetch` and `data` answer every source (a local file host, a test) never needs the last part.

   The `Processor` stays the only way in.
5. **The markdown-rs fork gets a parse option for the labels defined elsewhere** in the document, so a slice parses as it would in place (see [Labels](#labels)).
6. **The host decides where edits come from:** a text area's selection, a CodeMirror transaction, a CRDT's text delta. The processor takes text edits and nothing else.

## Edits

```rust
/// The old text's `[start, end)` became `text`. Offsets count code points, as the trees' positions do (spec §10.5).
pub struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}
```

The host knows nothing of blocks. It says only which range of the text changed and what replaced it; finding the blocks that range touches is the processor's job.

`start` and `end` are positions in the whole text the processor last saw. They are the same positions the trees' `offset` fields hold (spec §10.5), and [Positions](#positions) says exactly how they are counted.

For example, with the text `Hello world` (positions 0 to 11):

| The user | The host sends | The text becomes |
|---|---|---|
| types `!` at the end | `{ start: 11, end: 11, text: "!" }` | `Hello world!` |
| presses Backspace after `world` | `{ start: 10, end: 11, text: "" }` | `Hello worl` |
| selects `world` and pastes `there` | `{ start: 6, end: 11, text: "there" }` | `Hello there` |
| selects `world` and deletes it | `{ start: 6, end: 11, text: "" }` | `Hello ` |
| pastes two paragraphs at the start | `{ start: 0, end: 0, text: "One.\n\nTwo.\n\n" }` | `One.`, `Two.`, then `Hello world` |

Typing sends one edit per key press, each over the text the one before left: typing `abc` at position 5 is `{5, 5, "a"}`, then `{6, 6, "b"}`, then `{7, 7, "c"}`.

A host with several changes at once (a CodeMirror transaction with several cursors, a collaborator's delta, key presses made while the last edit was processed) sends them one after another, each with positions in the text the edits before it left. It may also merge them into one edit that covers them all.

### Positions

An editor and the processor can count the same text differently. A position is only right when both agree on two things: which text is counted, and what one step counts.

**One step is one code point**, a Unicode character as the standard numbers them. That is neither what the user sees nor what most editors count:

| Text | The user sees | Code points (the processor) | UTF-16 units (JavaScript, a text area, CodeMirror) | UTF-8 bytes (a Rust `&str` index) |
|---|---|---|---|---|
| `a` | 1 | 1 | 1 | 1 |
| `é`, written as one character (U+00E9) | 1 | 1 | 1 | 2 |
| `é`, written as `e` and a combining accent (U+0065 U+0301) | 1 | 2 | 2 | 3 |
| `😀` (U+1F600) | 1 | 1 | 2 | 4 |
| `👍🏽`, a thumb and a skin tone (U+1F44D U+1F3FD) | 1 | 2 | 4 | 8 |
| a tab | several columns | 1 | 1 | 1 |
| a line ending written as CRLF | none | 2 | 2, or 1 where the editor turns it into LF | 2 |

So with the text `😀 Hi`, typing `!` at the end is at position 4 for the processor, but CodeMirror's `from` and a text area's `selectionStart` say 5, and a Rust byte index says 7. The WebAssembly binding turns UTF-16 positions into code points, so a JavaScript host passes the editor's positions as they are (see [The WebAssembly binding](#the-webassembly-binding)). A Rust host that holds byte indexes turns them into code points (`text[..index].chars().count()`).

**What is counted is the text the host gave the processor**, from its first character, which is position 0, with one exception: a byte order mark at the very start is dropped before counting, as the spec ignores it. Everything else counts, the frontmatter included. Editors change text in ways the user doesn't see:

- A text area turns every line ending into LF in its `value`, and CodeMirror does by default. A file on disk written with CRLF then has one step more per line than the editor's text.
- An editor may drop a byte order mark when it opens a file, or keep it and count it.
- An editor that folds a section or shows a widget in place of some text still counts the hidden text in its positions. Positions on screen, a line and column the user sees, are not positions in the text.

**The rule for a host:** give the processor the editor's text, not the file it was read from (`textarea.value`, CodeMirror's `state.doc.toString()`), and send edits from the editor's own text positions. Then both count the same text, whatever the editor turned into what. Saving the file is the host's business and changes nothing for the processor.

| The edit touches | What is redone |
|---|---|
| The frontmatter, or creates or removes one | Everything: imports, inputs, data and settings may all change |
| A link reference or footnote definition, or creates or removes one | Stage 1 for the whole body, since whether `[x][ref]` is a link depends on which labels are defined; stages 2 to 4b for the blocks whose tree changed |
| A `compmd` definition fence of the processed document | Stage 1 for that block; stages 2 to 4b for the processed document, since its names, signatures and recursion (F019) may change. Imported documents are kept |
| Any other block of the body | That block, through every stage |
| Nothing in the text: inputs or the render time changed | Stages 4a and 4b, as `reload` does today |

An edit is classified by the blocks it overlaps before and after it is parsed: typing `[a]: /x` at the start of a paragraph turns it into a definition, so the class is known only after stage 1 has parsed the changed blocks.

## Stage 1: parsing

- **What is kept.** The processed document's units, each with its text range, how many of the tree's blocks and parse diagnostics it holds, and the labels it defines.
- **What is parsed again.** The units the edit overlaps, and one unit either side. An edit reaches across unit boundaries: a deleted blank line joins two units, a typed `---` line turns the paragraph above it into a heading. The slice runs from the end of the unit before it to the start of the unit after it, so the blank lines and stray lines between units are always parsed with the units around them. A slice that starts at the start of the text is parsed as the document is, with frontmatter allowed.
- **Resync.** The slice is parsed as a text of its own, placed at its origin, as the parser already does for fallbacks and definition bodies. Its last unit has to come out as it was before, moved: the same blocks and diagnostics. If it doesn't (a fence or a container opened and not closed takes in what follows), the slice is parsed again with twice as many units after the edit, until its last unit comes out as before or the slice reaches the end of the body. Doubling keeps the worst case, a fence opened at the top that takes in the whole rest, to about twice the cost of parsing the whole body, with no limit to tune.
- **Labels.** The slice is parsed with the labels the rest of the document defines (decision 5), so references parse as they would in place.
- **The units after the slice** keep their nodes. Their positions move by the edit's change in lines and offsets. Columns don't change: each block after the slice starts on a line of its own. Moving positions costs a pass over those nodes, far less than parsing them. Positions relative to each block would remove that pass, later.
- **The frontmatter** is read only when the edit touches it.

### Labels

A **label** is the name a link reference definition or a footnote definition gives, written in brackets at the start of a line:

```markdown
[docs]: https://example.com
[^1]: A footnote.
```

Here the labels are `docs` and `1`. Labels are compared without case and with runs of white space as one space, so `[Docs]` and `[docs]` are the same.

In CommonMark, whether brackets make a link depends on the labels the *whole* document defines, anywhere in it, before or after the reference:

```markdown
See [the docs][docs] and the note[^1].

[docs]: https://example.com
[^1]: A footnote.
```

With the two definitions, `[the docs][docs]` is a link and `[^1]` a footnote reference. Without them, both are plain text with the brackets in it.

Parsed alone, the slice `See [the docs][docs] and the note[^1].` has no definitions, so the fork would read both as text, and the tree would differ from parsing from scratch.

**What the fork lacks.** The fork collects the labels while it parses, in two lists (`definitions` and `gfm_footnote_definitions` in its parse state), and decides each reference against them. But it only knows the labels of the text it is given: it has no way to be told about definitions outside that text. That is the change: a new parse option whose labels fill those lists before parsing starts. The rest of the fork stays as it is.

```rust
markdown::ParseOptions {
    // The labels the rest of the document defines.
    definitions: vec!["docs".into()],
    footnote_definitions: vec!["1".into()],
    ..options
}
```

The processor keeps the document's labels with its blocks. An edit that adds, removes or renames a definition changes them, which is why it parses the whole body again (see [Edits](#edits)).

## The later stages

Parsing says which units changed. The stages after it keep what they give for each top-level block, so that an edit redoes only the blocks of those units. A unit that comes out of the slice parse as it was, or moved, counts as unchanged, so the unit on either side of the edit, parsed again to resync, isn't redone.

The text and where its lines start are kept with the processed document, and an edit updates both. The line starts count bytes and code points, so a position in code points is found in its line, without counting from the start of the text.

## Stage 2: resolving

- Imported documents depend only on the host's files and packs, not on the processed document's text, so they are kept across edits. A host that changes a file or a pack starts a new processor.
- For an edit inside one body block, the names and definitions of the processed document don't change. Only that block's name diagnostics (F002, F023, W113) are found again. The definitions after the edit, and the diagnostics at them, move.
- A unit that changed and holds a definition or a signature fence counts as a change to a definition, even when the edit is outside the fence: stages 2 to 4b run on the whole document.

## Stage 3: validating

- The processed document is validated block by block, and each block keeps what validating gave it: its diagnostics, its refused params, its invocations that render their fallback, and its parsed expressions.
- **One fact crosses blocks:** the inputs bound to params of host-language components, which a data import's `from` may not interpolate (F031). Each block keeps its share, and the data imports, which are few, are checked again after every edit.
- An expression's parsed program is shared by every block that writes it, and dropped when no block writes it any more.
- Definitions, the processed document's and the imported ones', are validated once and kept until a definition changes.

## Stage 4a: loading

- Loading depends on the frontmatter and the render context, and on which data imports validating refused. An edit to the body keeps what was loaded, unless it changed which data imports are refused (F031): then loading and expanding run again.

## Stage 4b: expanding

- The processed document is expanded block by block, and each block keeps how many expanded nodes it gave, its diagnostics and the state it starts and ends with.
- **Three things cross blocks:**
  - **The last heading.** Headings in a `compmd` definition are relative to the heading before them (spec §7.2). Each block notes whether it reads the depth of the heading before it (a `compmd` component with relative headings, before any heading of its own) and whether it leaves the depth as it found it. When the depth before a block changes, the block is expanded again only if it reads it.
  - **The node limit.** Each block keeps how many nodes it made, and the limit applies to their running total. A block whose start is past the limit, or that would cross it, is expanded again.
  - **Errors reported once** (spec §10.4). An evaluation error is reported once per code and place, however many blocks reach it, for example in a definition that several blocks use. Each block keeps all the errors it found, and the diagnostics keep the first of each, so removing the block that reported one moves the report to the next block that finds it.
- Origins hold absolute positions, so a block after the edit has its origins moved as its parse tree is. An origin's occurrence counts the nodes before it in the same list with the same range, and nodes of different top-level blocks never share a range, so occurrences stay right.
- Adjacent text is joined in any list (spec §8.1). Every top-level block gives blocks, never bare text, so no text of two blocks is joined today. If it ever were, the blocks couldn't be told apart, and the next edit expands the whole document.

## What `edit` gives

The same expanded processor, with the full trees and diagnostics as today, and what changed:

```rust
/// The processed document's top-level expanded nodes `[from, to)` of the last result became `nodes`; every node
/// after them moved by `lines` lines and `offset` code points.
pub struct Splice { pub from: usize, pub to: usize, pub nodes: Vec<expanded::Node>, pub lines: isize, pub offset: isize }
```

`processor.splice()` gives it. A host that renders the whole tree each time ignores it; one that crosses a boundary (a worker, WebAssembly, a process) sends the splice instead of the tree. After the first expansion, the splice replaces an empty tree with the whole one; after a reload, the whole tree before it. When a block after the edited ones is expanded again (it follows a heading that changed), the splice runs to it, so the nodes between come again too.

## The WebAssembly binding

- `new Document(request)` processes a document once and keeps the processor; `edit(start, end, text)` gives `{splice, inputs, diagnostics}` as JSON: the splice with its moved positions, the root's inputs, which the splice doesn't cover, and every diagnostic.
- An edit to the frontmatter gives a splice like any other. A host that fetches ahead asks for `sources()`, in the form `sources(request)` gives them, and after fetching calls `reload(fetched)`, whose splice replaces the whole tree.
- Offsets cross the boundary in UTF-16, as a JavaScript string counts them. The binding keeps the text with each line's start in bytes, UTF-16 units and code points, so it finds a position in its line. An edit that removes a leading byte order mark, or writes text before it, changes where the processor starts counting, so the binding sends it as an edit of the whole text.
- The TypeScript wrapper, `OpenDocument`, keeps the whole expanded tree by applying each splice, moving the origins of the nodes after it in place, and gives it to `@compmd/render`, which already updates the page by key. Its tests compare the tree and diagnostics after every edit with processing from scratch.
- For an edit inside one block, neither the processing nor the JSON grows with the document. Moving the origins after it does.
- **Measured** (`pnpm bench` in `wasm/`, p95, an Apple M4): a keystroke takes 0.10 ms in 52 KB and 0.29 ms in 500 KB in Chrome, 0.08 ms and 0.28 ms in WebKit. With the splice applied in JavaScript, it is 0.14 ms and 0.84 ms (Chrome), 0.14 ms and 0.78 ms (WebKit): at 500 KB, applying the splice, which moves the origins of every node after the edit, takes most of it.

## Where edits come from

The processor takes `TextEdit`s, so any source of edits fits:

- **A text area:** the selection before the input is the range, and what was typed or pasted is the text.
- **CodeMirror:** each transaction's changes, in order.
- **A CRDT such as Loro:** its text events are deltas of retain, insert and delete, local and a collaborator's alike. Each delta becomes edits in order: a retain moves the position, a delete is an edit with empty text, an insert one with an empty range. Loro counts positions in Unicode scalars (code points) or UTF-16, as the host asks, so a host asks for the unit the processor or its binding takes.

A collaborator's edit arrives like a local one, so the preview follows a shared document at the same cost.

## Correctness

- **Differential test.** For every document of the conformance set and the bench: a sequence of random edits (typing and deleting characters, pasting blocks, adding and removing blank lines, opening and closing fences and containers, editing the frontmatter and definitions, link and footnote definitions). After each edit, the incremental result is compared with a new processor on the same text and context: parse tree, expanded tree and diagnostics. A failure prints the document and the edits.
  - Parsing (`parse/edit.rs`): the tree, the diagnostics and the units, over the conformance set and the examples; the blocks outside those it says changed are as they were, or moved.
  - Every stage (`tests/edit.rs`): every processor case with its context, the examples and the bench's document, each with origins and without; and the splice turns the expanded tree before the edit into the one after it.
- **Unit tests** for each piece: classifying an edit, resync, moving positions, the line starts, the heading state, the node limit, loading again, joined text.

## Performance

- **Bench:** time per keystroke inside one block of the 52 KB body and of a 500 KB body, and for each edit class.
- **What to expect.** Today the WebAssembly build takes about 10 ms for a 52 KB body, about 0.2 µs a byte. A keystroke parses, validates and expands its block and one block either side, a few hundred bytes to a few KB: tens to a few hundred microseconds, not nanoseconds. Moving the positions of the blocks after the edit adds a pass over their nodes, a few nanoseconds each, which does grow with the document: about 0.1 ms for 500 KB.
- **The target:** a keystroke inside a block of ordinary size takes under 1 ms (p95) in both documents.
- **Measured** (`cargo bench --bench edit`, p95, an Apple M4, native):

  | Edit | 52 KB | 500 KB |
  |---|---|---|
  | A keystroke in a paragraph | 0.03 ms | 0.11 ms |
  | A paragraph split by a blank line | 0.04 ms | 0.14 ms |
  | A link definition's label (the whole body parsed again) | 3.5 ms | 53 ms |
  | The frontmatter (every stage run again) | 3.7 ms | 57 ms |
  | The whole document, from scratch | 4.0 ms | 56 ms |

  Before the later stages kept their blocks, a keystroke took 0.43 ms and 3.7 ms.
- The browser budgets stay as they are: a key press is painted within 16 ms for a short document and 50 ms for a long one.

## Phases

1. **Parsing.** The labels option in the fork, fallbacks parsed with the document's labels (see [Found while designing](#found-while-designing)), the slice parse with resync, moving positions, classifying edits, and the splice of the parse tree. Stages 2 to 4b still run on the whole document. The differential test starts here.
2. **The other stages.** Imported documents kept; resolving, validating and expanding block by block; the result's `Splice`. The differential test covers every stage.
3. **The WebAssembly binding.** `open` and `edit`, the UTF-16 offsets, and the TypeScript wrapper that applies splices; the bench in a browser.

## Alternatives considered

- **Processing the whole document on each change**, as today. Simple, and fast enough for short documents, but the time grows with the document, which an editor can't afford.
- **Processing after a pause in typing.** Less work, but the preview always lags, and a long document still takes as long once the pause comes.
- **Moving the processing to a server.** Adds a round trip to every key press and breaks working offline.
- **Keeping the text in a CRDT inside the processor (Loro).** The processor would own the document's history and get edits from it directly. But the processor would carry a CRDT it has no use for (history, merging, version vectors), and its WebAssembly build would grow by the CRDT's size (Loro's alone is several hundred KB). Hosts that keep text elsewhere, a text area, CodeMirror, a file, would gain nothing. Loro's stable cursors could stand in for moving positions, but diagnostics and trees need lines and columns, which still have to be counted. The processor takes edits instead, and a host with Loro turns its deltas into them ([Where edits come from](#where-edits-come-from)).
- **A tree-sitter grammar as the parser.** tree-sitter reparses after an edit by reusing the unchanged parts of the old tree, and recovers from errors, which is what an editor wants. But:
  - The maintained Markdown grammar isn't conformant: its README says it is "not recommended to use this parser where correctness is important", while the spec's conformance set demands CommonMark and GFM exactly.
  - CompMD's syntax would need a grammar of our own on top of it, with an external scanner, in a second language, kept in step with the spec beside the fork.
  - tree-sitter's runtime is C, which makes the `wasm32-unknown-unknown` build harder than the pure Rust one today.
  - Processing would still need stages 2 to 4b per block, which tree-sitter doesn't give.

  What the design takes from tree-sitter is its edit model: an edit is a range of the old text and its replacement, the old tree is moved by it, and only what the edit touched is parsed again.
- **Incremental inside markdown-rs, below the block level** (re-parsing part of a paragraph or a list item). Finer, but the fork would have to keep its tokenizer's state at each block, a deep change to a parser we don't own. A top-level block is small enough: a paragraph, or one list.
- **Positions relative to each block, now.** It would remove moving positions after an edit, but every reader of positions (diagnostics, origins, the renderer's keys) would add the block's start. It is kept for later, if moving positions shows in the bench.

## Found while designing

- **Fallbacks don't see the document's labels today.** The parser parses each fallback and each definition body as a text of its own, without the labels the rest of the document defines. Checked on today's processor with:

  ````markdown
  ```compmd component=note
  ---
  See [the docs][docs].
  ```

  In place: [the docs][docs].

  :: chart[See [the docs][docs].]

  [docs]: https://example.com
  ````

  The paragraph gives a `linkReference`, as it should. The definition's body and the `chart` fallback both give the text `See [the docs][docs].` with the brackets in it.

  - **The fallback is a bug.** A fallback is inline Markdown of the document it is written in (spec §3.1), and a plain renderer shows the line as a paragraph in which `[the docs][docs]` is a link. Slot content, the other text the caller writes, resolves in the caller's document too (spec §6.3). Phase 1 fixes it by parsing fallbacks with the labels option.
  - **The definition body is right.** The spec already settles it (§6.3, "Scope is lexical"): a `compmd` definition's body is a document of its own, and resolves footnotes and link reference definitions among its own lines. A body that needs `[docs]` defines it in the body.

  For the record, the choices the spec had for a body:

  | A body sees the labels of | Gains | Loses |
  |---|---|---|
  | **its own lines only** (the spec) | A body means the same wherever it is used, from any document; it parses on its own, once; editing the document around it never changes it | A label defined next to the definition doesn't reach into it, which can surprise; a body repeats the definitions it needs |
  | its own lines and the defining document | Labels written next to the definition work in it | A body imported from a pack depends on labels outside its fence; editing a definition anywhere in the defining document changes how every body in it parses |
  | the document that uses the component | A component could link to a target each user defines | The same body parses differently for each use, so it can't be parsed or checked once; its tree (spec §12.1) would depend on who uses it, against lexical scope everywhere else in the spec |

  Keeping the spec's choice also keeps incremental parsing simpler: a body never needs the labels option.

- **A first `---` line that is never closed broke the fork's parse of the lines after it.** With markdown-rs's frontmatter construct on, `---\nus\n- a\n` parsed `- a` as part of the paragraph `us`, where CommonMark makes it a list, as it does with the construct off. The differential test found it, since a slice is parsed without the construct. The bug was upstream's tokenizer, with CommonMark constructs alone: it recorded each line's start one index early, which nothing read until a failed attempt that crossed line endings left entries behind, and the next pass then skipped lines. The fork fixes it, and `tests/frontmatter.rs` there checks that such text parses the same with the construct on and off.

## Found while building

- **Counting code points from the start of the text cost most of an edit.** The slice parse turned each position into a byte index by counting the code points before it, several times per edit: 0.8 ms of the 0.9 ms a keystroke took in the 500 KB body. The line starts kept with the text, in bytes and code points, now find a position in its line.
- **Some text made the fork panic.** `[l]:x\n-\n1\n-` (a link reference definition, then `-` lines) panicked in markdown-rs, in `to_html` with its default options too, and in upstream's release as well, so the bug was upstream's. The differential test found it from a random edit. The first `-`, with only a definition above it, moves into the paragraph `1` as its first line, and the second `-` makes that paragraph a heading; the second step renamed the paragraph's old start, which the first had removed. The fork fixes it, giving `<h2>-\n1</h2>` as micromark does, and `tests/fuzz.rs` there checks it and variants.
