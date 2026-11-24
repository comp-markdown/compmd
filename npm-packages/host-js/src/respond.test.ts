import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { folderBundle } from "./node.ts";
import { respond, type Store } from "./respond.ts";

/** The serving contract's cases, which compmd-host runs too. */
const FIXTURES = join(import.meta.dirname, "../../../crates/host/fixtures/respond");

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
      file: ({ pack, version, path }) => {
        const body = cases.store[`${pack}@${version}/${path}`];
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

describe("folderBundle", () => {
  it("refuses a folder without a manifest, or with one compmd-host can't use, saying why", async () => {
    const folder = mkdtempSync(join(tmpdir(), "compmd-host-"));
    await expect(folderBundle(folder)).rejects.toThrow();
    for (const manifest of [{ format: 1, frames: {} }, { format: 2, frames: { hash: "1df336e75685de8c" } }, { format: 1, frames: { hash: "../../etc" } }]) {
      writeFileSync(join(folder, "manifest.json"), JSON.stringify(manifest));
      await expect(folderBundle(folder), JSON.stringify(manifest)).rejects.toThrow(`the bundle in ${folder} can't be served: `);
    }
  });
});
