// The views of the document beside the page: what a plain CommonMark and GFM renderer shows (spec §14), what its
// frontmatter gives the render (its fields, settings, data and inputs), the components it can name, and its trees
// (spec §12). The editor's worker
// keeps the document open and gives only what changed, so these come from a second worker that processes the
// document from scratch, and only while a tab shows one.

import { componentsOf, frontmatterOf, OpenDocumentJson, plainHtml, process, type Components, type Diagnostic, type Evaluated, type Frontmatter, type Request } from "@compmd/processor";

/** A view the second worker makes: a tab's, or the diagnostics, for the inspector, which has no editor's. */
export type ViewKind = "trees" | "plain" | "data" | "components" | "diagnostics";

/**
 * From the editor to the views' worker: the document as it is now, and the view to make of it; or an expression the
 * Console evaluates in it, numbered by `id`.
 */
export type ViewRequest = { request: Request; view: ViewKind } | { request: Request; view: "evaluate"; id: number; expression: string };

/** A view of a document, or why it couldn't be made; or an expression's value in it, or its F024. */
export type View =
  | { view: "trees"; parse: unknown; expanded: unknown }
  | { view: "plain"; html: string }
  | { view: "data"; frontmatter: Frontmatter }
  | { view: "components"; components: Components }
  | { view: "diagnostics"; diagnostics: Diagnostic[] }
  | { view: "evaluate"; id: number; result: Evaluated }
  | { view: ViewKind | "evaluate"; id?: number; failed: string };

/** `expression`'s value in `request`'s document, opened for it and freed after. */
function evaluated(request: Request, expression: string): Evaluated {
  const document = new OpenDocumentJson(request);
  try {
    return document.evaluate(expression);
  } finally {
    document.free();
  }
}

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/** What the views' worker does with each request. */
export function viewOf(asked: ViewRequest): View {
  if (asked.view !== "evaluate") return documentView(asked.request, asked.view);
  const { request, id, expression } = asked;
  try {
    return { view: "evaluate", id, result: evaluated(request, expression) };
  } catch (error) {
    return { view: "evaluate", id, failed: messageOf(error) };
  }
}

/** The view `view` of `request`'s document. */
function documentView(request: Request, view: ViewKind): View {
  try {
    if (view === "plain") return { view, html: plainHtml(request.files?.[request.path] ?? "") };
    if (view === "data") return { view, frontmatter: frontmatterOf(request) };
    if (view === "components") return { view, components: componentsOf(request) };
    const { parse, expanded, diagnostics } = process(request);
    if (view === "diagnostics") return { view, diagnostics };
    return { view, parse, expanded };
  } catch (error) {
    return { view, failed: messageOf(error) };
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
 * takes its place, since only the latest text and tab matter. The diagnostics wait apart from a tab's view, so asking
 * for one never drops the other. Expressions to evaluate wait in order, and go first: each gets its answer.
 */
export class ViewsClient {
  readonly #worker: ViewsPort;
  #busy = false;
  readonly #waiting = new Map<"tab" | "diagnostics", ViewRequest>();
  readonly #evaluations: ViewRequest[] = [];

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
    this.#waiting.set(view === "diagnostics" ? "diagnostics" : "tab", { request, view });
    if (!this.#busy) this.#send();
  }

  /** Asks for `expression`'s value in `request`'s document; its answer has `id`. */
  evaluate(request: Request, id: number, expression: string): void {
    this.#evaluations.push({ request, view: "evaluate", id, expression });
    if (!this.#busy) this.#send();
  }

  #send(): void {
    const next = this.#evaluations.shift() ?? this.#nextView();
    if (!next) return;
    this.#busy = true;
    this.#worker.postMessage(next);
  }

  /** The view that waits longest, taken from the waiting ones. */
  #nextView(): ViewRequest | undefined {
    const [next] = this.#waiting;
    if (!next) return undefined;
    const [slot, waiting] = next;
    this.#waiting.delete(slot);
    return waiting;
  }
}
