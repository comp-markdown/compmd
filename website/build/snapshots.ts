// The documents in `content/` as the page shows them before the processor loads, and without JavaScript: each one's
// text, the page it renders and that page's stylesheet, and what a plain Markdown renderer shows of it.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { initSync, plainHtml, process, type Diagnostic } from "@compmd/processor";
import { createPage } from "@compmd/render";
import { Window } from "happy-dom";

/** A document as the page shows it before the processor loads. */
export interface Snapshot {
  /** The document's text. */
  text: string;
  /** The page it renders. */
  html: string;
  /** The page's stylesheet. */
  css: string;
  /** What a plain CommonMark and GFM renderer shows of it (spec §14). */
  plain: string;
}

/** The extension of a CompMD document. */
export const EXTENSION = ".comp.md";

let loaded = false;

/** Loads the processor's module, once. */
function load(): void {
  if (loaded) return;
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
  loaded = true;
}

const where = ({ code, message, range }: Diagnostic) => `${code} at ${String(range.start.line)}:${String(range.start.column)}: ${message}`;

/**
 * Renders the document `path` of `files` as it renders at `now`. Each page's block classes start `compmd-b0`, so
 * they are renamed `compmd-<prefix>-b0`, and pages of several documents can share one HTML page.
 *
 * @throws Error when the document has a diagnostic: an example shows CompMD as it is meant to be written.
 */
export function snapshot(path: string, files: Record<string, string>, now: string, prefix: string): Snapshot {
  load();
  const text = files[path] ?? "";
  const { expanded, diagnostics } = process({ path, files, now });
  if (diagnostics.length > 0) throw new Error(`${path}: ${diagnostics.map(where).join("; ")}`);
  const window = new Window();
  // happy-dom's elements are the DOM's, as the renderer takes them.
  const root = window.document.body.appendChild(window.document.createElement("div")) as unknown as HTMLElement;
  const page = createPage(root);
  page.render(expanded);
  // An input's state is in its properties, which HTML doesn't hold; its attributes give it to the snapshot.
  for (const input of root.querySelectorAll("input")) {
    if (input.type === "checkbox") input.toggleAttribute("checked", input.checked);
    else input.setAttribute("value", input.value);
  }
  const renamed = (rendered: string) => rendered.replace(/\bcompmd-b(\d+)\b/g, `compmd-${prefix}-b$1`);
  const snapshot = { text, html: renamed(root.innerHTML), css: renamed(page.stylesheet.text), plain: plainHtml(text) };
  window.close();
  return snapshot;
}

/** Every document in `folder`, by its name without `.comp.md`, rendered at `now`; each reads the folder's other files. */
export function snapshots(folder: string, now: string): Record<string, Snapshot> {
  const files = Object.fromEntries(readdirSync(folder).map((name) => [name, readFileSync(join(folder, name), "utf8")]));
  const documents = Object.keys(files).filter((name) => name.endsWith(EXTENSION)).sort();
  return Object.fromEntries(
    documents.map((path) => {
      const name = path.slice(0, -EXTENSION.length);
      return [name, snapshot(path, files, now, name)];
    }),
  );
}
