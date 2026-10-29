// The CompMD processor as WebAssembly, typed: stages 1 to 4b of a document in one call, or a document kept open for an
// editor, which takes edits and keeps its expanded tree.

import type { ExpandedInput, ExpandedRoot, Splice } from "@compmd/render";
import { Document as Kept, frontmatter as frontmatterJson, plain, process as processJson, sources as sourcesJson } from "../pkg/compmd_wasm.js";
import type { ParseRoot } from "./parse-tree.ts";

export { default as init, initSync } from "../pkg/compmd_wasm.js";
export type * from "./parse-tree.ts";
export type { ExpandedInput, ExpandedRoot, Splice, TextEdit } from "@compmd/render";

/** A document to process, with everything it needs. Only `path` and `now` are required. */
export interface Request {
  /** The processed document's path in `files`. */
  path: string;
  /** The files of the document's root folder by their `/`-separated paths: the processed document, what it imports, its data. */
  files?: Record<string, string>;
  /** The registry's packs by name, each with its releases. */
  packs?: Record<string, PackRelease[]>;
  /** Packs every document imports, at the lowest priority (spec §6.3). */
  ambient?: string[];
  /** Whether to report each use of a name from an ambient pack (W113). */
  reportAmbient?: boolean;
  /** The origins remote data may come from, such as `https://status.example.org`. */
  allowedOrigins?: string[];
  /** Remote data fetched beforehand, by URL; {@link sources} lists the URLs a render needs. */
  remote?: Record<string, Fetched>;
  /** The host's data, by source without `host:`; {@link sources} lists the sources a render needs. */
  host?: Record<string, unknown>;
  /** The render time, RFC 3339: what `md.now()` and `md.today()` give. */
  now: string;
  /** The processor's time zone, `md.tz` for a document without a `tz` key: an IANA name, or `UTC` (the default). */
  tz?: string;
  /** The values given for the document's inputs, by name. */
  inputs?: Record<string, unknown>;
  /** Whether expanded nodes carry their origins (spec §12.3), as a live preview needs. */
  origins?: boolean;
}

/** A release of a pack, with its files: `package.json`, the pack's document, and what it imports. */
export interface PackRelease {
  version: string;
  /** The integrity hash the registry publishes for it, `sha512-…`. */
  integrity?: string | null;
  files: Record<string, string>;
}

/** What a remote URL gave. */
export interface Fetched {
  text: string;
  /** The `Content-Type` header, which decides the format; without one, the URL's extension does. */
  contentType?: string | null;
}

/** A place in a document: 1-based line, 1-based column in code points. */
export interface Location {
  line: number;
  column: number;
}

export interface Range {
  start: Location;
  end?: Location;
}

/** A diagnostic (spec §10.5). */
export interface Diagnostic {
  code: string;
  severity: "error" | "warning";
  message: string;
  /** The document it is in, when it isn't the processed one. */
  file?: string;
  range: Range;
  /** Related places, innermost first. */
  related?: { file?: string; range: Range; message: string }[];
  stage: "parse" | "resolve" | "validate" | "load" | "evaluate";
  /** The edit that fixes it, when its message suggests one: the text at `range`, on one line, becomes `text`. */
  fix?: { range: Required<Range>; text: string };
}

/** What processing gave. */
export interface Processed {
  parse: ParseRoot;
  expanded: ExpandedRoot;
  /** The diagnostics of every stage, in stage order. */
  diagnostics: Diagnostic[];
}

/** The remote and `host:` sources a render needs, in the order they are written. */
export interface Needed {
  remote: string[];
  /** `host:` sources, without `host:`. */
  host: string[];
}

/**
 * Runs stages 1 to 4b on `request`'s document. The module must be loaded first, with {@link init} or
 * {@link initSync}.
 *
 * @throws Error when the request isn't valid (such as a `now` that isn't RFC 3339) or `path` isn't one of its files.
 */
export function process(request: Request): Processed {
  return JSON.parse(processJson(JSON.stringify(request))) as Processed;
}

/**
 * The remote and `host:` sources a render of `request`'s document needs, for the host to fetch and pass in
 * `remote` and `host` before it calls {@link process}.
 *
 * @throws Error as {@link process} does.
 */
export function sources(request: Request): Needed {
  return JSON.parse(sourcesJson(JSON.stringify(request))) as Needed;
}

/** What a change gave. */
export interface Changes {
  /** What it did to the expanded tree, with the root's inputs and, for an edit, the edit in code points. */
  splice: Splice;
  /** Every diagnostic of the document, in stage order. */
  diagnostics: Diagnostic[];
  /** The host-language components, when the change changed them (and after opening). */
  components?: HostComponent[];
}

/** What a change gave, with the splice as JSON text, as {@link OpenDocumentJson} gives it. */
export interface ChangesJson extends Omit<Changes, "splice"> {
  /** The {@link Splice}, as JSON text. */
  splice: string;
  /** The document's inputs after it, by name, as the splice holds them too, for a host that doesn't read the splice. */
  inputs: Record<string, ExpandedInput>;
}

/** What a document's frontmatter gives its render (spec §2.1, §8.6, §8.7). */
export interface Frontmatter {
  /** The author's own fields, what `md.doc` holds. */
  doc: Record<string, unknown>;
  /** The settings under `md` other than data and inputs. */
  settings: {
    version?: string;
    lang?: string;
    tz?: string;
    /** Each component import, with the names it takes (`badge as tag` when renamed), or `*`. */
    import?: { from: string; names: string[] }[];
  };
  /** Each data import, in written order, with what was loaded: `null` when it failed, which a diagnostic says. */
  data: { name: string; from: string; schema?: [string, unknown][]; fallback?: string; value: unknown }[];
  /** The inputs, by name, with the values the render used. */
  inputs: Record<string, ExpandedInput>;
}

/**
 * What the frontmatter of `request`'s document gives its render: its fields, its settings, each data import with what
 * was loaded, its inputs.
 *
 * @throws Error as {@link process} does.
 */
export function frontmatterOf(request: Request): Frontmatter {
  return JSON.parse(frontmatterJson(JSON.stringify(request))) as Frontmatter;
}

/**
 * What a plain CommonMark and GFM renderer shows of `text` (spec §14), as HTML: CompMD's syntax as text, raw HTML
 * escaped. The module must be loaded first.
 */
export function plainHtml(text: string): string {
  return plain(text);
}

/**
 * A host-language component the expanded tree can name, with its code, for a renderer that runs it (spec §7.4): the
 * tree names a component but doesn't hold its code. A name two documents give different components is the processed
 * document's, or else the first loaded.
 */
export interface HostComponent {
  /** The name its nodes have in the expanded tree. */
  name: string;
  language: "jsx" | "tsx" | "svelte" | "js" | "ts";
  /** The fence's `runtime=`; absent for the binding's default. */
  runtime?: string;
  /** A fence's content, or a component file's text. */
  code: string;
}

/** Data the host fetched for {@link OpenDocument.reload}, as a request gives it. */
export interface FetchedData {
  remote?: Record<string, Fetched>;
  host?: Record<string, unknown>;
}

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
 * @throws Error when the request isn't valid, as {@link process} does.
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

  /** The remote and `host:` sources a render needs now, as {@link OpenDocument.sources}. */
  sources(): Needed {
    return JSON.parse(this.#kept.sources()) as Needed;
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
 * {@link init} or {@link initSync}; {@link OpenDocument.free} releases it.
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
 * @throws Error when the request isn't valid, as {@link process} does.
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
   * The remote and `host:` sources a render needs now. After an edit to the frontmatter, a host that fetches ahead
   * fetches what it hasn't yet and calls {@link reload}.
   */
  sources(): Needed {
    return this.#json.sources();
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
