// The configuration (docs/vite.md, decisions 1 and 2): `compmd.config.json`, or the `compmd` key of `package.json`,
// written for people and translated here into what the processor takes. No Node: a host that isn't a Vite app, such as
// the playground, reads the same configuration with these functions.

import type { Policy } from "@compmd/processor";
import semver from "semver";

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

const SCOPED = /^@[a-z0-9][a-z0-9._~-]*\/[a-z0-9][a-z0-9._~-]*$/;
const PACKAGE = /^(?:@[a-z0-9][a-z0-9._~-]*\/)?[a-z0-9][a-z0-9._~-]*$/;
const SIZE = /^(\d+(?:\.\d+)?)\s*(B|KB|MB|GB)$/i;
const UNITS: Record<string, number> = { B: 1, KB: 1e3, MB: 1e6, GB: 1e9 };
const SCOPE = /^@[a-z0-9~-][a-z0-9._~-]*$/;
const TOP_KEYS = new Set(["$schema", "packs", "imports", "deny", "maxPackSize", "libraries", "registries", "fetch"]);
const PACK_KEYS = new Set(["trust", "network", "host", "ambient", "version"]);

const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const isStrings = (value: unknown): value is string[] => Array.isArray(value) && value.every((item) => typeof item === "string");

/** `origin` as `https://host[:port]`, or why it isn't one a pack may be given. */
function originOf(origin: string): { origin: string } | { problem: string } {
  let url: URL;
  try {
    url = new URL(origin);
  } catch {
    return { problem: `\`${origin}\` isn't a URL` };
  }
  const local = ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname);
  if (url.protocol !== "https:" && !(url.protocol === "http:" && local)) return { problem: `\`${origin}\` must be \`https:\` (\`http:\` only for localhost)` };
  if (url.hostname.includes("*") || origin.includes("*")) return { problem: `\`${origin}\` has a wildcard: name each origin` };
  if (url.pathname !== "/" || url.search || url.hash || url.username) return { problem: `\`${origin}\` must be an origin alone, without a path` };
  return { origin: url.origin };
}

/** `url` as a registry's URL, ending in `/`, or why it can't be one. */
function registryUrlOf(url: string): { url: string } | { problem: string } {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return { problem: `\`${url}\` isn't a URL: a registry is named in full, such as \`https://registry.npmjs.org/\`` };
  }
  const local = ["localhost", "127.0.0.1", "[::1]"].includes(parsed.hostname);
  if (parsed.protocol !== "https:" && !(parsed.protocol === "http:" && local)) return { problem: `\`${url}\` must be \`https:\` (\`http:\` only for localhost)` };
  if (parsed.username || parsed.password || parsed.search || parsed.hash) return { problem: `\`${url}\` must have no credentials, query or fragment: a registry's token belongs to the host's proxy` };
  return { url: parsed.href.endsWith("/") ? parsed.href : `${parsed.href}/` };
}

/** The registries read, adding what's wrong with them to `problems`. */
function readRegistries(registries: unknown, problems: string[]): Record<string, string> {
  if (!isRecord(registries)) {
    problems.push("registries: an object, by scope (`@kit`), with `*` for every other scope");
    return {};
  }
  const read: Record<string, string> = {};
  for (const [scope, url] of Object.entries(registries)) {
    const at = `registries["${scope}"]`;
    if (scope !== "*" && !SCOPE.test(scope)) problems.push(`${at}: a scope, such as \`@kit\`, or \`*\` for every other scope`);
    const checked = typeof url === "string" ? registryUrlOf(url) : { problem: "a registry's URL" };
    if ("problem" in checked) problems.push(`${at}: ${checked.problem}`);
    else read[scope] = checked.url;
  }
  return read;
}

/** `size` in bytes, or `undefined` when it isn't a size. */
function bytesOf(size: unknown): number | undefined {
  if (typeof size === "number") return Number.isFinite(size) && size > 0 ? size : undefined;
  if (typeof size !== "string") return undefined;
  const match = SIZE.exec(size.trim());
  return match ? Number(match[1]) * (UNITS[(match[2] ?? "B").toUpperCase()] ?? 1) : undefined;
}

/** The pack `key`'s options read, adding what's wrong with them to `problems`. */
function readPack(key: string, options: unknown, problems: string[]): ReadPack {
  const at = `packs["${key}"]`;
  if (!isFolder(key) && !SCOPED.test(key)) problems.push(`${at}: a pack is a scoped name (\`@scope/pack\`) or a folder path (\`./components\`)`);
  if (!isRecord(options)) {
    problems.push(`${at}: its options are an object, \`{}\` for none`);
    return { trust: { level: "strict" }, ambient: false };
  }
  for (const name of Object.keys(options)) if (!PACK_KEYS.has(name)) problems.push(`${at}.${name}: not an option; a pack takes ${[...PACK_KEYS].join(", ")}`);
  const { trust = "strict", network, host, ambient = false, version } = options;
  if (trust !== "strict" && trust !== "permissive" && trust !== "trusted") problems.push(`${at}.trust: \`strict\`, \`permissive\` or \`trusted\``);
  if (trust === "trusted" && !isFolder(key)) problems.push(`${at}.trust: only a folder in the app may be trusted, never a registry pack; a registry pack that needs more is \`permissive\``);
  if (network !== undefined && trust !== "permissive") problems.push(`${at}.network: only a permissive pack reaches the network`);
  if (host !== undefined && trust !== "permissive") problems.push(`${at}.host: only a permissive pack is given \`host\``);
  if (host !== undefined && typeof host !== "boolean") problems.push(`${at}.host: true or false`);
  if (typeof ambient !== "boolean") problems.push(`${at}.ambient: true or false`);
  if (version !== undefined && (typeof version !== "string" || semver.validRange(version) === null)) problems.push(`${at}.version: a semver range, such as \`^1.2.0\``);
  const origins: string[] = [];
  if (network !== undefined && !isStrings(network)) problems.push(`${at}.network: a list of origins`);
  for (const written of isStrings(network) ? network : []) {
    const read = originOf(written);
    if ("problem" in read) problems.push(`${at}.network: ${read.problem}`);
    else origins.push(read.origin);
  }
  const read: ReadPack = {
    trust: trust === "permissive" ? { level: "permissive", network: origins, host: host === true } : trust === "trusted" ? { level: "trusted" } : { level: "strict" },
    ambient: ambient === true,
  };
  if (typeof version === "string") read.version = version;
  return read;
}

/** The libraries read, adding what's wrong with them to `problems`. */
function readLibraries(libraries: unknown, problems: string[]): Library[] {
  if (!Array.isArray(libraries)) {
    problems.push("libraries: a list of npm package names, or `{name, imports}`");
    return [];
  }
  return libraries.flatMap((library: unknown, i) => {
    const read = typeof library === "string" ? { name: library, imports: [library] } : isRecord(library) && typeof library.name === "string" && isStrings(library.imports) ? { name: library.name, imports: library.imports } : undefined;
    if (!read || !PACKAGE.test(read.name) || read.imports.length === 0) {
      problems.push(`libraries[${String(i)}]: an npm package name, such as \`"d3"\`, or \`{ "name": "d3", "imports": ["d3"] }\``);
      return [];
    }
    return [read];
  });
}

/**
 * Reads and checks a configuration.
 *
 * @param value The parsed JSON: `compmd.config.json`'s content, or `package.json`'s `compmd` key.
 * @param where Where it came from, for the error's message.
 * @throws ConfigError with every problem found.
 * @example
 * const config = readConfig({ packs: { "./components": { trust: "trusted", ambient: true } } });
 */
export function readConfig(value: unknown, where = "compmd.config.json"): ReadConfig {
  const problems: string[] = [];
  if (!isRecord(value)) throw new ConfigError(where, ["the configuration is an object"]);
  for (const key of Object.keys(value)) if (!TOP_KEYS.has(key)) problems.push(`${key}: not a key of the configuration; it takes ${[...TOP_KEYS].join(", ")}`);
  const packs = new Map<string, ReadPack>();
  if (value.packs !== undefined && !isRecord(value.packs)) problems.push("packs: an object, by pack name or folder path");
  for (const [key, options] of Object.entries(isRecord(value.packs) ? value.packs : {})) packs.set(key, readPack(key, options, problems));
  const imports = value.imports ?? "any";
  if (imports !== "any" && imports !== "listed") problems.push("imports: `any` or `listed`");
  const deny = Array.isArray(value.deny) ? value.deny : [];
  if (value.deny !== undefined && !Array.isArray(value.deny)) problems.push("deny: a list of `{pack, version?, reason}`");
  const denied = deny.flatMap((entry: unknown, i) => {
    const at = `deny[${String(i)}]`;
    if (!isRecord(entry) || typeof entry.pack !== "string" || typeof entry.reason !== "string" || entry.reason.trim() === "") {
      problems.push(`${at}: \`{pack, reason}\`, and \`version\` for some versions only`);
      return [];
    }
    if (!SCOPED.test(entry.pack)) problems.push(`${at}.pack: a scoped name, \`@scope/pack\``);
    if (entry.version !== undefined && (typeof entry.version !== "string" || semver.validRange(entry.version) === null)) problems.push(`${at}.version: a semver range, such as \`<2\``);
    return [{ pack: entry.pack, reason: entry.reason, ...(typeof entry.version === "string" ? { version: entry.version } : {}) }];
  });
  const maxPackSize = value.maxPackSize === undefined ? undefined : bytesOf(value.maxPackSize);
  if (value.maxPackSize !== undefined && maxPackSize === undefined) problems.push('maxPackSize: bytes, or a size such as `"5 MB"`');
  const libraries = value.libraries === undefined ? [] : readLibraries(value.libraries, problems);
  const registries = value.registries === undefined ? {} : readRegistries(value.registries, problems);
  const fetch = value.fetch ?? "host";
  if (fetch !== "host" && fetch !== "direct") problems.push("fetch: `host` or `direct`");
  if (problems.length > 0) throw new ConfigError(where, problems);
  return {
    packs,
    imports: imports === "listed" ? "listed" : "any",
    deny: denied,
    ...(maxPackSize === undefined ? {} : { maxPackSize }),
    libraries,
    registries,
    fetch: fetch === "direct" ? "direct" : "host",
  };
}

/** What the processor takes from a configuration: the policy and the ambient packs (`Request.policy`, `Request.ambient`). */
export interface Translated {
  policy: Policy;
  ambient: string[];
}

/**
 * Translates a configuration into what the processor takes.
 *
 * @param config What {@link readConfig} gave.
 * @param nameOf The pack name of a pack key: a folder pack's `package.json` name; a pack name is its own.
 * @param refused Packs the host left out, each with why, such as being larger than `maxPackSize`.
 * @example
 * const { policy, ambient } = policyOf(config, (key) => (isFolder(key) ? "@acme/app" : key));
 * process({ path, files, now, policy, ambient });
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
  return { policy, ambient };
}
