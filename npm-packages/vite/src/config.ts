// The configuration (docs/vite.md, decisions 1 and 2): `compmd.config.json`, or the `compmd` key of `package.json`,
// written for people, read and checked by `compmd-project` (docs/language-server.md, Q3) through `@compmd/host`'s
// module, and translated here into what the processor takes. No Node: a host that isn't a Vite app reads the same
// configuration with these functions; in Node the module loads when first used, and a page or a worker loads it first
// (`init()` from `@compmd/host`).

import { readConfig as readProjectConfig, type ProjectConfig } from "@compmd/host/project";
import type { Policy } from "@compmd/processor";

export { cspFor, type Csp } from "./csp.ts";

/** How far a pack's code is trusted, as the configuration writes it. */
export type TrustLevel = "strict" | "permissive" | "trusted";

/** A pack's options, as written. */
export interface PackOptions {
  /** `strict` (the default), `permissive`, or `trusted`, for a folder pack only. */
  trust?: TrustLevel;
  /** The origins a permissive pack's code may reach, each `https://host[:port]`. */
  network?: string[];
  /** Whether a permissive pack's components get `host`. */
  host?: boolean;
  /** Whether every document gets the pack's names without importing it. */
  ambient?: boolean;
  /** A semver range the installed version must satisfy. */
  version?: string;
}

/** A library the frames offer components through their import map: an npm package and the import names it gives. */
export interface Library {
  name: string;
  imports: string[];
}

/** The configuration, as written. */
export interface Config {
  $schema?: string;
  /** By pack name (`@acme/maps`) or folder path (`./components`). */
  packs?: Record<string, PackOptions>;
  /** `any` (the default): a document may import any installed pack; `listed`: only those listed here. */
  imports?: "any" | "listed";
  /** Packs denied, all versions or those in `version`, with the reason readers see. */
  deny?: { pack: string; version?: string; reason: string }[];
  /** A pack whose files are larger is left out: bytes, or text such as `"5 MB"`. */
  maxPackSize?: number | string;
  /** npm packages the frames offer through their import map, besides React and Svelte: a name, or a name and its imports. */
  libraries?: (string | Library)[];
  /** The registries packs the app doesn't install come from, by scope (`@kit`), with `*` for every other scope. */
  registries?: Record<string, string>;
  /** Who fetches from them: `host` (the default), through the app's own proxy; `direct`, the reader's browser. */
  fetch?: "host" | "direct";
  /**
   * The origins documents' remote data and images may come from without asking the reader, each `https://host[:port]`;
   * or `"any"`, every origin, for documents the app publishes itself, such as a static site's or a blog's (spec §15).
   * None by default: the reader is asked about each.
   */
  origins?: string[] | "any";
  /** What an editor offers for the project's documents; every other host ignores it. */
  editor?: EditorOptions;
}

/** What an editor offers, as written. */
export interface EditorOptions {
  /** The scopes whose packs completion and auto-import offer, besides CompMD's own, such as `@acme`. */
  suggest?: string[];
}

/** A pack's trust, as the processor takes it. */
export type Trust = { level: "strict" } | { level: "permissive"; network: string[]; host: boolean } | { level: "trusted" };

/** A listed pack, read. */
export interface ReadPack {
  trust: Trust;
  ambient: boolean;
  version?: string;
}

/** The configuration, read and checked: every default filled in, sizes in bytes, origins normalized. */
export interface ReadConfig {
  /** By pack name or folder path, as written. */
  packs: Map<string, ReadPack>;
  imports: "any" | "listed";
  deny: { pack: string; version?: string; reason: string }[];
  /** In bytes; absent for no limit. */
  maxPackSize?: number;
  libraries: Library[];
  /** Each registry's URL, ending in `/`, by scope or `*`; empty for none. */
  registries: Record<string, string>;
  fetch: "host" | "direct";
  /** The origins documents' data and images need no question for, normalized; or `"any"`; empty for none. */
  origins: string[] | "any";
  /** What an editor offers: the scopes it suggests, empty for none. */
  editor: { suggest: string[] };
}

/** What is wrong with a configuration: every problem found, each with where it is. */
export class ConfigError extends Error {
  readonly problems: readonly string[];

  constructor(where: string, problems: readonly string[]) {
    super(`${where}:\n${problems.map((problem) => `- ${problem}`).join("\n")}`);
    this.name = "ConfigError";
    this.problems = problems;
  }
}

/** Whether a pack key is a folder path, `./components`, rather than a pack name. */
export const isFolder = (key: string): boolean => key.startsWith("./") || key.startsWith("../");

/** The configuration `compmd-project` read, as {@link ReadConfig} writes it. */
export function readConfigOf(config: ProjectConfig): ReadConfig {
  const { packs, maxPackSize, ...rest } = config;
  return { ...rest, packs: new Map(packs), ...(maxPackSize === undefined ? {} : { maxPackSize }) };
}

/**
 * Reads and checks a configuration, as `compmd-project` does.
 *
 * @param value The parsed JSON: `compmd.config.json`'s content, or `package.json`'s `compmd` key.
 * @param where Where it came from, for the error's message.
 * @throws ConfigError with every problem found.
 * @example
 * const config = readConfig({ packs: { "./components": { trust: "trusted", ambient: true } } });
 */
export function readConfig(value: unknown, where = "compmd.config.json"): ReadConfig {
  const read = readProjectConfig(value, where);
  if ("problems" in read) throw new ConfigError(where, read.problems);
  if ("error" in read) throw new ConfigError(where, [read.error]);
  return readConfigOf(read.config);
}

/**
 * What the processor takes from a configuration: the policy, the ambient packs and the origins remote data may come
 * from (`Request.policy`, `Request.ambient`, `Request.allowedOrigins`).
 */
export interface Translated {
  policy: Policy;
  ambient: string[];
  allowedOrigins: string[] | "any";
}

/**
 * Translates a configuration into what the processor takes.
 *
 * @param config What {@link readConfig} gave.
 * @param nameOf The pack name of a pack key: a folder pack's `package.json` name; a pack name is its own.
 * @param refused Packs the host left out, each with why, such as being larger than `maxPackSize`.
 * @example
 * const { policy, ambient, allowedOrigins } = policyOf(config, (key) => (isFolder(key) ? "@acme/app" : key));
 * process({ path, files, now, policy, ambient, allowedOrigins });
 */
export function policyOf(config: ReadConfig, nameOf: (key: string) => string, refused: Readonly<Record<string, string>> = {}): Translated {
  const packs: NonNullable<Policy["packs"]> = {};
  const ambient: string[] = [];
  for (const [key, pack] of config.packs) {
    const name = nameOf(key);
    packs[name] = pack.trust;
    if (pack.ambient) ambient.push(name);
  }
  const policy: Policy = { packs, imports: config.imports, deny: config.deny };
  if (Object.keys(refused).length > 0) policy.refused = { ...refused };
  return { policy, ambient, allowedOrigins: config.origins === "any" ? "any" : [...config.origins] };
}
