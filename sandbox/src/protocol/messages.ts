// The sandbox's frame protocol (docs/sandbox-protocol.md): the messages between a frame and its parent, the window
// that made it. They travel on the port `@compmd/render`'s frame protocol uses too, beside its own.

import type { ExpandedInput, ExpandedUse } from "@compmd/render";

/** The protocol's version, in `hello` and `ready`: a frame of another version gets nothing. */
export const SANDBOX_VERSION = 1;

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
export type ComponentCode = Runnable | { name: string; level: Level | "trusted" };

/**
 * A component with its code. A built module the host's page holds is a `Blob`, whose bytes the frames share, with its
 * place (`pack@version/path`), by which a frame knows it has it already. A module by its URL is a built module's plain
 * mount, or, with a `binding`, a pack's source component the host compiled ahead (a Svelte one's styles in its `css`
 * export). Code given as text or a `Blob` may import
 * library packs (docs/sharing-library-packs.md), which come with it in `libraries`, by the name it imports.
 */
export type Runnable = { name: string; level: Level | "trusted" } & (
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
}

/** A source to compile, for the compile frame. */
export interface CompileRequest {
  name: string;
  language: "jsx" | "tsx" | "svelte" | "js" | "ts";
  runtime?: string;
  code: string;
}

/** Messages from a parent to its frame, on the frame's port. `host` carries its port in the message's ports. */
export type ToFrame =
  | { type: "level"; level: Level; policy?: string }
  | { type: "code"; components: ComponentCode[] }
  | { type: "part"; part: Part }
  | { type: "styles"; urls: string[] }
  | { type: "scheme"; scheme: Scheme }
  | { type: "host" }
  | { type: "ping"; n: number }
  | ({ type: "compile" } & CompileRequest)
  /** Record what passes in the frame, and report it, or stop (docs/devtools.md, decision 6). */
  | { type: "observe"; on: boolean }
  /** The answer to the frame's `image`: the image's file, or none when the parent can't or won't give it. */
  | { type: "fetched"; url: string; image?: Blob };

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
  | { type: "size"; width: number; height: number }
  | { type: "pong"; n: number }
  | { type: "compiled"; name: string; code: string; css?: string }
  | { type: "failed"; name: string; message: string }
  /** What the frame's observer recorded since its last report, once its parent asked: `Observed` events, unchecked. */
  | { type: "observed"; events: unknown[] }
  /** A page frame asks for an image of its page, by its `src`: it has no network. */
  | { type: "image"; url: string }
  /**
   * The frame has settled after the first `seen` messages that change what it shows (`CHANGES`): its uses drawn, its
   * own frames settled and sized, its images and fonts loaded, then a frame with nothing moving.
   */
  | { type: "settled"; seen: number };

/** Every message type, each way: what the schema (`protocol.schema.json`) has too. */
export const TO_FRAME: readonly ToFrame["type"][] = ["level", "code", "part", "styles", "scheme", "host", "ping", "compile", "observe", "fetched"];
export const FROM_FRAME: readonly FromFrame["type"][] = ["ready", "loaded", "set", "error", "recovered", "link", "rendered", "size", "pong", "compiled", "failed", "observed", "image", "settled"];

/**
 * The messages that change what a frame shows, the sandbox's and `@compmd/render`'s: a parent counts those it sends, and
 * a frame those it takes, in the order the port keeps, so `settled` says for which it settled.
 */
export const CHANGES: readonly string[] = ["code", "part", "styles", "scheme", "render", "splices", "theme"];

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
