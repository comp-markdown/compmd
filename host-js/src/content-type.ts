// A served file's type, by its extension: what a bundle's files are. The same table as compmd-host's.

const TYPES: Readonly<Record<string, string>> = {
  html: "text/html; charset=utf-8",
  js: "text/javascript; charset=utf-8",
  mjs: "text/javascript; charset=utf-8",
  css: "text/css; charset=utf-8",
  json: "application/json",
  map: "application/json",
  wasm: "application/wasm",
  svg: "image/svg+xml",
  png: "image/png",
  woff2: "font/woff2",
  woff: "font/woff",
  ttf: "font/ttf",
};

/**
 * The type of the file at `path`, by its extension; `application/octet-stream` for any other.
 *
 * @example
 * contentType("dist/mermaid.js"); // "text/javascript; charset=utf-8"
 */
export function contentType(path: string): string {
  const name = path.slice(path.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return (dot > 0 ? TYPES[name.slice(dot + 1)] : undefined) ?? "application/octet-stream";
}
