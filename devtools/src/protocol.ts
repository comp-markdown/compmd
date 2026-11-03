// The messages between the editor and its worker.

import type { Diagnostic, ExpandedInput, HostComponent, PositionalHint, Request } from "@compmd/processor";
import type { Edit } from "./edit.ts";

/**
 * From the editor to its worker: open a document, or apply edits to it, each in the text the one before left; or only
 * the caret moved. Each carries where the caret is after it, in UTF-16 units, for the hint the answer gives.
 */
export type ToWorker = { type: "open"; request: Request; caret: number } | { type: "edits"; edits: Edit[]; caret: number } | { type: "hint"; caret: number };

/**
 * Where the worker fetches the packs a document imports that its request doesn't hold, as a host's configuration names
 * them (docs/packs-from-a-registry.md).
 */
export interface Registries {
  /** Each scope's registry by its URL, `*` for every other scope. */
  registries: Record<string, string>;
  /** Who fetches: the host's server, or the reader's browser. */
  fetch: "host" | "direct";
  /** The largest pack it fetches, in bytes, unpacked. */
  maxPackSize?: number;
}

/** From the page to the worker, before the first document: the registries, which a worker can't be built with. */
export type RegistryConfig = { type: "registry" } & Registries;

/** From the worker to the editor, once per message it was sent. */
export type FromWorker =
  | {
      type: "processed";
      /** What each edit did to the expanded tree, as the processor's JSON text, which the editor passes on unread. */
      splices: string[];
      /** Whether the document was opened: its one splice is of an empty tree, and replaces the page. */
      opened: boolean;
      diagnostics: Diagnostic[];
      /** The document's inputs after them, by name, with their values. */
      inputs: Record<string, ExpandedInput>;
      /** The host-language components, when they changed (and after opening). */
      components?: readonly HostComponent[];
      /** The checked text of the built modules they load that were fetched from the registry, by `pack@version/path`. */
      modules?: Record<string, Blob>;
      /** What is at the caret in the text the edits left, when there is something to say. */
      hint?: PositionalHint;
      /** How long processing took, in the worker. */
      ms: number;
    }
  | { type: "hint"; hint?: PositionalHint }
  | { type: "failed"; message: string }
  /**
   * Not an answer: packs the document imports came from the registry (`arrived`: the editor opens it again with them),
   * or some couldn't be had (`problems`).
   */
  | { type: "fetched"; arrived: boolean; problems: string[] }
  /**
   * Not an answer: remote URLs the document needs that the worker hasn't reported yet, once typing pauses. The page
   * asks the reader about their origins and fetches them (`loadData`, spec §8.6), never the worker.
   */
  | { type: "remote"; urls: string[] };
