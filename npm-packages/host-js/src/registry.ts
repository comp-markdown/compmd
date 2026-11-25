// The registry rules' resolution as `compmd-host` decides it (docs/registry-rules.md), for `@compmd/registry`: its
// steps and answers typed, a tarball's bytes given as bytes. `loaded()` loads the module first: from its file in Node,
// with `init()` from beside this file in a page or a worker.

import init, {
  answeredVersions as answeredVersionsJson,
  initSync,
  offOrigin as offOriginOf,
  Resolution as Rules,
} from "../pkg/compmd_host_wasm.js";

/** Node's `process`, as far as it gives its built-in modules; absent in a page or a worker. */
type NodeProcess = { getBuiltinModule?: (id: "node:fs") => typeof import("node:fs") };

let loading: Promise<void> | undefined;

/** Loads the module, once: in Node from its file, elsewhere by `init()`, which fetches it from beside this file. */
export function loaded(): Promise<void> {
  loading ??= (async () => {
    const fs = (globalThis as { process?: NodeProcess }).process?.getBuiltinModule?.("node:fs");
    if (fs) initSync({ module: fs.readFileSync(new URL("../pkg/compmd_host_wasm_bg.wasm", import.meta.url)) });
    else await init();
  })();
  return loading;
}

/** A version of a pack, as a registry lists it. */
export interface VersionInfo {
  version: string;
  /** Where its tarball is. */
  tarball: string;
  /** The integrity the registry publishes, `sha512-…`. */
  integrity?: string;
  /** Its unpacked size in bytes. */
  unpackedSize?: number;
}

/** A release as a cache keeps it. */
export interface KeptRelease {
  pack: string;
  version: string;
  integrity: string;
}

/** A release's checked files: its text, and the rest as bytes. */
export interface Files {
  text: Record<string, string>;
  binary: Record<string, number[]>;
}

/** A pack the request imports that it has no release of, as the processor's `sources` lists it. */
export interface Needed {
  pack: string;
  range: string;
  integrity?: string;
  library?: boolean;
}

/** A release for the request's packs, as the processor takes it. */
export interface Release {
  version: string;
  integrity: string | null;
  files: Record<string, string>;
  broken?: string;
}

/** What the resolution asks the host to do (the registry rules, decision 2). */
export type Step =
  | { kind: "missing"; id: number; releases: [string, Release][]; refused: Record<string, string> }
  | { kind: "keptVersions"; id: number; registry: string; pack: string }
  | { kind: "versions"; id: number; registry: string; pack: string }
  | { kind: "keepVersions"; id: number; registry: string; pack: string; versions: VersionInfo[]; maxAge: number }
  | { kind: "kept"; id: number; release: KeptRelease }
  | { kind: "tarball"; id: number; registry: string; version: VersionInfo; maxBytes: number }
  | { kind: "keep"; id: number; release: KeptRelease; files: Files };

/** What the host did; a tarball's bytes go by `giveTarball`. */
export type Answer =
  | { kind: "missing"; packs: Needed[] }
  | { kind: "keptVersions"; kept: { versions: VersionInfo[]; fresh: boolean } | null }
  | { kind: "versions"; versions: VersionInfo[]; maxAge: number }
  | { kind: "kept"; files: Files | null }
  | { kind: "failed"; message: string }
  | { kind: "done" };

/** What a resolution gave. */
export interface Resolved {
  releases: Record<string, Release[]>;
  refused: Record<string, string>;
  /** Each built module's checked text, by its place. */
  modules: Record<string, string>;
  /** Each image and font's file, by its place. */
  files: Record<string, { mediaType: string; bytes: number[] }>;
  problems: { pack: string; message: string }[];
}

/** A host's settings for a resolution. */
export interface Options {
  /** The registries by scope, `@kit`, with `*` for every other scope, each a URL ending in `/`. */
  registries: Record<string, string>;
  maxPackSize?: number;
  deny?: { pack: string; version?: string }[];
}

/** One resolution of a request's packs: it says what it needs, the host does it and gives the answer. */
export class Resolution {
  readonly #rules: Rules;

  /** Load the module first, with `loaded()`. */
  constructor(options: Options) {
    this.#rules = new Rules(JSON.stringify(options));
  }

  /** The steps to do now, or what the resolution gave. */
  next(): { steps: Step[] } | { done: Resolved } {
    return JSON.parse(this.#rules.nextSteps()) as { steps: Step[] } | { done: Resolved };
  }

  /** The answer to the step `id`. */
  give(id: number, answer: Answer): void {
    this.#rules.give(id, JSON.stringify(answer));
  }

  /** A tarball's bytes, the answer to the step `id`. */
  giveTarball(id: number, bytes: Uint8Array): void {
    this.#rules.giveTarball(id, bytes);
  }

  /** Gives up on `pack`, which hasn't come within `milliseconds`. */
  giveUp(pack: string, milliseconds: number): void {
    this.#rules.giveUp(pack, milliseconds);
  }
}

/** The versions of `pack` an npm registry at `registry` answered with `status` and `body`; throws why there are none. */
export function answeredVersions(registry: string, pack: string, status: number, body: string): VersionInfo[] {
  const answered = JSON.parse(answeredVersionsJson(registry, pack, status, body)) as { versions: VersionInfo[] } | { error: string };
  if ("error" in answered) throw new Error(answered.error);
  return answered.versions;
}

/** Why `tarball` may not be fetched for the npm registry at `registry`: it isn't on its origin. */
export function offOrigin(tarball: string, registry: string): string | undefined {
  return offOriginOf(tarball, registry);
}
