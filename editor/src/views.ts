// The views of the document beside the page: what a plain CommonMark and GFM renderer shows (spec §14), what its
// frontmatter gives the render (its fields, settings, data and inputs), and its trees (spec §12). The editor's worker
// keeps the document open and gives only what changed, so these come from a second worker that processes the
// document from scratch, and only while a tab shows one.

import { frontmatterOf, plainHtml, process, type Frontmatter, type Request } from "@compmd/processor";

/** A view the second worker makes. */
export type ViewKind = "trees" | "plain" | "data";

/** From the editor to the views' worker: the document as it is now, and the view to make of it. */
export interface ViewRequest {
  request: Request;
  view: ViewKind;
}

/** A view of a document, or why it couldn't be made. */
export type View =
  | { view: "trees"; parse: unknown; expanded: unknown }
  | { view: "plain"; html: string }
  | { view: "data"; frontmatter: Frontmatter }
  | { view: ViewKind; failed: string };

/** What the views' worker does with each request. */
export function viewOf({ request, view }: ViewRequest): View {
  try {
    if (view === "plain") return { view, html: plainHtml(request.files?.[request.path] ?? "") };
    if (view === "data") return { view, frontmatter: frontmatterOf(request) };
    const { parse, expanded } = process(request);
    return { view, parse, expanded };
  } catch (error) {
    return { view, failed: error instanceof Error ? error.message : String(error) };
  }
}

/** A plain renderer's HTML as a page of its own, with plain styles, light or dark as the reader's system is. */
export function plainPage(html: string): string {
  const style = [
    ":root { color-scheme: light dark; font: 15px/1.6 system-ui, sans-serif; }",
    "body { margin: 0; padding: 16px 24px; }",
    "pre, code { font: 13px/1.5 ui-monospace, Menlo, monospace; }",
    "pre { padding: 12px; overflow: auto; background: color-mix(in srgb, currentColor 6%, transparent); }",
    "table { border-collapse: collapse; } th, td { border: 1px solid color-mix(in srgb, currentColor 25%, transparent); padding: 4px 10px; }",
    "blockquote { margin-inline: 0; padding-inline-start: 1em; border-inline-start: 3px solid color-mix(in srgb, currentColor 25%, transparent); }",
  ].join("\n");
  return `<!doctype html><html><head><meta charset="utf-8"><style>${style}</style></head><body>${html}</body></html>`;
}

/** What the editor needs of the views' worker: a `Worker` is one. */
export interface ViewsPort {
  postMessage(message: ViewRequest): void;
  addEventListener(type: "message", listener: (event: { data: View }) => void): void;
}

/**
 * Asks the views' worker for a view, one request at a time: a request made while it is busy waits, and a later one
 * takes its place, since only the latest text and tab matter.
 */
export class ViewsClient {
  readonly #worker: ViewsPort;
  #busy = false;
  #waiting: ViewRequest | undefined;

  /** Asks `worker`, and calls `onView` with each answer. */
  constructor(worker: ViewsPort, onView: (view: View) => void) {
    this.#worker = worker;
    worker.addEventListener("message", ({ data }) => {
      this.#busy = false;
      onView(data);
      this.#send();
    });
  }

  /** Asks for `view` of `request`'s document. */
  request(request: Request, view: ViewKind): void {
    this.#waiting = { request, view };
    if (!this.#busy) this.#send();
  }

  #send(): void {
    const waiting = this.#waiting;
    if (!waiting) return;
    this.#waiting = undefined;
    this.#busy = true;
    this.#worker.postMessage(waiting);
  }
}
