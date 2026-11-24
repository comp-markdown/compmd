// The views of the document beside the page: what a plain CommonMark and GFM renderer shows (spec §14), what its
// frontmatter gives the render (its fields, settings, data and inputs), the components it can name, and its trees
// (spec §12). The editor's worker
// keeps the document open and gives only what changed, so these come from a second worker that processes the
// document from scratch, and only while a tab shows one.

import { componentsOf, frontmatterOf, OpenDocumentJson, plainHtml, process, type Components, type Diagnostic, type Evaluated, type Frontmatter, type Request } from "@compmd/processor";
import { messageOf } from "./message-of.ts";
import { WorkerLink, type WorkerPort } from "./worker-link.ts";

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

/**
 * What the plain view's page may load: its own styles and inline images, nothing from the network, so a remote image
 * the document names isn't fetched without the reader being asked (spec §15), and no script runs.
 */
export const PLAIN_POLICY = "default-src 'none'; style-src 'unsafe-inline'; img-src data:";

/**
 * A plain renderer's HTML as a page of its own, with plain styles, light or dark as the reader's system is, under
 * {@link PLAIN_POLICY}.
 */
export function plainPage(html: string): string {
  const style = [
    ":root { color-scheme: light dark; font: 15px/1.6 system-ui, sans-serif; }",
    "body { margin: 0; padding: 16px 24px; }",
    "pre, code { font: 13px/1.5 ui-monospace, Menlo, monospace; }",
    "pre { padding: 12px; overflow: auto; background: color-mix(in srgb, currentColor 6%, transparent); }",
    "table { border-collapse: collapse; } th, td { border: 1px solid color-mix(in srgb, currentColor 25%, transparent); padding: 4px 10px; }",
    "blockquote { margin-inline: 0; padding-inline-start: 1em; border-inline-start: 3px solid color-mix(in srgb, currentColor 25%, transparent); }",
  ].join("\n");
  return `<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="${PLAIN_POLICY}"><style>${style}</style></head><body>${html}</body></html>`;
}

/** What the editor needs of the views' worker: a `Worker` is one. */
export type ViewsPort = WorkerPort<ViewRequest, View>;

/**
 * Asks the views' worker for a view, one request at a time: a request made while it is busy waits, and a later one
 * takes its place, since only the latest text and tab matter. The diagnostics wait apart from a tab's view, so asking
 * for one never drops the other. Expressions to evaluate wait in order, and go first: each gets its answer.
 */
export class ViewsClient {
  readonly #worker: WorkerLink<ViewRequest, View>;
  /** The request the worker is making a view of; none while it waits. */
  #asked: ViewRequest | undefined;
  readonly #waiting = new Map<"tab" | "diagnostics", ViewRequest>();
  readonly #evaluations: ViewRequest[] = [];

  /**
   * Asks `worker`, and calls `onView` with each answer. A worker that dies answers the request it was making with why,
   * as a view that failed, so nothing waits for it.
   */
  constructor(worker: ViewsPort, onView: (view: View) => void) {
    this.#worker = new WorkerLink(
      worker,
      (view) => {
        this.#asked = undefined;
        onView(view);
        this.#send();
      },
      (message) => {
        const asked = this.#asked;
        this.#asked = undefined;
        if (asked) onView({ view: asked.view, ...(asked.view === "evaluate" ? { id: asked.id } : {}), failed: message });
      },
    );
  }

  /** Ends the worker, as the Developer Tools close. */
  close(): void {
    this.#worker.close();
  }

  /** Asks for `view` of `request`'s document. */
  request(request: Request, view: ViewKind): void {
    this.#waiting.set(view === "diagnostics" ? "diagnostics" : "tab", { request, view });
    if (!this.#asked) this.#send();
  }

  /** Asks for `expression`'s value in `request`'s document; its answer has `id`. */
  evaluate(request: Request, id: number, expression: string): void {
    this.#evaluations.push({ request, view: "evaluate", id, expression });
    if (!this.#asked) this.#send();
  }

  #send(): void {
    const next = this.#evaluations.shift() ?? this.#nextView();
    if (!next) return;
    this.#asked = next;
    this.#worker.post(next);
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
