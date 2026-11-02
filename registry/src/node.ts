// A fetch handler, such as the proxy, served by Node's `http` (docs/resolving-registry-packs.md, decision 9): for
// `http.createServer`, and Vite's dev server.

import type { IncomingMessage, ServerResponse } from "node:http";
import { Readable } from "node:stream";
import type { ReadableStream as NodeReadableStream } from "node:stream/web";

/**
 * A Node request listener that answers with `handler`.
 *
 * @example
 * import { createServer } from "node:http";
 * createServer(toNodeListener(registryProxy({ registries, base: "/compmd/registry/" }))).listen(8080);
 */
export function toNodeListener(handler: (request: Request) => Promise<Response>): (request: IncomingMessage, response: ServerResponse) => void {
  return (incoming, outgoing) => {
    // Only the method and the URL: the proxy passes on nothing else the reader sent, cookies least of all.
    const url = new URL(incoming.url ?? "/", `http://${incoming.headers.host ?? "localhost"}`);
    void handler(new Request(url, { method: incoming.method ?? "GET" })).then(
      (response) => {
        outgoing.writeHead(response.status, Object.fromEntries(response.headers));
        if (response.body) Readable.fromWeb(response.body as NodeReadableStream).pipe(outgoing);
        else outgoing.end();
      },
      () => {
        outgoing.writeHead(500).end();
      },
    );
  };
}
