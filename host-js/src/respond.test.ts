import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { contentType } from "./content-type.ts";
import { folderBundle } from "./node.ts";
import { respond, type Store } from "./respond.ts";

/** The serving contract's cases, which compmd-host runs too. */
const FIXTURES = join(import.meta.dirname, "../../host/fixtures/respond");

interface Cases {
  store: Record<string, string>;
  headers: Record<string, Record<string, string | null>>;
  cases: { method?: string; path: string; store?: boolean; status: number; answer: string; type?: string; body: string }[];
}

const cases = JSON.parse(readFileSync(join(FIXTURES, "cases.json"), "utf8")) as Cases;

describe("respond", () => {
  it("gives every case of the contract its answer, as compmd-host does", async () => {
    const bundle = await folderBundle(join(FIXTURES, "bundle"));
    const store: Store = {
      file: ({ release, path }) => {
        const body = cases.store[`${release}/${path}`];
        return Promise.resolve(body === undefined ? undefined : new TextEncoder().encode(body));
      },
    };
    for (const each of cases.cases) {
      const request = new Request(`https://compmd.example${each.path}`, { method: each.method ?? "GET" });
      const response = await respond(request, bundle, each.store ? store : undefined);
      const label = `${request.method} ${each.path}`;
      expect(response.status, label).toBe(each.status);
      expect(await response.text(), label).toBe(each.body);
      if (each.type !== undefined) expect(response.headers.get("content-type"), label).toBe(each.type);
      for (const [name, value] of Object.entries(cases.headers[each.answer] ?? {})) expect(response.headers.get(name), `${label}: ${name}`).toBe(value);
    }
  });
});

describe("contentType", () => {
  it("gives each kind of file its type, by extension", () => {
    expect(contentType("a.js.map")).toBe("application/json");
    expect(contentType("fonts/a.woff2")).toBe("font/woff2");
    expect(contentType("chunks/.js")).toBe("application/octet-stream");
    expect(contentType("a")).toBe("application/octet-stream");
  });
});

describe("folderBundle", () => {
  it("refuses a folder without a manifest, or with one without the frames' hash", async () => {
    const folder = mkdtempSync(join(tmpdir(), "compmd-host-"));
    await expect(folderBundle(folder)).rejects.toThrow();
    writeFileSync(join(folder, "manifest.json"), JSON.stringify({ frames: {} }));
    await expect(folderBundle(folder)).rejects.toThrow("without the frames' hash");
  });
});
