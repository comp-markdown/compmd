// A document kept open for an editor: processed once, then each edit processes only the blocks it touches and gives
// what changed, as JSON text for a host that passes it on, or applied to the expanded tree it keeps.

import type { ExpandedInput, ExpandedRoot, Splice } from "@compmd/render";
import { Document as Kept } from "../pkg/compmd_wasm.js";
import type { CacheKey, Changes, ChangesJson, Diagnostic, Evaluated, FetchedData, HostComponent, Needed, Pending, PositionalHint, Request } from "./index.ts";

/** Moves every origin under `value` by `lines` and `offset`. */
function move(value: unknown, lines: number, offset: number): void {
  if (Array.isArray(value)) {
    for (const item of value) move(item, lines, offset);
  } else if (value !== null && typeof value === "object") {
    for (const [key, field] of Object.entries(value)) {
      if (key === "origin") {
        for (const point of [(field as Origin).start, (field as Origin).end]) {
          point.line += lines;
          point.offset += offset;
        }
      } else if (typeof field === "object") {
        move(field, lines, offset);
      }
    }
  }
}

interface Origin {
  start: { line: number; offset: number };
  end: { line: number; offset: number };
}

/**
 * A document kept open, as {@link OpenDocument} is, for a host that passes each change on to where it renders without
 * reading it, such as an editor's worker to the page's frame: each splice comes as JSON text, which is posted in one
 * piece and parsed once, by the renderer (`@compmd/render`'s `splice`). It keeps no tree.
 *
 * @example
 * // in the worker
 * const document = new OpenDocumentJson({ ...request, origins: true });
 * postMessage(document.changes());                   // the whole tree, as one splice
 * postMessage(document.edit(start, end, inserted));  // { splice, diagnostics, components? }
 *
 * @throws Error when the request isn't valid, as `process` does.
 */
export class OpenDocumentJson {
  readonly #kept: Kept;

  constructor(request: Request) {
    this.#kept = new Kept(JSON.stringify(request));
  }

  /** What the last change gave; after opening, the whole tree as one splice, and every component. */
  changes(): ChangesJson {
    const components = this.#kept.components();
    return {
      splice: this.#kept.splice(),
      diagnostics: JSON.parse(this.#kept.diagnostics()) as Diagnostic[],
      inputs: JSON.parse(this.#kept.inputs()) as Record<string, ExpandedInput>,
      ...(components === undefined ? {} : { components: JSON.parse(components) as HostComponent[] }),
    };
  }

  /** The text's UTF-16 `start..end` became `text`, as {@link OpenDocument.edit}. */
  edit(start: number, end: number, text: string): ChangesJson {
    this.#kept.edit(start, end, text);
    return this.changes();
  }

  /** What is at UTF-16 position `offset`, as {@link OpenDocument.hintAt}. */
  hintAt(offset: number): PositionalHint | undefined {
    return (JSON.parse(this.#kept.hint(offset)) as PositionalHint | null) ?? undefined;
  }

  /** `expression`'s value in the document's scope, as {@link OpenDocument.evaluate}. */
  evaluate(expression: string): Evaluated {
    return JSON.parse(this.#kept.evaluate(expression)) as Evaluated;
  }

  /** What a render needs now that the request didn't hold, as {@link OpenDocument.sources}. */
  sources(): Needed {
    return JSON.parse(this.#kept.sources()) as Needed;
  }

  /** Each data import's cache key for the inputs as they are now, as {@link OpenDocument.cacheKeys}. */
  cacheKeys(): CacheKey[] {
    return JSON.parse(this.#kept.cacheKeys()) as CacheKey[];
  }

  /** What the last render treated as pending, as {@link OpenDocument.pending}. */
  pending(): Pending {
    return JSON.parse(this.#kept.pending()) as Pending;
  }

  /** Loads and expands again with the data the host fetched, as {@link OpenDocument.reload}. */
  reload(fetched: FetchedData): ChangesJson {
    this.#kept.reload(JSON.stringify(fetched));
    return this.changes();
  }

  /** Releases the module's memory for the document. */
  free(): void {
    this.#kept.free();
  }
}

/**
 * A document kept open for an editor. It is processed once; each edit then processes only the blocks it touches, and
 * gives what changed. It keeps the expanded tree by applying each change. The module must be loaded first, with
 * `init` or `initSync`; {@link OpenDocument.free} releases it.
 *
 * Positions are the editor's, in UTF-16 units: `textarea.selectionStart`, CodeMirror's `from` and `to`. Give it the
 * editor's text (`textarea.value`, `state.doc.toString()`), not the file it was read from, so both count the same text.
 *
 * The tree it gives is updated in place by later changes: only the top-level array is new each time. A page renders
 * each change's splice (`page.splice`), or the whole tree.
 *
 * @example
 * const document = new OpenDocument({ path: "notes.comp.md", files: { "notes.comp.md": text }, now, origins: true });
 * page.render(document.expanded);
 * // The user selected `world` and typed `there`.
 * page.splice([document.edit(6, 11, "there").splice]);
 *
 * @throws Error when the request isn't valid, as `process` does.
 */
export class OpenDocument {
  readonly #json: OpenDocumentJson;
  readonly #origins: boolean;
  #expanded: ExpandedRoot = { type: "root", children: [] };
  #diagnostics: Diagnostic[] = [];
  #components: HostComponent[] = [];

  constructor(request: Request) {
    this.#json = new OpenDocumentJson(request);
    this.#origins = request.origins ?? false;
    this.#apply(this.#json.changes());
  }

  /** The expanded tree. */
  get expanded(): ExpandedRoot {
    return this.#expanded;
  }

  /** The diagnostics of every stage, in stage order. */
  get diagnostics(): readonly Diagnostic[] {
    return this.#diagnostics;
  }

  /** The host-language components the tree can name, with their code. */
  get components(): readonly HostComponent[] {
    return this.#components;
  }

  /** The text's UTF-16 `start..end` became `text`; one edit per change, each in the text the one before left. */
  edit(start: number, end: number, text: string): Changes {
    return this.#apply(this.#json.edit(start, end, text));
  }

  /**
   * What is at UTF-16 position `offset` (as `edit` takes it) of the text as the last change left it, and what it
   * takes: a component's name, a param's key or value, a slot, a path, a function, a frontmatter key; with what may be
   * written there and the diagnostics there. `undefined` where there is nothing to say, such as in plain prose.
   *
   * @example
   * const hint = document.hintAt(textarea.selectionStart);
   * if (hint?.subject?.kind === "param") show(hint.subject.param.takes.description, hint.completions);
   */
  hintAt(offset: number): PositionalHint | undefined {
    return this.#json.hintAt(offset);
  }

  /**
   * Evaluates `expression`, a CEL expression of the profile without its parentheses (spec §8.2), in the scope the
   * document's body sees at its top level, as the last change left it: `md.doc`, `md.data`, `md.input` with the
   * inputs' values the render used, `md.version`, `md.lang`, `md.tz`, and the `md.` functions at the render's time. It
   * is checked against the document's types, as an expression in the document is, and evaluated within the same work
   * budget. Nothing of the document changes: for a developer's console.
   *
   * @returns `{value}`, as JSON gives it; or `{error}`, F024 with its message and its line and column in `expression`
   * (a syntax error, an unknown name with the nearest known one, an evaluation error).
   *
   * @example
   * const result = document.evaluate("md.data.sales.filter(s, s.region == md.input.region).size()");
   * if ("error" in result) show(result.error.message, result.error.range.start.column);
   * else show(result.value);
   */
  evaluate(expression: string): Evaluated {
    return this.#json.evaluate(expression);
  }

  /**
   * The remote and `host:` sources a render needs now, and the packs the request has no release of. After an edit to
   * the frontmatter, a host that fetches ahead fetches what it hasn't yet and calls {@link reload}; a pack it gets
   * needs the document opened again with it.
   */
  sources(): Needed {
    return this.#json.sources();
  }

  /**
   * Each data import's cache key for the inputs as they are now, as `Processed.cacheKeys`: its final URL or path,
   * which a host keeps its fetched copies by. Computed when asked, not with each change.
   *
   * @example
   * const cached = new Map(document.cacheKeys().map(({ as, key }) => [as, cache.get(key)]));
   */
  cacheKeys(): CacheKey[] {
    return this.#json.cacheKeys();
  }

  /**
   * What the last render treated as pending, of what the request (packs) and the last reload (data) said is: for a
   * host's "loading 1 pack".
   */
  pending(): Pending {
    return this.#json.pending();
  }

  /** Loads and expands again with the data the host fetched, in place of what it was given before. */
  reload(fetched: FetchedData): Changes {
    return this.#apply(this.#json.reload(fetched));
  }

  /** Releases the module's memory for the document. */
  free(): void {
    this.#json.free();
  }

  #apply({ splice: json, diagnostics, components }: ChangesJson): Changes {
    const splice = JSON.parse(json) as Splice;
    const { from, to, nodes, lines, offset } = splice;
    const children = this.#expanded.children;
    const after = children.slice(to);
    if (this.#origins && (lines !== 0 || offset !== 0)) move(after, lines, offset);
    this.#expanded = {
      type: "root",
      children: children.slice(0, from).concat(nodes, after),
      ...(splice.inputs ? { inputs: splice.inputs } : {}),
    };
    this.#diagnostics = diagnostics;
    if (components) this.#components = components;
    return { splice, diagnostics, ...(components ? { components } : {}) };
  }
}
