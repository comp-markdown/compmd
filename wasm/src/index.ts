// The CompMD processor as WebAssembly: stages 1 to 4b of a document in one call, typed.

import type { ExpandedRoot } from "@compmd/render";
import { process as processJson, sources as sourcesJson } from "../pkg/compmd_wasm.js";
import type { ParseRoot } from "./parse-tree.ts";

export { default as init, initSync } from "../pkg/compmd_wasm.js";
export type * from "./parse-tree.ts";
export type { ExpandedRoot } from "@compmd/render";

/** A document to process, with everything it needs. Only `path` and `now` are required. */
export interface Request {
  /** The processed document's path in `files`. */
  path: string;
  /** The project's files by their `/`-separated paths: the processed document, what it imports, its data. */
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
