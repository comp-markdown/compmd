// A document of the page made live: kept open by the processor, rendered into the element that held its snapshot, and
// processed again as its text is edited or its inputs change. It loads after the page has painted.
import { init, OpenDocument, type Diagnostic, type Request } from "@compmd/processor";
import { createPage, type Diagnostic as Located } from "@compmd/render";

/** A document made live. */
export interface Live {
  /** The text is now `text`; gives its diagnostics. */
  edit(text: string): readonly Diagnostic[];
  destroy(): void;
}

/** Where `before` and `after` differ: the range of `before` that became `inserted`, in UTF-16 units. */
export function changed(before: string, after: string): [start: number, end: number, inserted: string] {
  let start = 0;
  while (start < before.length && start < after.length && before[start] === after[start]) start++;
  let tail = 0;
  while (tail < before.length - start && tail < after.length - start && before[before.length - 1 - tail] === after[after.length - 1 - tail]) tail++;
  return [start, before.length - tail, after.slice(start, after.length - tail)];
}

const located = (diagnostics: readonly Diagnostic[]): Located[] => diagnostics.map(({ code, range }) => ({ code, line: range.start.line, column: range.start.column }));

/**
 * Renders the document `path`, whose text is `text`, into `target` in place of what it holds, and keeps it open.
 * `files` are the other files it reads.
 */
export async function liven(target: HTMLElement, path: string, text: string, files: Record<string, string> = {}): Promise<Live> {
  await init();
  let inputs: Record<string, unknown> = {};
  const request = (): Request => ({ path, files: { ...files, [path]: text }, now: new Date().toISOString(), inputs });
  let open = new OpenDocument(request());
  target.replaceChildren();
  const page = createPage(target, {
    // An input changed: the text is the same, so the document opens again with the input's value.
    onSet(input, value) {
      inputs = { ...inputs, [input]: value };
      open.free();
      open = new OpenDocument(request());
      page.render(open.expanded);
    },
  });
  page.render(open.expanded, { text });
  return {
    edit(next) {
      const [start, end, inserted] = changed(text, next);
      text = next;
      const { splice, diagnostics } = open.edit(start, end, inserted);
      page.splice([splice], { text, diagnostics: located(diagnostics) });
      return diagnostics;
    },
    destroy() {
      page.destroy();
      open.free();
    },
  };
}
