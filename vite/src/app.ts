// What the plugin knows of an app (docs/vite.md, "What the plugin does"): its configuration, its packs, what the
// processor and the sandbox are given, the files served under `compmd/`, and what the checks found. Node only; the
// plugin turns it into Vite's hooks.

import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { initSync, process as processDocument, type Diagnostic, type PackRelease } from "@compmd/processor";
import semver from "semver";
import { parse as parseYaml } from "yaml";
import { isFolder, policyOf, readConfig, type ReadConfig, type Translated } from "./config.ts";
import { findPacks, integrities, releaseOf, type ComponentFile, type Pack } from "./packs.ts";

/** A trusted component, which the app's own build compiles and the sandbox mounts in the page. */
export interface TrustedComponent {
  /** The name its pack exports it as, which documents and the expanded tree use. */
  name: string;
  /** Its file. */
  file: string;
  /** The binding that mounts it. */
  binding: "react" | "svelte" | "plain";
}

/** A built module, as the sandbox loads it. */
export interface ServedModule {
  /** Its URL path under `compmd/`: `packs/<name>@<version>/<path>`. */
  path: string;
  /** Its file's hash, `sha384-…`, which the frame's script checks. */
  integrity: string;
}

/** Everything the plugin gives the app. */
export interface App {
  config: ReadConfig;
  packs: Pack[];
  /** What the processor takes: `Request.packs`, `ambient` and `policy`. */
  request: Translated & { packs: Record<string, PackRelease[]> };
  trusted: TrustedComponent[];
  /** Each built module by its place, `<pack>@<version>/<path>`, as the processor's `HostComponent.module` names it. */
  modules: Record<string, ServedModule>;
  /** Each file served under `compmd/packs/`, by its path there, with where it is on disk. */
  served: Map<string, string>;
  /** What the checks found that doesn't stop the build. */
  warnings: string[];
}

/** What stops the build: problems in the configuration, the packs, or what the processor found in them. */
export class AppError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "AppError";
  }
}

/** The bindings by component file language (spec §7.6). */
const BINDINGS: Record<ComponentFile["language"], TrustedComponent["binding"]> = { jsx: "react", tsx: "react", svelte: "svelte", js: "plain", ts: "plain" };
const FRONTMATTER = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/;

/**
 * The app's configuration: `compmd.config.json` at `root`, or the `compmd` key of its `package.json`; none is an empty
 * configuration.
 *
 * @throws AppError when both are there, or a file isn't JSON; ConfigError when the configuration is wrong.
 */
export async function configOf(root: string): Promise<ReadConfig> {
  const file = join(root, "compmd.config.json");
  const manifestFile = join(root, "package.json");
  const parse = async (path: string): Promise<unknown> => {
    try {
      return JSON.parse(await readFile(path, "utf8"));
    } catch (error) {
      throw new AppError(`${path} isn't JSON: ${error instanceof Error ? error.message : String(error)}`);
    }
  };
  const manifest = existsSync(manifestFile) ? ((await parse(manifestFile)) as { compmd?: unknown }) : {};
  if (existsSync(file)) {
    if (manifest.compmd !== undefined) throw new AppError("the configuration is in both compmd.config.json and package.json's `compmd` key: keep one");
    return readConfig(await parse(file), "compmd.config.json");
  }
  return readConfig(manifest.compmd ?? {}, "package.json (`compmd`)");
}

/** A pack document's frontmatter `md` key, or an empty one. */
function mdOf(text: string): Record<string, unknown> {
  try {
    const md = (parseYaml(FRONTMATTER.exec(text)?.[1] ?? "") as { md?: unknown } | null)?.md;
    return typeof md === "object" && md !== null ? (md as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

/** The names of an import or an export entry: `[name, as]`. */
function namesOf(entries: unknown): [string, string][] {
  if (!Array.isArray(entries)) return [];
  return entries.flatMap((entry: unknown): [string, string][] => {
    if (typeof entry === "string") return [[entry, entry]];
    if (typeof entry === "object" && entry !== null) {
      const { name, as } = entry as { name?: unknown; as?: unknown };
      if (typeof name === "string") return [[name, typeof as === "string" ? as : name]];
    }
    return [];
  });
}

/** A trusted pack's components, by the names its document exports, each with its file and binding. */
export function trustedComponents(pack: Pack): TrustedComponent[] {
  const md = mdOf(pack.files[pack.document] ?? "");
  const local = new Map<string, ComponentFile>();
  for (const entry of Array.isArray(md.import) ? (md.import as unknown[]) : []) {
    const { from, names } = (typeof entry === "object" && entry !== null ? entry : {}) as { from?: unknown; names?: unknown };
    if (typeof from !== "string") continue;
    const path = posix.normalize(posix.join(posix.dirname(pack.document), from));
    const file = pack.components.find((component) => component.path === path);
    if (!file) continue;
    const stem = posix.basename(path).replace(/\.[^.]+$/, "");
    for (const [, as] of names === "*" ? [[stem, stem] as [string, string]] : namesOf(names)) local.set(as, file);
  }
  const exported = (md.export as { names?: unknown } | undefined)?.names;
  const pairs = exported === "*" ? [...local.keys()].map((name): [string, string] => [name, name]) : namesOf(exported);
  return pairs.flatMap(([name, as]) => {
    const file = local.get(name);
    return file ? [{ name: as, file: join(pack.dir, ...file.path.split("/")), binding: BINDINGS[file.language] }] : [];
  });
}

/** `sha384-…` of the file at `file`, as a script's `integrity` takes it. */
const integrityOf = (file: string): string => `sha384-${createHash("sha384").update(readFileSync(file)).digest("base64")}`;

let processorLoaded = false;

/** Loads the processor's module, once. */
function loadProcessor(): void {
  if (processorLoaded) return;
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
  processorLoaded = true;
}

/** A diagnostic as a line of the build's output. */
const lineOf = ({ code, message, file, range }: Diagnostic): string => `${file ?? "the check"}:${String(range.start.line)}:${String(range.start.column)}: ${code} ${message}`;

/**
 * Processes each listed pack's document, by importing all of each listed pack in one document (docs/vite.md,
 * decision 7): what's in error stops the build; warnings, and packs left out (F026), are reported.
 *
 * @throws AppError with every error.
 */
function check(listed: readonly Pack[], request: App["request"], warnings: string[]): void {
  if (listed.length === 0) return;
  loadProcessor();
  const imports = listed.map((pack) => `    - from: "${pack.name}@${pack.version}"\n      names: "*"`).join("\n");
  const text = `---\nmd:\n  import:\n${imports}\n---\n`;
  const { diagnostics } = processDocument({ path: "check.comp.md", files: { "check.comp.md": text }, ...request, now: new Date(0).toISOString() });
  const errors = diagnostics.filter((diagnostic) => diagnostic.severity === "error" && diagnostic.code !== "F026");
  warnings.push(...diagnostics.filter((diagnostic) => !errors.includes(diagnostic)).map(lineOf));
  if (errors.length > 0) throw new AppError(`the listed packs have errors:\n${errors.map((error) => `- ${lineOf(error)}`).join("\n")}`);
}

/**
 * Everything the plugin gives the app at `root`.
 *
 * @throws AppError, ConfigError or PackError, saying what to fix.
 */
export async function loadApp(root: string): Promise<App> {
  const config = await configOf(root);
  const folders = [...config.packs.keys()].filter(isFolder);
  const packs = await findPacks(root, folders);
  const byKey = (key: string) => packs.find((pack) => (isFolder(key) ? pack.folder === key : pack.name === key && pack.folder === undefined));
  const problems: string[] = [];
  for (const [key, options] of config.packs) {
    const pack = byKey(key);
    if (!pack) problems.push(`${key} is listed in the configuration, but it isn't installed: \`npm install ${key}\``);
    else if (options.version !== undefined && !semver.satisfies(pack.version, options.version)) problems.push(`${key} is ${pack.version}, which isn't \`${options.version}\` as the configuration asks`);
  }
  for (const library of config.libraries) {
    if (!existsSync(join(root, "node_modules", ...library.name.split("/"), "package.json"))) problems.push(`the library ${library.name} isn't installed: \`npm install ${library.name}\``);
  }
  if (problems.length > 0) throw new AppError(problems.join("\n"));
  const locked = await integrities(root);
  const refused: Record<string, string> = {};
  for (const pack of packs) {
    const integrity = locked.get(`${pack.name}@${pack.version}`);
    if (integrity) pack.integrity = integrity;
    if (config.maxPackSize !== undefined && pack.size > config.maxPackSize) refused[pack.name] = `its files are ${String(pack.size)} bytes, more than the ${String(config.maxPackSize)} this app serves`;
  }
  const translated = policyOf(config, (key) => byKey(key)?.name ?? key, refused);
  const trustedPacks = packs.filter((pack) => translated.policy.packs?.[pack.name]?.level === "trusted");
  const releases: Record<string, PackRelease[]> = {};
  for (const pack of packs.filter((pack) => !(pack.name in refused))) (releases[pack.name] ??= []).push(releaseOf(pack, { trusted: trustedPacks.includes(pack) }));
  const request = { ...translated, packs: releases };
  const modules: Record<string, ServedModule> = {};
  const served = new Map<string, string>();
  for (const pack of packs.filter((pack) => !trustedPacks.includes(pack) && !(pack.name in refused))) {
    for (const component of pack.components.filter(({ language }) => language === "js")) {
      const file = join(pack.dir, ...component.path.split("/"));
      const path = `packs/${pack.name}@${pack.version}/${component.path}`;
      modules[`${pack.name}@${pack.version}/${component.path}`] = { path, integrity: integrityOf(file) };
      served.set(path, file);
    }
  }
  const warnings: string[] = [];
  const listed = packs.filter((pack) => [...config.packs.keys()].some((key) => byKey(key) === pack) && !(pack.name in refused));
  check(listed, request, warnings);
  for (const [name, why] of Object.entries(refused)) warnings.push(`${name} is left out: ${why}`);
  return { config, packs, request, trusted: trustedPacks.flatMap(trustedComponents), modules, served, warnings };
}
