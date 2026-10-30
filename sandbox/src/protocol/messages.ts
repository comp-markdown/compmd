// The sandbox's frame protocol (docs/sandbox-protocol.md): the messages between a frame and its parent, the window
// that made it. They travel on the port `@compmd/render`'s frame protocol uses too, beside its own.

import type { ExpandedComponent, ExpandedInput } from "@compmd/render";

/** The protocol's version, in `hello` and `ready`: a frame of another version gets nothing. */
export const SANDBOX_VERSION = 1;

/** What the parent first posts to the frame's window, with the frame's port. */
export const HELLO = "compmd-sandbox";

/** A frame's trust level; trusted code never runs in a frame. */
export type Level = "strict" | "permissive";

/** A component's code, as a frame loads it: compiled text with its binding, or a built module by its URL. */
export type ComponentCode =
  | { name: string; binding: "react" | "svelte" | "plain"; code: string; css?: string }
  | { name: string; module: { url: string; integrity: string } };

/** The use a frame renders: its node, its slots as expanded nodes in it, and the document's inputs. */
export interface Part {
  node: ExpandedComponent;
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
  | { type: "host" }
  | { type: "ping"; n: number }
  | ({ type: "compile" } & CompileRequest);

/** Why a component shows its fallback: `@compmd/render`'s reasons, code that didn't load, a level the frame can't hold. */
export type ErrorReason = "missing" | "threw" | "failed" | "not-here";

/** Messages from a frame to its parent. `set` has `@compmd/render`'s shape, so a page frame's comes as it is. */
export type FromFrame =
  | { type: "ready"; sandbox: number }
  | { type: "loaded" }
  | { type: "set"; input: string; value: unknown }
  | { type: "error"; reason: ErrorReason; component?: string; message: string }
  | { type: "link"; url: string }
  | { type: "rendered"; ms: number }
  | { type: "size"; width: number; height: number }
  | { type: "pong"; n: number }
  | { type: "compiled"; name: string; code: string; css?: string }
  | { type: "failed"; name: string; message: string };

/** Every message type, each way: what the schema (`protocol.schema.json`) has too. */
export const TO_FRAME: readonly ToFrame["type"][] = ["level", "code", "part", "styles", "host", "ping", "compile"];
export const FROM_FRAME: readonly FromFrame["type"][] = ["ready", "loaded", "set", "error", "link", "rendered", "size", "pong", "compiled", "failed"];

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
