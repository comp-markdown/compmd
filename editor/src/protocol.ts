// The messages between the editor and its worker, and the host stylesheets the editor offers its page.

import type { Diagnostic, ExpandedInput, HostComponent, PositionalHint, Request } from "@compmd/processor";
import type { Edit } from "./edit.ts";

/**
 * From the editor to its worker: open a document, or apply edits to it, each in the text the one before left; or only
 * the caret moved. Each carries where the caret is after it, in UTF-16 units, for the hint the answer gives.
 */
export type ToWorker = { type: "open"; request: Request; caret: number } | { type: "edits"; edits: Edit[]; caret: number } | { type: "hint"; caret: number };

/**
 * From the page to the worker, before the first document: the registries the playground's configuration names, which
 * the worker fetches missing packs from. The page has them from `virtual:compmd`, which a worker's own build lacks.
 */
export interface RegistryConfig {
  type: "registry";
  /** Each scope's registry by its URL, `*` for every other scope. */
  registries: Record<string, string>;
  fetch: "host" | "direct";
  maxPackSize?: number;
}

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
  | { type: "fetched"; arrived: boolean; problems: string[] };

/** The stylesheets a host might give the page, which the editor lets the author try: `frame.css?host=<id>`. */
export const HOST_STYLES = [
  { id: "reference", label: "Reference" },
  { id: "github", label: "GitHub-like" },
  { id: "docs", label: "Documentation" },
  { id: "fancy", label: "Fancy" },
  { id: "terminal", label: "Terminal" },
  { id: "magazine", label: "Magazine" },
  { id: "sketch", label: "Hand-drawn" },
  { id: "unstyled", label: "Unstyled" },
] as const;

export type HostStyle = (typeof HOST_STYLES)[number]["id"];
