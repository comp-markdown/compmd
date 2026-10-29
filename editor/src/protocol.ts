// The messages between the editor, its worker and its frame.

import type { Diagnostic, HostComponent, Request } from "@compmd/processor";
import type { Edit } from "./edit.ts";

/** From the editor to its worker: open a document, or apply edits to it, each in the text the one before left. */
export type ToWorker = { type: "open"; request: Request } | { type: "edits"; edits: Edit[] };

/** From the worker to the editor, once per message it was sent. */
export type FromWorker =
  | {
      type: "processed";
      /** What each edit did to the expanded tree, as the processor's JSON text, which the editor passes on unread. */
      splices: string[];
      /** Whether the document was opened: its one splice is of an empty tree, and replaces the page. */
      opened: boolean;
      diagnostics: Diagnostic[];
      /** The host-language components, when they changed (and after opening). */
      components?: readonly HostComponent[];
      /** How long processing took, in the worker. */
      ms: number;
    }
  | { type: "failed"; message: string };

/** From the editor to the frame, on the port `@compmd/render/frame` uses too: the components to compile. */
export interface ComponentsMessage {
  type: "components";
  components: readonly HostComponent[];
}

/** From the frame to the editor, on the same port: a component that couldn't be compiled or loaded. */
export interface ComponentFailed {
  type: "component-failed";
  component: string;
  message: string;
}

/** What the editor sends the frame first, with the port. */
export const FRAME_HELLO = "compmd-editor";
