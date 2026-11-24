// The sandbox's frame protocol (docs/sandbox-protocol.md): the messages between a frame and its parent, the window
// that made it. They travel on the port `@compmd/render`'s frame protocol uses too, beside its own.

import type { CodeLines, ExpandedInput, ExpandedUse, TypeStyle } from "@compmd/render";

/**
 * The protocol's version, in `hello` and `ready`: a frame of another version gets nothing. 2 added `highlight` and
 * `highlighted`, and the level's `timeout`; 3 made a use frame's `size` its component's natural size, and added `box`
 * (docs/sizing-components.md).
 */
export const SANDBOX_VERSION = 3;

/** What the parent first posts to the frame's window, with the frame's port. */
export const HELLO = "compmd-sandbox";

/** A frame's trust level; trusted code never runs in a frame. */
export type Level = "strict" | "permissive";

/**
 * The colour scheme of the page around a frame, which the frame's document takes: a frame of another scheme than its
 * page's shows as an opaque box, and its components couldn't follow the page.
 */
export type Scheme = "light" | "dark";

/**
 * A component as a frame gets it, with its level: compiled text with its binding, a built module the host's page holds,
 * or a built module by its URL. One of a level above the frame's comes without code: the frame can't hold it, and its
 * uses show their fallbacks.
 */
export type ComponentCode = Runnable | Named;

/**
 * A component's name and level, which every component's code has, and `print: "fallback"` when its uses print their
 * fallback (spec §7.3, §14).
 */
export interface Named {
  name: string;
  level: Level | "trusted";
  /** The pack it comes from; absent for a document's own. A permissive frame runs only its own pack's permissive code. */
  pack?: string;
  print?: "fallback";
  /** The number of the example it is a component of (spec §7.8); absent for the document's own. */
  example?: number;
}

/**
 * Where a component is kept beside the others: its name, with its example's number for an example's, since an
 * example's `card` is not the document's.
 */
export const entryOf = ({ name, example }: Pick<Named, "name" | "example">): string => (example === undefined ? name : `${String(example)}:${name}`);

/**
 * A component with its code. A built module the host's page holds is a `Blob`, whose bytes the frames share, with its
 * place (`pack@version/path`), by which a frame knows it has it already. A module by its URL is a built module's plain
 * mount, or, with a `binding`, a pack's source component the host compiled ahead (a Svelte one's styles in its `css`
 * export). Code given as text or a `Blob` may import
 * library packs (docs/sharing-library-packs.md), which come with it in `libraries`, by the name it imports.
 */
export type Runnable = Named & (
  | { binding: "react" | "svelte" | "plain"; code: string; css?: string; libraries?: Libraries }
  | { binding: "plain"; code: Blob; place: string; libraries?: Libraries }
  | { module: { url: string; integrity: string }; binding?: "react" | "svelte" }
);

/** Library packs' modules by the name code imports them by. */
export type Libraries = Readonly<Record<string, LibraryCode>>;

/**
 * A library pack's module: its place (`pack@version/path`), by which a frame loads it once, its file as a `Blob` the
 * host's page has checked, and the libraries it imports in turn.
 */
export interface LibraryCode {
  place: string;
  code: Blob;
  libraries?: Libraries;
}

/** The binding a component compiled from `language` mounts through (spec §7.6). */
export function bindingOf(language: CompileRequest["language"]): "react" | "svelte" | "plain" {
  return language === "jsx" || language === "tsx" ? "react" : language === "svelte" ? "svelte" : "plain";
}

/** Whether `component` comes with code to run. */
export const hasCode = (component: ComponentCode): component is Runnable => "code" in component || "module" in component;

/**
 * The use a frame renders: its node, its slots as expanded nodes in it, and the document's inputs. A code block a code
 * renderer renders is its own node, so the frame has its `alt` and code, its fallback (spec §7.7).
 */
export interface Part {
  node: ExpandedUse;
  inputs: Record<string, ExpandedInput>;
  /** The number of the example the use is in (spec §7.8), whose components its node names; absent for the document. */
  example?: number;
}

/** A source to compile, for the compile frame. */
export interface CompileRequest {
  name: string;
  language: "jsx" | "tsx" | "svelte" | "js" | "ts";
  runtime?: string;
  code: string;
}

/** A file of a face of a document's own font, as a frame gets it: the face's name, its weight and style, and the file. */
export interface FrameFont {
  name: string;
  weight: "normal" | "bold";
  style: "normal" | "italic";
  file: Blob;
}

/** Messages from a parent to its frame, on the frame's port. `host` carries its port in the message's ports. */
export type ToFrame =
  /** `timeout`: how long the frame waits for an answer to its `highlight`, in ms (`HIGHLIGHT_MS` without it). */
  | { type: "level"; level: Level; policy?: string; schemes?: readonly string[]; packs?: string; timeout?: number }
  | { type: "code"; components: ComponentCode[] }
  | { type: "part"; part: Part }
  | { type: "styles"; urls: string[] }
  | { type: "scheme"; scheme: Scheme }
  /**
   * The use's box's type in the page (spec §7.4), which the frame's root takes, so what the component draws inherits it;
   * with the files of the document's own fonts its family names (spec §9.4), which the frame declares.
   */
  | { type: "type-style"; style: TypeStyle; fonts?: FrameFont[] }
  | { type: "host" }
  | { type: "ping"; n: number }
  | ({ type: "compile" } & CompileRequest)
  /** Record what passes in the frame, and report it, or stop (docs/devtools.md, decision 6). */
  | { type: "observe"; on: boolean }
  /**
   * The answer to the frame's `image`: the image's file, or none when the parent can't or won't give it, or `refused`
   * with its origin when the reader hasn't allowed that origin (spec §15).
   */
  | { type: "fetched"; url: string; image?: Blob; refused?: string }
  /** The reader allowed the origin `origin`: the frame's images refused for it ask again (spec §15). */
  | { type: "images-allowed"; origin: string }
  /**
   * The answer to the frame's `highlight` with the same `id`: the block's tokens, or none for a language the host's
   * highlighter doesn't cover, a block the parent didn't send, or a failure (docs/host-highlighter.md).
   */
  | { type: "highlighted"; id: number; lines?: CodeLines }
  /**
   * The use's box changed size, or the frame loaded, with the width of the box's inside: a use frame measures its
   * component's natural height at that width, and its natural width, and reports them. A browser pauses a frame's own
   * observers while it is off screen, and may not give it its new size then, never its messages.
   */
  | { type: "box"; width: number };

/** Why a component shows its fallback: `@compmd/render`'s reasons, code that didn't load, a level the frame can't hold. */
export type ErrorReason = "missing" | "threw" | "failed" | "not-here";

/** The diagnostics a sandbox reports itself, at rendering (spec §10.2). */
export type SandboxDiagnostic = "W118" | "W121";

/**
 * The diagnostic a component's fallback is, by why it shows: code that threw or stopped answering is W118, code that
 * didn't compile or load (an import it may not make, a module that doesn't match its file) W121. A component the
 * request didn't give, or one of a level its frame can't hold, is none: the processor reports the first, and the
 * second is the sandbox's design.
 */
export function diagnosticOf(reason: ErrorReason): SandboxDiagnostic | undefined {
  return reason === "threw" ? "W118" : reason === "failed" ? "W121" : undefined;
}

/** Messages from a frame to its parent. `set` has `@compmd/render`'s shape, so a page frame's comes as it is. */
export type FromFrame =
  | { type: "ready"; sandbox: number }
  | { type: "loaded" }
  | { type: "set"; input: string; value: unknown }
  /** `use` names the use that shows its fallback, which a later `recovered` names too; ids under a frame's own are prefixed by its parent. */
  | { type: "error"; reason: ErrorReason; component?: string; use?: string; message: string }
  /** A use that failed draws again, by the `use` its `error` named. */
  | { type: "recovered"; component: string; use: string }
  | { type: "link"; url: string }
  | { type: "rendered"; ms: number }
  /**
   * A use frame's component's natural size, which the parent gives the use's box (docs/sizing-components.md, *The
   * mechanism*); a page frame's content's size, which the parent sets on the frame.
   */
  | { type: "size"; width: number; height: number }
  | { type: "pong"; n: number }
  | { type: "compiled"; name: string; code: string; css?: string }
  | { type: "failed"; name: string; message: string }
  /** What the frame's observer recorded since its last report, once its parent asked: `Observed` events, unchecked. */
  | { type: "observed"; events: unknown[] }
  /** A page frame asks for an image of its page, by its `src`: it has no network. */
  | { type: "image"; url: string }
  /**
   * The reader asks, by the button beside a refused image, to see the images of `origin`: the parent asks them, with
   * its own question, and only about an origin of images it sent the frame (spec §15).
   */
  | { type: "allow-origin"; origin: string }
  /**
   * A frame asks for the tokens of a code block of its page or part, by its text and fence language, `id` its own
   * count: it loads no highlighter, and the host's page highlights for it, only a block of what it sent the frame.
   */
  | { type: "highlight"; id: number; code: string; language: string }
  /**
   * The frame has settled after the first `seen` messages that change what it shows (`CHANGES`): its uses drawn, its
   * own frames settled and sized, its images and fonts loaded, then a frame with nothing moving.
   */
  | { type: "settled"; seen: number };

/** Every message type, each way: what the schema (`protocol.schema.json`) has too. */
export const TO_FRAME: readonly ToFrame["type"][] = ["level", "code", "part", "styles", "scheme", "type-style", "host", "ping", "compile", "observe", "fetched", "images-allowed", "highlighted", "box"];
export const FROM_FRAME: readonly FromFrame["type"][] = ["ready", "loaded", "set", "error", "recovered", "link", "rendered", "size", "pong", "compiled", "failed", "observed", "image", "allow-origin", "highlight", "settled"];

/** How long a frame waits for its `highlight` to be answered, in ms, when its level doesn't say: the block keeps its text. */
export const HIGHLIGHT_MS = 5000;

/**
 * The messages that change what a frame shows, the sandbox's and `@compmd/render`'s: a parent counts those it sends, and
 * a frame those it takes, in the order the port keeps, so `settled` says for which it settled.
 */
export const CHANGES: readonly string[] = ["code", "part", "styles", "scheme", "type-style", "render", "splices", "theme"];

/** Whether `data` is a message that changes what a frame shows. */
export const isChange = (data: unknown): boolean => typeof data === "object" && data !== null && CHANGES.includes(String((data as { type?: unknown }).type));

/** What the protocol needs of a port: a `MessagePort` is one. It is `@compmd/render`'s, with transfers and ports. */
export interface Port {
  postMessage(message: unknown, transfer?: Transferable[]): void;
  addEventListener(type: "message", listener: (event: { data: unknown; ports?: readonly MessagePort[] }) => void): void;
  removeEventListener(type: "message", listener: (event: { data: unknown; ports?: readonly MessagePort[] }) => void): void;
  start?(): void;
}

/** Whether `data` is a message of one of `types`, by its `type`. */
export function isMessage<T extends { type: string }>(data: unknown, types: readonly T["type"][]): data is T {
  return typeof data === "object" && data !== null && types.includes((data as { type?: unknown }).type as T["type"]);
}
