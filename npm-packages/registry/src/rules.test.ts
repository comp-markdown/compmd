// The registry rules' shared cases (`crates/host/fixtures/registry/`, docs/registry-rules.md, decision 7), through the
// WebAssembly build `@compmd/registry` resolves with, each case's registry served as an npm registry, as
// `compmd-host`'s own tests serve it.

import { readdirSync, readFileSync } from "node:fs";
import { answeredVersions, loaded, offOrigin, Resolution, type Answer, type Needed, type Options, type Resolved, type Step } from "@compmd/host/registry";
import { beforeAll, describe, expect, it } from "vitest";
import { archive } from "../test/tar-archive.ts";

const NPM = "https://registry.npmjs.org/";
const FOLDER = new URL("../../../crates/host/fixtures/registry/", import.meta.url);

interface Case {
  name: string;
  imports: Needed[];
  brings?: Record<string, Needed[]>;
  registry: { pack: string; version: string; files: Record<string, string>; dist?: Record<string, unknown>; corrupt?: boolean }[];
  options?: Omit<Options, "registries">;
  resolved: unknown;
}

/** The gzip of `bytes`, as a registry serves a tarball. */
async function gzip(bytes: Uint8Array<ArrayBuffer>): Promise<Uint8Array> {
  return new Uint8Array(await new Response(new Blob([bytes]).stream().pipeThrough(new CompressionStream("gzip"))).arrayBuffer());
}

/** The tarball of `files`, as npm packs a package: each under `package/`. */
const tarball = (files: Record<string, string>) => gzip(archive(...Object.entries(files).map(([path, text]): [string, string] => [`package/${path}`, text])));

/** A tarball's integrity, as npm publishes it. */
async function integrityOf(bytes: Uint8Array): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-512", new Uint8Array(bytes)));
  return `sha512-${btoa(String.fromCharCode(...digest))}`;
}

/** The host of `testCase`: its registry's metadata and tarballs, and the imports the processor would list. */
class Host {
  readonly metadata = new Map<string, string>();
  readonly tarballs = new Map<string, Uint8Array>();
  readonly downloaded: string[] = [];
  readonly #have = new Set<string>();
  readonly testCase: Case;

  constructor(testCase: Case) {
    this.testCase = testCase;
  }

  async serve(): Promise<void> {
    const byPack = new Map<string, Record<string, unknown>>();
    for (const release of this.testCase.registry) {
      const bytes = await tarball(release.files);
      const url = `${NPM}${release.pack}/-/${release.version}.tgz`;
      const size = Object.values(release.files).reduce((total, text) => total + new TextEncoder().encode(text).length, 0);
      const written: Record<string, unknown> = { tarball: url, integrity: await integrityOf(bytes), unpackedSize: size, ...release.dist };
      // A key the case sets to `null` is one the metadata doesn't have.
      const dist = Object.fromEntries(Object.entries(written).filter(([, value]) => value !== null));
      this.tarballs.set(url, release.corrupt ? await tarball({ "index.comp.md": "# Changed" }) : bytes);
      byPack.set(release.pack, { ...byPack.get(release.pack), [release.version]: { dist } });
    }
    for (const [pack, versions] of byPack) this.metadata.set(`${NPM}${pack.replace("/", "%2f")}`, JSON.stringify({ name: pack, versions }));
  }

  /** The answer to `step`, a tarball's given to `resolution` itself. */
  perform(step: Step, resolution: Resolution): Answer | undefined {
    switch (step.kind) {
      case "missing": {
        for (const [pack] of step.releases) this.#have.add(pack);
        const brought = [...this.#have].flatMap((pack) => this.testCase.brings?.[pack] ?? []);
        const refused = new Set(Object.keys(step.refused));
        return { kind: "missing", packs: [...this.testCase.imports, ...brought].filter(({ pack }) => !this.#have.has(pack) && !refused.has(pack)) };
      }
      case "keptVersions":
        return { kind: "keptVersions", kept: null };
      case "versions": {
        const body = this.metadata.get(`${step.registry}${step.pack.replace("/", "%2f")}`);
        const versions = body === undefined ? answeredVersions(step.registry, step.pack, 404, "") : answeredVersions(step.registry, step.pack, 200, body);
        return { kind: "versions", versions, maxAge: 300 };
      }
      case "kept":
        return { kind: "kept", files: null };
      case "tarball": {
        const elsewhere = offOrigin(step.version.tarball, step.registry);
        if (elsewhere !== undefined) return { kind: "failed", message: elsewhere };
        const bytes = this.tarballs.get(step.version.tarball);
        if (!bytes) return { kind: "failed", message: `${step.version.tarball} answered 404` };
        this.downloaded.push(step.version.tarball.slice(NPM.length).replace("/-/", "@").replace(/\.tgz$/, ""));
        resolution.giveTarball(step.id, bytes);
        return undefined;
      }
      case "keepVersions":
      case "keep":
        return { kind: "done" };
    }
  }

  /** Drives a resolution with `options` to its end, each round's steps in order. */
  resolve(options: Options): Resolved {
    const resolution = new Resolution(options);
    for (;;) {
      const next = resolution.next();
      if ("done" in next) return next.done;
      for (const step of next.steps) {
        const answer = this.perform(step, resolution);
        if (answer) resolution.give(step.id, answer);
      }
    }
  }
}

const cases: [string, Case][] = readdirSync(FOLDER)
  .filter((file) => file.endsWith(".json"))
  .sort()
  .map((file) => [file, JSON.parse(readFileSync(new URL(file, FOLDER), "utf8")) as Case]);

beforeAll(() => loaded());

describe("the registry rules' shared cases, as WebAssembly", () => {
  it("are read", () => {
    expect(cases.length).toBeGreaterThanOrEqual(4);
  });

  it.each(cases)("%s holds", async (_, testCase) => {
    const host = new Host(testCase);
    await host.serve();
    const resolved = host.resolve({ registries: { "*": NPM }, ...testCase.options });
    const releases = Object.fromEntries(
      Object.entries(resolved.releases).map(([pack, releases]) => [pack, releases.map(({ version, files }) => ({ version, files: Object.keys(files).sort() }))]),
    );
    const broken = Object.entries(resolved.releases).flatMap(([pack, releases]) => releases.filter(({ broken }) => broken !== undefined).map(({ version }) => `${pack}@${version}`));
    expect({ releases, broken, refused: resolved.refused, problems: resolved.problems, downloaded: host.downloaded }).toEqual(testCase.resolved);
  });
});
