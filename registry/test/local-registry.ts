// A registry for the tests that speaks npm's protocol: each pack's short metadata, and its tarballs, from the fixtures
// `pnpm pack` wrote (test/fixtures/tarballs). It records every request, so a test can tell what was fetched.

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { join } from "node:path";

/** A version as the registry serves it. */
export interface Served {
  /** The fixture tarball, `kit-charts-1.0.0.tgz`. */
  file: string;
  /** The unpacked size its metadata states; `null` for none (default 2000). */
  unpackedSize?: number | null;
  /** Whether its bytes are changed after its integrity was published. */
  corrupt?: boolean;
}

/** Packs by name, each version by its number. */
export type Catalog = Record<string, Record<string, Served>>;

export interface LocalRegistry {
  /** Its URL, ending in `/`. */
  url: string;
  /** Every request's path, in order. */
  hits: string[];
  /** Makes it answer 500 to everything, or not. */
  failing(failing: boolean): void;
  close(): Promise<void>;
}

const TARBALLS = join(import.meta.dirname, "fixtures/tarballs");

/** A fixture tarball's bytes. */
export const tarballOf = (file: string): Buffer => readFileSync(join(TARBALLS, file));

/** npm's integrity of `bytes`. */
export const integrityOf = (bytes: Buffer): string => `sha512-${createHash("sha512").update(bytes).digest("base64")}`;

/** Every pack the tests use. */
export const CATALOG: Catalog = {
  "@kit/report": { "1.0.0": { file: "kit-report-1.0.0.tgz" } },
  "@kit/charts": { "1.0.0": { file: "kit-charts-1.0.0.tgz" }, "1.1.0": { file: "kit-charts-1.1.0.tgz" } },
  "@kit/huge": { "1.0.0": { file: "kit-charts-1.0.0.tgz", unpackedSize: 10_000_000 } },
  "@kit/unsized": { "1.0.0": { file: "kit-charts-1.0.0.tgz", unpackedSize: null } },
  "@kit/broken": { "1.0.0": { file: "kit-report-1.0.0.tgz", corrupt: true } },
  "@kit/bars": { "1.0.0": { file: "kit-bars-1.0.0.tgz" } },
  "@kit/lines": { "1.0.0": { file: "kit-lines-1.0.0.tgz" } },
  "@kit/dashboard": { "1.0.0": { file: "kit-dashboard-1.0.0.tgz" } },
  "@kit/d3": { "7.9.5": { file: "kit-d3-7.9.5.tgz" }, "7.12.0": { file: "kit-d3-7.12.0.tgz" } },
};

/** A registry serving `catalog` on a free port of 127.0.0.1. */
export async function localRegistry(catalog: Catalog = CATALOG): Promise<LocalRegistry> {
  const hits: string[] = [];
  let failing = false;
  let url = "";
  const server = createServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    hits.push(path);
    const answer = (status: number, body: string | Buffer, headers: Record<string, string> = {}) =>
      response.writeHead(status, { "access-control-allow-origin": "*", ...headers }).end(body);
    if (failing) return answer(500, "failing");
    const tarball = /^\/tarballs\/(@[^/]+\/[^/]+)\/([^/]+)$/.exec(path);
    if (tarball) {
      const served = catalog[tarball[1] ?? ""]?.[tarball[2] ?? ""];
      if (!served) return answer(404, "not found");
      const bytes = Buffer.from(tarballOf(served.file));
      if (served.corrupt) bytes[bytes.length - 1] = (bytes.at(-1) ?? 0) ^ 0xff;
      return answer(200, bytes, { "content-type": "application/octet-stream" });
    }
    const versions = catalog[path.slice(1)];
    if (!versions) return answer(404, JSON.stringify({ error: "Not found" }), { "content-type": "application/json" });
    const listed = Object.fromEntries(
      Object.entries(versions).map(([version, served]) => [
        version,
        {
          name: path.slice(1),
          version,
          dist: {
            tarball: `${url}tarballs/${path.slice(1)}/${version}`,
            integrity: integrityOf(tarballOf(served.file)),
            ...(served.unpackedSize === null ? {} : { unpackedSize: served.unpackedSize ?? 2000 }),
          },
        },
      ]),
    );
    answer(200, JSON.stringify({ name: path.slice(1), versions: listed }), {
      "content-type": "application/vnd.npm.install-v1+json",
      "cache-control": "public, max-age=300",
    });
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  url = `http://127.0.0.1:${String((server.address() as AddressInfo).port)}/`;
  return {
    url,
    hits,
    failing: (value) => {
      failing = value;
    },
    close: () =>
      new Promise((resolve) => {
        server.close(() => {
          resolve();
        });
        server.closeAllConnections();
      }),
  };
}
