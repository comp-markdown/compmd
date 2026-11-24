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
    const request = requestOf(incoming);
    // One that isn't a request a fetch handler can be given (a `Host` that isn't one, `TRACE`) is the reader's mistake,
    // and the server goes on (audit S23).
    if (!request) {
      outgoing.writeHead(400, { "content-type": "text/plain; charset=utf-8" }).end("Bad request");
      return;
    }
    void handler(request).then(
      (response) => {
        outgoing.writeHead(response.status, Object.fromEntries(response.headers));
        if (!response.body) {
          outgoing.end();
          return;
        }
        // A body that fails midway ends the response, never the process.
        const body = Readable.fromWeb(response.body as NodeReadableStream);
        body.on("error", () => outgoing.destroy());
        body.pipe(outgoing);
      },
      () => {
        outgoing.writeHead(500).end();
      },
    );
  };
}

/**
 * The fetch `Request` for `incoming`: only its method and its URL, as the proxy passes on nothing else the reader sent,
 * cookies least of all; `undefined` when they don't make one.
 */
function requestOf(incoming: IncomingMessage): Request | undefined {
  try {
    const url = new URL(incoming.url ?? "/", `http://${incoming.headers.host ?? "localhost"}`);
    return new Request(url, { method: incoming.method ?? "GET" });
  } catch {
    return undefined;
  }
}
