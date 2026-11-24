// Answering a request for a site's file from Node (audit S23, S29, A11): `cblo dev`'s server and the website's dev
// server read a request's path the same way, never throwing on one that can't be read, answer only this machine when
// they serve drafts, and send each file with its type and `nosniff`.

import type { IncomingMessage, ServerResponse } from "node:http";
import { extname } from "node:path";

/** A file's type by its extension; any other is bytes. */
const TYPES: Readonly<Record<string, string>> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
  ".csv": "text/csv; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".wasm": "application/wasm",
  ".woff2": "font/woff2",
  ".pdf": "application/pdf",
};

/** The type a file is sent with, by its extension. */
export const contentTypeOf = (path: string): string => (Object.hasOwn(TYPES, extname(path)) ? (TYPES[extname(path)] ?? "") : "application/octet-stream");

/**
 * The path `request` asks for, decoded, without its query; `undefined` for one that can't be read: a URL that isn't
 * one (`//`), or an escape that isn't UTF-8 (`%zz`, `%ff`).
 */
export function requestedPath(request: IncomingMessage): string | undefined {
  try {
    return decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
  } catch {
    return undefined;
  }
}

/** The names this machine answers to, which a page of another site can't make a browser send (DNS rebinding). */
const THIS_MACHINE = new Set(["localhost", "127.0.0.1", "[::1]"]);

/** Whether `request` names this machine in its `Host`, as a browser on it does. */
export function isFromThisMachine(request: IncomingMessage): boolean {
  const host = request.headers.host;
  if (host === undefined) return false;
  try {
    return THIS_MACHINE.has(new URL(`http://${host}`).hostname);
  } catch {
    return false;
  }
}

/** Sends `content` as the file `path`, with its type, `nosniff` and `headers`. */
export function sendFile(response: ServerResponse, path: string, content: string | Uint8Array, headers: Readonly<Record<string, string>> = {}): void {
  response.writeHead(200, { "content-type": contentTypeOf(path), "x-content-type-options": "nosniff", ...headers });
  response.end(content);
}

/** Answers `400`, saying why. */
export function sendBadRequest(response: ServerResponse, why: string): void {
  response.writeHead(400, { "content-type": "text/plain; charset=utf-8", "x-content-type-options": "nosniff" });
  response.end(why);
}
