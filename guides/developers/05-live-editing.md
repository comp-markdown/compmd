# 5. Live editing

An editor processes on every key press, so it can't process the whole document each time. This chapter builds a text area with a live page beside it: it keeps the document open, sends each edit, and gives the page only what changed, so the page follows the text as it is typed, and components keep their state. It lists each diagnostic with a button for its fix, and says what is at the caret.

## An open document

`new OpenDocument(request)` processes a document once and keeps it. Each `edit(start, end, text)` then processes only the blocks the edit touches, and gives what it did:

```ts
const opened = new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now, origins: true });
page.render(opened.expanded);
// The reader selected `world` and typed `there`.
const { splice, diagnostics, components } = opened.edit(6, 11, "there");
```

- **An edit** is the range of the old text that changed and what replaced it, in UTF-16 units, as a JavaScript string counts them: a text area's `selectionStart`, CodeMirror's `from` and `to`. One edit at a time, each in the text the one before left.
- **A change** gives `splice`, what the edit did to the expanded tree; every `diagnostics` of the document; and `components`, only when the components the tree names changed.
- **The open document keeps the tree.** `opened.expanded` is always the whole tree, `opened.diagnostics` and `opened.components` the latest.
- `free()` releases the document's memory in the module. Call it when you open the document again, or close it.

Give the open document the editor's own text, `textarea.value` or CodeMirror's `state.doc.toString()`, not the file it was read from: a text area turns line endings into LF, and both must count the same text.

For an edit inside one block, a keystroke takes about 0.1 ms in a 52 KB document and 0.3 ms in a 500 KB one ([incremental processing](../../processor/docs/incremental.md) has how, and what an edit to the frontmatter or a definition redoes).

## Splices

A splice says which of the tree's top-level nodes changed: the nodes `from` to `to` became `nodes`, and every node after them moved by `lines` lines and `offset` code points. It also has the root's `inputs` and the `edit` that made it, in code points.

```ts
interface Splice {
  from: number;
  to: number;
  nodes: ExpandedNode[];
  lines: number;
  offset: number;
  inputs?: Record<string, ExpandedInput>;
  edit?: TextEdit;
}
```

A page applies it with `page.splice([splice], change)`, and a sandbox's view with `view.splice`: only the replaced blocks render again. `change` is `{text, diagnostics}`, the text after the edit and its diagnostics, each `{code, line, column}`. A component in a block the splice didn't touch keeps its instance; one in a block it did is matched to its old view by its key.

**Keys come from origins.** With `origins: true`, every expanded node carries its origin, the range of the text that made it. The page gives each origin an id, follows it through each edit as an editor follows positions, and gives a node whose range lands where an old one went the old one's view. So typing inside a card keeps the card, and its component's state, and typing above it only moves it. Without origins, nodes are matched by place.

**Text being typed is often invalid.** Given the diagnostics, the page keeps what it showed where the text is broken mid-typing:

- frontmatter that doesn't parse (F001, F033) keeps the last good page, until it parses again;
- a top-level container not closed yet (F004) shows its opener as a paragraph, and what follows stays as it was;
- any other error in a top-level block keeps that block's last good render.

So pass the diagnostics of the processed document itself (those without `file`), as `forPage` does below.

## The editor

A text area, a line for the hint, a list for the diagnostics, and the page:

```html snippet=live-editor/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>A live CompMD editor</title>
  </head>
  <body style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem">
    <div>
      <textarea id="text" spellcheck="false" style="width: 100%; height: 20rem; font-family: monospace"></textarea>
      <p id="hint"></p>
      <ul id="problems"></ul>
    </div>
    <main id="page" style="height: 24rem"></main>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

The document it starts with, `dinner.comp.md`:

```md snippet=live-editor/dinner.comp.md
---
md:
  inputs:
    guests: number = 4
---

# Dinner for :value[4]{of=(string(md.input.guests))}

:: number-input[Guests: 4] {bind=guests}

::: box {padding=m border=subtle border-radius=m}
**To buy:** :value[600 g]{of=(string(md.input.guests * 150) + " g")} of rice, and a lemon.
::: /box

Edit this text, and the page follows it as you type.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/live-editor) in the playground.

And `main.ts`. The app is the quick start's, with the same `vite.config.ts`:

```ts snippet=live-editor/main.ts
import { init, OpenDocument, type Diagnostic, type TextEdit } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./dinner.comp.md?raw";

await init();

const editor = document.getElementById("text") as HTMLTextAreaElement;
const hint = document.getElementById("hint") as HTMLElement;
const problems = document.getElementById("problems") as HTMLElement;
editor.value = text;

// The document, open with origins, which the page's keys come from; `shown` is the text it has.
let inputs: Record<string, unknown> = {};
let shown = editor.value;
const open = () =>
  new OpenDocument({ ...request, path: "dinner.comp.md", files: { "dinner.comp.md": shown }, now: new Date().toISOString(), inputs, origins: true });
let opened = open();

const view = createSandbox(sandbox).open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);
showProblems(opened.diagnostics);

/** The processor's diagnostics of the text itself, as the page's rules for invalid text read them. */
const forPage = (diagnostics: readonly Diagnostic[]) =>
  diagnostics.filter(({ file }) => file === undefined).map(({ code, range }) => ({ code, line: range.start.line, column: range.start.column }));

/** The edit that turned `before` into `after`: what lies between the text they start and end with in common. */
function editOf(before: string, after: string): TextEdit {
  let start = 0;
  while (start < before.length && before[start] === after[start]) start++;
  let kept = 0;
  while (kept < before.length - start && kept < after.length - start && before[before.length - 1 - kept] === after[after.length - 1 - kept]) kept++;
  // Whole characters: 😀 and 😁 share their first UTF-16 unit.
  const second = (at: number) => /[\uDC00-\uDFFF]/.test(before.charAt(at));
  if (second(start)) start--;
  if (second(before.length - kept)) kept--;
  return { start, end: before.length - kept, text: after.slice(start, after.length - kept) };
}

/** The text area holds the text after `edit`: the open document takes it, and the page gets its splice. */
function edited(edit: TextEdit) {
  shown = editor.value;
  const { splice, diagnostics, components } = opened.edit(edit.start, edit.end, edit.text);
  if (components) void view.components(components);
  view.splice([splice], { text: shown, diagnostics: forPage(diagnostics) });
  showProblems(diagnostics);
  showHint();
}

editor.addEventListener("input", () => {
  edited(editOf(shown, editor.value));
});

/** Where a line and column, counted in code points as diagnostics count them, are in `text`, in UTF-16 units. */
function offsetOf(text: string, { line, column }: { line: number; column: number }): number {
  let at = text.split("\n").slice(0, line - 1).reduce((sum, before) => sum + before.length + 1, 0);
  for (let counted = 1; counted < column; counted++) at += (text.codePointAt(at) ?? 0) > 0xffff ? 2 : 1;
  return at;
}

/** Each diagnostic, with a button for its fix, which edits the text as typing does. */
function showProblems(diagnostics: readonly Diagnostic[]) {
  problems.replaceChildren(
    ...diagnostics.map(({ code, message, range, fix }) => {
      const item = document.createElement("li");
      item.textContent = `${String(range.start.line)}:${String(range.start.column)} ${code} ${message} `;
      if (fix) {
        const button = document.createElement("button");
        button.textContent = `Write ${fix.text}`;
        button.addEventListener("click", () => {
          const edit = { start: offsetOf(shown, fix.range.start), end: offsetOf(shown, fix.range.end), text: fix.text };
          editor.setRangeText(edit.text, edit.start, edit.end, "end");
          edited(edit);
        });
        item.append(button);
      }
      return item;
    }),
  );
}

/** What is at the caret: a param and what it takes, a component, or the diagnostic there. */
function showHint() {
  const at = opened.hintAt(editor.selectionStart);
  const subject = at?.subject;
  if (subject?.kind === "param") hint.textContent = `${subject.param.name} takes ${subject.param.takes.description}`;
  else if (subject?.kind === "component") hint.textContent = `${subject.name}, a ${subject.component?.kind ?? "unknown"} component`;
  else hint.textContent = at?.diagnostics[0]?.message ?? "";
}

editor.addEventListener("keyup", showHint);
editor.addEventListener("click", showHint);

// A reader changed an input: the same text, opened again with the new value.
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded, { text: shown, diagnostics: forPage(opened.diagnostics) });
});
```

Run it, and type in the text area: a line typed at the end shows on the page as you type it. Change the guests to 6 on the page, and the heading says *Dinner for 6* and the box *900 g*, while the text stays as you wrote it. Write `border=subtl`: the list shows F009 with a *Write subtle* button, which mends the text as typing would. Put the caret in `border`, and the line under the text says what it takes.

## Edits from a text area

`editOf` finds the edit by comparing the text before and after an input: what lies between what both start and end with. That is always a right edit, whatever the input was: typing, deleting, pasting, undo. It keeps a character that takes two UTF-16 units, such as an emoji, whole.

It can place an edit where the reader didn't make it: cutting `:: notes[Notes]` from in front of `::: card` leaves `::` in front of `card`, and a comparison alone puts the edit inside the card's opener. The page's keys follow the edit, so the card may lose its state. The playground also reads the selection before each input: when the text outside it didn't change, the selection is what was replaced. An editor with a model of its own, such as CodeMirror, has the edits already, in each transaction's changes, in order.

A diagnostic's fix is an edit too. Its range is in lines and code point columns, so `offsetOf` turns it into UTF-16 positions in the text; the fix then goes through `edited` like a key press. A fix is of the text its diagnostic was found in: here every edit is processed before the next, so they are always the same text.

## Hints at the caret

`hintAt(offset)` says what is at a UTF-16 position of the text as the last change left it ([positional hints](../../processor/docs/positional-hints.md)), or `undefined` in plain prose:

| Field | What it is |
| --- | --- |
| `range` | the text the hint is about, a name, a key, a value, in lines and code point columns; a completion replaces it |
| `subject` | what it is: a `component`, the `params` of one, a `param` with what it `takes`, a `slot`, a `path` in an interpolation or an expression, a `function`, a `frontmatter` key, or a `type` |
| `completions` | what may be written at `range`, each `{label, text, detail?}` |
| `diagnostics` | the diagnostics whose range holds the position |

A hint is a lookup, not a pass, so it is cheap enough for every caret move. The example shows a param's `takes.description`, the words its F009 uses, so a hint and a diagnostic never say different things. An editor with completion as you type lists `completions`, and writes the one chosen over `range`, as a fix is written.

## Keeping a page in step with a text

The example keeps three things in step: the text area's text, the open document's (`shown`), and the page's. Each input changes the first, `edited` the other two, in that order, before the next input. Two things open the document again instead of editing it:

- **An input changed.** The text is the same, so the new document's tree has the same origins, and `view.render(tree, change)` with the same text keeps every block's keys.
- **Anything else in the request changed**, a file the document imports, a pack, data you fetched: an open document keeps what it was opened with, so open it again with the new request. After an edit to the frontmatter that names a URL or a `host:` source the request doesn't hold, the document shows its fallback (or F026, without one) until you fetch it, and `opened.sources()` lists it, with `opened.reload({remote, host})` to give it ([chapter 6](06-data-and-inputs.md)).

## The playground's way: a worker

The example processes in the text area's input handler, which is fine for a short document. The playground never does: a long document's frontmatter edit takes milliseconds, and the key press would be painted that much later. Its editor sends each edit to a worker, which keeps the document open, and the page gets the splices from it.

The worker holds an `OpenDocumentJson`, which is `OpenDocument` without the tree: each change's splice comes as the module's JSON text, which the worker posts and the page passes to its view unread. Posted text is copied in one piece, and the renderer parses it once, in the page's frame.

```ts
// editor.worker.ts
import { init, OpenDocumentJson, type Request, type TextEdit } from "@compmd/processor";

const ready = init();
let opened: OpenDocumentJson | undefined;
addEventListener("message", async ({ data }: MessageEvent<{ request: Request } | { edits: TextEdit[] }>) => {
  await ready;
  if ("request" in data) {
    opened?.free();
    opened = new OpenDocumentJson(data.request);
    postMessage({ opened: true, changes: [opened.changes()] });
  } else if (opened) {
    const document = opened;
    postMessage({ opened: false, changes: data.edits.map(({ start, end, text }) => document.edit(start, end, text)) });
  }
});
```

`changes()` gives what the last change did; after opening, the whole tree as one splice, of an empty tree. Each change is `{splice, diagnostics, inputs, components?}`, with the inputs apart, since the page doesn't read the splice. The page gives them all to its view at once: `view.splice(changes.map(({ splice }) => splice), { text, diagnostics }, { replace: opened, inputs: last.inputs })`, where `replace` says the first splice is of an empty tree, as after opening.

While the worker works, the reader types on. The playground's editor doesn't queue a message per key press: edits made while the worker is busy wait, and go together in the next message, each with positions in the text the one before left, and the worker applies them in order. A host may also merge them into one edit that covers them all. Each message carries the caret, so each answer brings the hint at it, of the text just processed, and a caret that moves while nothing is typed asks for the hint alone.

Next: [data and inputs](06-data-and-inputs.md), where a document's data comes from, and the values a reader changes.
