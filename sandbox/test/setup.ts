// The browser tests' servers, in Node before the tests: the frame documents as a host serves them, from an origin of
// their own, and two probes that record every request made to them, one a permissive pack may reach and one it may
// not. A test asks a probe what reached it.

import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, normalize } from "node:path";
import type { TestProject } from "vitest/node";
import { buildFrames } from "../src/build/frames.ts";

declare module "vitest" {
  interface ProvidedContext {
    /** Where the frame documents are served, ending in `/`. */
    frames: string;
    /** A probe a permissive pack may be given. */
    allowed: string;
    /** A probe no frame may reach. */
    denied: string;
  }
}

const ROOT = join(import.meta.dirname, "..");
const OUT = join(ROOT, "test/.frames");
const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css" };

/** Every request a server got: its path with its query. */
type Hits = string[];

function listen(server: Server): Promise<string> {
  return new Promise((resolve) => {
    server.listen(0, "127.0.0.1", () => {
      resolve(`http://127.0.0.1:${String((server.address() as AddressInfo).port)}/`);
    });
  });
}

/** Answers `/hits` with what it got, and `/reset` by forgetting it; records everything else. */
function recording(hits: Hits, respond: (path: string, answer: (status: number, body?: string | Buffer, type?: string) => void) => void): Server {
  return createServer((request, response) => {
    const url = new URL(request.url ?? "/", "http://localhost");
    const answer = (status: number, body?: string | Buffer, type = "text/plain") =>
      response.writeHead(status, { "access-control-allow-origin": "*", "content-type": type }).end(body);
    if (url.pathname === "/hits") return answer(200, JSON.stringify(hits), "application/json");
    if (url.pathname === "/reset") {
      hits.length = 0;
      return answer(200);
    }
    hits.push(url.pathname + url.search);
    respond(normalize(decodeURIComponent(url.pathname)), answer);
  });
}

export default async function setup(project: TestProject): Promise<() => Promise<void>> {
  const frameHits: Hits = [];
  // The frame documents and their files; the frames have an opaque origin, so everything they load is cross-origin.
  const frames = recording(frameHits, (path, answer) => {
    readFile(join(OUT, path)).then(
      (body) => {
        answer(200, body, TYPES[extname(path)] ?? "text/plain");
      },
      () => {
        answer(404);
      },
    );
  });
  const probed = (_path: string, answer: (status: number, body?: string) => void) => {
    answer(200, "probed");
  };
  const probes = [recording([], probed), recording([], probed)];
  const base = await listen(frames);
  const [allowed, denied] = await Promise.all(probes.map(listen));
  await rm(OUT, { recursive: true, force: true });
  await buildFrames({ out: OUT, base, from: join(ROOT, "dist/frames") });
  await mkdir(join(OUT, "packs"), { recursive: true });
  await writeFile(join(OUT, "packs/hello.js"), "export default function mount(target) { target.textContent = 'built module'; }\n");
  project.provide("frames", base);
  project.provide("allowed", allowed ?? "");
  project.provide("denied", denied ?? "");
  return async () => {
    await Promise.all(
      [frames, ...probes].map(
        (server) =>
          new Promise((resolve) => {
            server.close(resolve);
            server.closeAllConnections();
          }),
      ),
    );
  };
}
