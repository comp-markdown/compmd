# Incremental processing

**Status: proposal.** How the processor handles an edit without processing the whole document again.

Today every change runs stages 1 to 4b on the whole text. The WebAssembly build takes about 3 ms for the spec's complete example and 10 ms for a 52 KB body (p95, an Apple M4), and the time grows with the document: a body ten times as long would take about ten times as long. An editor that processes on every key press needs the time to depend on the edit, not on the document.

## Decisions

1. **The unit is a top-level block** of the processed document: a paragraph, a heading, a whole list, a whole block quote, a whole `:::` container, a fence. A block nested in a list or a container can't be parsed on its own, because Markdown's block structure depends on what is around it; a top-level block can.
2. **An edit is classified, and its class decides what is redone** (see [Edits](#edits)). An edit inside one body block redoes that block, through every stage.
3. **The result equals processing from scratch:** the same parse tree, expanded tree and diagnostics. A differential test checks this over random edits.
4. **One more method on the expanded processor**, next to `reload`:

   ```rust
   let processor = processor.edit(&TextEdit { start, end, text }, &context, &fetch, &data);
   ```

   The `Processor` stays the only way in.
5. **The markdown-rs fork gets a parse option for the labels defined elsewhere** in the document (link and footnote definitions), so a slice parses as it would in place.
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

A host with several changes at once (a paste over a selection, a collaborator's delta, changes made while the last edit was processed) applies them in order, or merges them into one edit over the text the processor last saw.

| The edit touches | What is redone |
|---|---|
| The frontmatter, or creates or removes one | Everything: imports, inputs, data and settings may all change |
| A link reference or footnote definition, or creates or removes one | Stage 1 for the whole body, since whether `[x][ref]` is a link depends on which labels are defined; stages 2 to 4b for the blocks whose tree changed |
| A `compmd` definition fence of the processed document | Stage 1 for that block; stages 2 to 4b for the processed document, since its names, signatures and recursion (F019) may change. Imported documents are kept |
| Any other block of the body | That block, through every stage |
| Nothing in the text: inputs or the render time changed | Stages 4a and 4b, as `reload` does today |

An edit is classified by the blocks it overlaps before and after it is parsed: typing `[a]: /x` at the start of a paragraph turns it into a definition, so the class is known only after stage 1 has parsed the changed blocks.

## Stage 1: parsing

- **What is kept.** The processed document's top-level blocks, each with its text range, its nodes and its parse diagnostics.
- **What is parsed again.** The blocks the edit overlaps, and one block either side. An edit reaches across block boundaries: a `---` line turns the paragraph above it into a heading, a deleted blank line joins two blocks, a lazy continuation line joins the block before it.
- **Resync.** The slice is parsed as a text of its own, placed at its origin, as the parser already does for fallbacks and definition bodies. Its last block has to end where the slice ends. If it doesn't (a fence or a container opened and not closed takes in what follows), the slice grows by the next block and is parsed again. Past a limit, the whole body is parsed.
- **Labels.** The slice is parsed with the labels the rest of the document defines (decision 5), so references parse as they would in place.
- **The blocks after the slice** keep their nodes. Their positions move by the edit's change in lines and offsets. Columns don't change: each block after the slice starts on a line of its own. Moving positions costs a pass over those nodes, far less than parsing them. Positions relative to each block would remove that pass, later.
- **The frontmatter** is read only when the edit touches it.

## Stage 2: resolving

- Imported documents depend only on the host's files and packs, not on the processed document's text, so they are kept across edits. A host that changes a file or a pack starts a new processor.
- For an edit inside one body block, the names and definitions of the processed document don't change. Only that block's name diagnostics (F002, F023, W113) are found again.

## Stage 3: validating

- The processed document is validated block by block, and each block keeps what validating gave it: its diagnostics, its refused params, its invocations that render their fallback, and its parsed expressions.
- **One fact crosses blocks:** the inputs bound to params of host-language components, which a data import's `from` may not interpolate (F031). Each block keeps its share, and the data imports are checked again only when the whole set changes.
- Definitions, the processed document's and the imported ones', are validated once and kept until a definition changes.

## Stage 4a: loading

- Loading depends on the frontmatter and the render context, never on the body, so an edit to the body keeps what was loaded.

## Stage 4b: expanding

- The processed document is expanded block by block, and each block keeps its expanded nodes, its diagnostics and the state it starts and ends with.
- **Two things cross blocks:**
  - **The last heading.** Headings in a `compmd` definition are relative to the heading before them (spec §7.2). When a block's last heading changes, the blocks after it are expanded again, until one starts with the state it had before.
  - **The node limit.** Each block keeps how many nodes it made, and the limit applies to their running total.
- Origins hold absolute positions, so a block after the edit has its origins moved as its parse tree is. An origin's occurrence counts the nodes before it in the same list with the same range, and nodes of different top-level blocks never share a range, so occurrences stay right.

## What `edit` gives

The same expanded processor, with the full trees and diagnostics as today, and what changed:

```rust
/// The processed document's top-level expanded nodes `[from, to)` of the last result became `nodes`; every node
/// after them moved by `lines` lines and `offset` code points.
pub struct Splice { pub from: usize, pub to: usize, pub nodes: Vec<expanded::Node>, pub lines: isize, pub offset: isize }
```

A host that renders the whole tree each time ignores it; one that crosses a boundary (a worker, WebAssembly, a process) sends the splice instead of the tree.

## The WebAssembly binding

- `open(request)` processes a document once and keeps the processor; `edit(start, end, text)` gives the splice, the diagnostics and the moved positions, as JSON.
- Offsets cross the boundary in UTF-16, as a JavaScript string counts them, and the binding turns them into code points.
- The TypeScript wrapper keeps the whole expanded tree by applying each splice, and gives it to `@compmd/render`, which already updates the page by key.
- For an edit inside one block, neither the processing nor the JSON grows with the document.

## Where edits come from

The processor takes `TextEdit`s, so any source of edits fits:

- **A text area:** the selection before the input is the range, and what was typed or pasted is the text.
- **CodeMirror:** each transaction's changes, in order.
- **A CRDT such as Loro:** its text events are deltas of retain, insert and delete, local and a collaborator's alike. Each delta becomes edits in order: a retain moves the position, a delete is an edit with empty text, an insert one with an empty range. Loro counts positions in Unicode scalars (code points) or UTF-16, as the host asks, so a host asks for the unit the processor or its binding takes.

A collaborator's edit arrives like a local one, so the preview follows a shared document at the same cost.

## Correctness

- **Differential test.** For every document of the conformance set and the bench: a sequence of random edits (typing and deleting characters, pasting blocks, adding and removing blank lines, opening and closing fences and containers, editing the frontmatter and definitions, link and footnote definitions). After each edit, the incremental result is compared with a new processor on the same text and context: parse tree, expanded tree and diagnostics. A failure prints the document and the edits.
- **Unit tests** for each piece: classifying an edit, resync, moving positions, the heading state, the node limit.

## Performance

- **Bench:** time per keystroke inside one block of the 52 KB body and of a 500 KB body, and for each edit class. Keystrokes inside a block should take about the same time in both documents.
- The browser budgets stay as they are: a key press is painted within 16 ms for a short document and 50 ms for a long one.

## Phases

1. **Parsing.** The labels option in the fork, the slice parse with resync, moving positions, classifying edits, and the splice of the parse tree. Stages 2 to 4b still run on the whole document. The differential test starts here.
2. **The other stages.** Imported documents kept; validating and expanding block by block; the result's `Splice`. The differential test covers every stage.
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

## Open questions

- **Fallbacks and definition bodies** are already parsed as texts of their own, without the labels the document defines, so a reference link written in one may not resolve as it would in place. The labels option should be given to them too; check whether they parse wrongly today.
- **The limit before resync parses the whole body:** a number of blocks, or a share of the document. To be set from the bench.
