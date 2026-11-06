// The CompMD processor as WebAssembly, typed: stages 1 to 4b of a document in one call, or a document kept open for an
// editor, which takes edits and keeps its expanded tree.

import type { ExpandedInput, ExpandedRoot, Splice } from "@compmd/render";
import {
  components as componentsJson,
  frontmatter as frontmatterJson,
  newestMatching as newestMatchingOf,
  plain,
  process as processJson,
  sources as sourcesJson,
} from "../pkg/compmd_wasm.js";
import type { ParseRoot, Signature } from "./parse-tree.ts";

export { default as init, initSync } from "../pkg/compmd_wasm.js";
export { OpenDocument, OpenDocumentJson } from "./open-document.ts";
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
  /** The host's policy for packs: those it lists with their trust, which a document may import, those it denies. */
  policy?: Policy;
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
  /**
   * `lenient` (the default), for previews and editors, or `strict`, for CI and publishing, in which any `F` diagnostic
   * fails processing (spec §10.1): {@link Processed.failed}. Both report the same diagnostics and make the same trees.
   */
  mode?: "lenient" | "strict";
  /**
   * What the host is still getting (spec §6.4, §8.6): packs by name, remote URLs and `host:` sources. A pending
   * pack's uses render their fallbacks marked as pending (`componentPending`), and pending data is its `fallback` or
   * `null`; neither is F026. Leave a source out once the host gives up on it, and it is F026, as any that can't be had.
   */
  pending?: Partial<Pending>;
}

/** Packs by name, remote URLs and `host:` sources (without `host:`) that are pending. */
export interface Pending {
  packs: string[];
  remote: string[];
  host: string[];
}

/** A release of a pack, with its files: `package.json`, the pack's document, and what it imports. */
export interface PackRelease {
  version: string;
  /** The integrity hash the registry publishes for it, `sha512-…`. */
  integrity?: string | null;
  /** Why its files can't be used, when the host found they don't match its integrity: an import of it is F035. */
  broken?: string | null;
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
  /** Whether processing failed: in `strict` mode, on any `F` diagnostic. The trees are there either way. */
  failed: boolean;
  /**
   * The processed document's data imports, in written order, each with the key a host keeps its cached copy of the
   * source by (spec §8.6): its final URL or path, the inputs filled in. An import whose origin isn't allowed has none.
   */
  cacheKeys: CacheKey[];
  /** What the render treated as pending, of what the request's `pending` names: for a host's "loading 1 pack". */
  pending: Pending;
}

/** A data import's cache key. */
export interface CacheKey {
  /** The import's name, `sales` for `md.data.sales`. */
  as: string;
  /** An `https:` URL, a `host:` source with `host:`, or a file's path in the root folder or in a pack (`@acme/kit@1.4.2/icons.json`). */
  key: string;
}

/** What a render needs that the request doesn't hold, each once, in the order it is written. */
export interface Needed {
  remote: string[];
  /** `host:` sources, without `host:`. */
  host: string[];
  /**
   * Remote URLs not fetched because their origin isn't in `allowedOrigins` (F028), for a host that asks the reader
   * about new origins first (spec §8.6): `loadData` in `@compmd/sandbox` takes them with `remote`.
   */
  disallowed: string[];
  /**
   * The packs the document and the packs it holds import that no release in `packs` matches (spec §6.4), for a host
   * that gets packs from a registry to fetch and add before it processes again; then the library packs those packs
   * name in `peerDependencies` that none matches. Packs the policy refuses aren't listed.
   */
  packs: NeededPack[];
}

/** A pack import, or a library a pack names, that the request has no release for. */
export interface NeededPack {
  /** The pack, `@kit/mermaid`. */
  pack: string;
  /** The version or range after the name, as written (`1`, `^1.4.2`, `1.4.2`); `*` when none is. For a library, its `peerDependencies` range. */
  range: string;
  /** The import's `integrity`, when it pins one with an exact version: the release's tarball must have this hash. */
  integrity?: string;
  /**
   * A library pack a pack names: the processor takes the newest release that matches every range asked for it, so a
   * host that fetches one fetches that version when the registry has it.
   */
  library?: true;
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
 * `remote` and `host` before it calls {@link process}; and the packs it imports that `packs` has no matching release
 * of, for a host that gets them from a registry. The processor fetches nothing.
 *
 * @example
 * const { remote, host, packs } = sources(request);
 * // packs: [{ pack: "@kit/mermaid", range: "1" }], when the document imports `@kit/mermaid@1` and the request has none
 *
 * @throws Error as {@link process} does.
 */
export function sources(request: Request): Needed {
  return JSON.parse(sourcesJson(JSON.stringify(request))) as Needed;
}

/**
 * The newest of `versions` that `range` takes, as the processor reads the version or range after a pack's name in an
 * import (spec §6.1): `1.4.2` is exactly that version, `1` and `^1.4` are ranges, `*` is any. `undefined` when none
 * matches. A host that fetches the packs {@link sources} lists picks a version with this, so the processor takes the
 * release it is given. Versions that aren't semver are skipped.
 *
 * @example
 * newestMatching("1", ["1.0.0", "1.3.0", "2.0.0"]); // "1.3.0"
 */
export function newestMatching(range: string, versions: string[]): string | undefined {
  return newestMatchingOf(range, versions);
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

/** The components a document can name (spec §6.3), and the names it uses that stand for none. */
export interface Components {
  /**
   * The components by the file that defines them: the document first (without `file`), then the files it imports in
   * written order, then the packs the processor provides (`provided`).
   */
  groups: { file?: string; provided: boolean; components: ListedComponent[] }[];
  /** The names the document uses that stand for no one component: unknown (F002), ambiguous (F023) or from an import that failed. */
  unknown: { name: string; uses: Required<Range>[] }[];
}

/** A component a document can name. */
export interface ListedComponent {
  /** The name the document gives it. */
  name: string;
  /** Its name in the file that defines it, when an import renames it. */
  definedAs?: string;
  /** What implements it: `compmd`, `processor`, or the host language as written (`jsx`, `svelte`, …). */
  kind: string;
  /** Absent for a host-language component without one, whose params go unchecked. */
  signature?: Signature;
  /** Where the document defines it, its definition fence; absent for one another file defines. */
  defined?: Required<Range>;
  /** Where the document invokes it, each the range of the name. */
  uses: Required<Range>[];
}

/**
 * The components `request`'s document can name, grouped by the file that defines them, each with where the document
 * uses it.
 *
 * @throws Error as {@link process} does.
 */
export function componentsOf(request: Request): Components {
  return JSON.parse(componentsJson(JSON.stringify(request))) as Components;
}

/**
 * What is at a place in an open document's text, and what it takes: the processor's positional hint. Ranges are in
 * lines and code point columns, as diagnostics' are.
 */
export interface PositionalHint {
  /** The text the hint is about: a name, a key, a value. A completion replaces it; empty where nothing is written yet. */
  range: Required<Range>;
  /** What the text is; absent where only a diagnostic has something to say. */
  subject?: HintSubject;
  /** What may be written at `range`, each replacing it. */
  completions: Completion[];
  /** The diagnostics whose range holds the place. */
  diagnostics: Diagnostic[];
}

/** What the text at a place is. */
export type HintSubject =
  | { kind: "component"; name: string; component?: ComponentHint }
  | { kind: "params"; component: string; params: ParameterHint[] }
  | { kind: "param"; component: string; param: ParameterHint; value?: string }
  | { kind: "slot"; component: string; slot: string; slots: NonNullable<Signature["slots"]> }
  | { kind: "path"; path: string; name: NameHint }
  | { kind: "function"; name: string; hint?: FunctionHint }
  | { kind: "frontmatter"; path: string[]; about?: string }
  | { kind: "type"; name: string; takes?: KindHint };

/** A component a name stands for. */
export interface ComponentHint {
  definedAs?: string;
  /** `compmd`, `processor`, or the host language as written. */
  kind: string;
  signature?: Signature;
  /** The file that defines it; absent for the processed document. */
  file?: string;
  /** Whether the processor provides it: `@compmd/core` or an ambient pack. */
  provided: boolean;
  /** Where in that file it is defined. */
  defined?: Required<Range>;
}

/** A param: what it is and what it takes. */
export interface ParameterHint {
  name: string;
  origin: "declared" | "common" | "attribute";
  takes: KindHint;
  required: boolean;
  default?: string;
  doc?: string;
  /** Whether it may carry a breakpoint, `md:gap=l`. */
  breakpoints: boolean;
}

/** What a value may be, and how a message says it. */
export interface KindHint {
  /** The words it takes; for a bound input, the inputs it can bind. */
  tokens: string[];
  forms: string[];
  examples: string[];
  upTo: number;
  flag: boolean;
  quoted: boolean;
  /** What it takes, as F009 says it. */
  description: string;
}

/** What a step of a path names. */
export interface NameHint {
  name: string;
  /** Its type, as CEL writes it; absent for a name nothing has. */
  type?: string;
  about?: string;
  /** The fields of a record, or of a list's records, each `name: type`. */
  fields?: string[];
}

/** A function of the profile, or an `md.` function. */
export interface FunctionHint {
  /** Each way to call it: `string.indexOf(string) → int`. */
  signatures: string[];
  doc: string;
}

/** What may be written at a hint's range. */
export interface Completion {
  label: string;
  text: string;
  detail?: string;
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
  /** A fence's content, or a component file's text; empty for a built module, which loads from `module`. */
  code: string;
  /** For a built module, a `.js` file in a pack (spec §6.4): where it is, for the renderer to load from the pack's files. */
  module?: { pack: string; version: string; path: string; integrity?: string };
  /** How far its code is trusted: its pack's level in the request's `policy` (spec §15). */
  trust: Trust;
  /**
   * For a component of a pack, the library packs its pack names in `peerDependencies`, by the name its code imports,
   * each with the module chosen for the document; absent when there are none.
   */
  libraries?: Record<string, LibraryModule>;
}

/** A library pack's module chosen for a document, and the libraries it imports in turn, by name. */
export interface LibraryModule {
  pack: string;
  version: string;
  /** The module's path in the library pack, as its `compmd` field names it. */
  path: string;
  libraries?: Record<string, LibraryModule>;
}

/** How far a pack's code is trusted (spec §15); a permissive pack's components get `host` when `host` is true. */
export type Trust = { level: "strict" } | { level: "permissive"; network: string[]; host: boolean } | { level: "trusted" };

/** The host's policy for packs (spec §6.4, §15). Every key is optional: the default allows any pack, in the strict sandbox. */
export interface Policy {
  /** The packs the host lists, each with its trust level; any other is strict. `network` defaults to none, `host` to false. */
  packs?: Record<string, { level: "strict" } | { level: "permissive"; network?: string[]; host?: boolean } | { level: "trusted" }>;
  /** `listed`: a document may import only the packs listed here and the ambient ones (F026 otherwise). */
  imports?: "any" | "listed";
  /** Packs the host denies, as its own policy: all versions, or those in the semver range `version` (F026, with `reason`). */
  deny?: { pack: string; version?: string; reason: string }[];
  /** Packs the host left out, such as those larger than it serves, each with why (F026). */
  refused?: Record<string, string>;
  /**
   * The runtimes the host runs (spec §7.6), by npm name and version, such as `{ name: "svelte", version: "5.57.2" }`:
   * a component on another is W102, and a pack's `peerDependencies` on them are checked (W121). Absent: every runtime
   * of the spec's bindings, unchecked.
   */
  runtimes?: Library[];
  /** The libraries the host offers component code (spec §6.3), checked against a pack's `peerDependencies` (W121). */
  libraries?: Library[];
}

/** A runtime or library the host provides component code: its npm name and the version it has. */
export interface Library {
  name: string;
  version: string;
}

/** An expression's value in a document's scope ({@link OpenDocument.evaluate}), or its F024. */
export type Evaluated = { value: unknown } | { error: Diagnostic };

/** Data the host fetched for {@link OpenDocument.reload}, as a request gives it, and what of it is still pending. */
export interface FetchedData {
  remote?: Record<string, Fetched>;
  host?: Record<string, unknown>;
  pending?: Partial<Omit<Pending, "packs">>;
}

