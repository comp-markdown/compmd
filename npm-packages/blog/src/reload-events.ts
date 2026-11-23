// How `cblo dev` tells its open pages that the blog changed: server-sent events (docs/blog-tool.md, decision 3).

import type { ServerResponse } from "node:http";

/** The stream's path on `cblo dev`'s server, outside every blog's own path. */
export const RELOAD_PATH = "/_cblo/reload";

/** What a page `cblo dev` serves runs: it reloads on the stream's event, and the browser reconnects by itself. */
const RELOAD_SCRIPT = `<script>new EventSource(${JSON.stringify(RELOAD_PATH)}).onmessage = () => location.reload();</script>`;

/** `html` with the script that reloads it when the blog changes, before its `</body>`, or at its end without one. */
export function withReload(html: string): string {
  const end = html.lastIndexOf("</body>");
  return end === -1 ? `${html}${RELOAD_SCRIPT}` : `${html.slice(0, end)}${RELOAD_SCRIPT}${html.slice(end)}`;
}

/** The open pages' streams: each page `cblo dev` serves opens one, and every one is told when the blog changes. */
export class ReloadEvents {
  readonly #streams = new Set<ServerResponse>();

  /** Makes `response` a page's stream, until the page closes it. */
  open(response: ServerResponse): void {
    response.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache" });
    response.write(": open\n\n");
    this.#streams.add(response);
    response.on("close", () => this.#streams.delete(response));
  }

  /** Tells every open page to reload. */
  reload(): void {
    for (const stream of this.#streams) stream.write("data: reload\n\n");
  }

  /** Ends every stream, for a server that stops. */
  close(): void {
    for (const stream of this.#streams) stream.end();
    this.#streams.clear();
  }
}
