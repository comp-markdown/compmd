import { init, OpenDocument, type Diagnostic, type TextEdit } from "@compmd/processor";
import { codePointOffset, utf16Offset } from "@compmd/render";
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

// The text and the page, both ways: a click on the page puts the caret where what was clicked came from, and a
// click in the text shows on the page what the caret is in.
view.on("pointed", ({ source }) => {
  if (source.start.offset === undefined) return;
  const caret = utf16Offset(shown, source.start.offset);
  editor.setSelectionRange(caret, caret);
  showHint();
});
editor.addEventListener("click", () => {
  view.reveal(codePointOffset(shown, editor.selectionStart));
});

// A reader changed an input: the same text, opened again with the new value.
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded, { text: shown, diagnostics: forPage(opened.diagnostics) });
});
