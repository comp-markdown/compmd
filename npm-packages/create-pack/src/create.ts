// Writes a pack from the template (docs/create-pack.md): the files every pack has, and those of a leaf or a code
// renderer, with the pack's and the component's names filled in, and the versions of the packages it builds and tests
// with, as they were when this package was released; or a library pack, which has its own files.

import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { basename, dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

/** What a pack is made with. */
export interface PackOptions {
  /** The folder to write it in, which must not exist or be empty. */
  folder: string;
  /** The npm package, scoped: `@kit/mermaid`. */
  name: string;
  /** The component's name, which documents use: `mermaid`. */
  component: string;
  /** A language the component renders the code blocks of, which makes it a code renderer (spec §7.7). */
  renders?: string;
  /** The npm package it wraps, which makes it a library pack (companion, section 5), with no component. */
  library?: string;
}

/** What is wrong with what a pack is made with. */
export class CreateError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CreateError";
  }
}

const HERE = dirname(fileURLToPath(import.meta.url));
const TEMPLATE = join(HERE, "../template");
const MANIFEST = join(HERE, "../package.json");
const SCOPED = /^@[a-z0-9][a-z0-9._~-]*\/[a-z0-9][a-z0-9._~-]*$/;
const COMPONENT = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/;
const LANGUAGE = /^[a-z][a-z0-9+-]*$/;
const PACKAGE = /^(?:@[a-z0-9][a-z0-9._~-]*\/)?[a-z0-9][a-z0-9._~-]*$/;

/** A component name from a folder's name: `My Chart` is `my-chart`. */
export function componentOf(folder: string): string {
  const name = basename(folder)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^[^a-z]+|-+$/g, "");
  return COMPONENT.test(name) ? name : "component";
}

/** Why `options` can't make a pack, or `undefined` when they can. */
export function problemOf(options: PackOptions): string | undefined {
  if (!SCOPED.test(options.name)) return `${options.name} isn't a scoped npm name, such as @kit/mermaid: packs are always scoped`;
  if (options.library !== undefined) {
    if (!PACKAGE.test(options.library)) return `${options.library} isn't an npm package's name, such as three`;
    return options.renders === undefined ? undefined : "a library pack has no components, so it renders nothing: leave out --renders";
  }
  if (!COMPONENT.test(options.component)) return `${options.component} isn't a component name: lowercase letters and digits, words joined by -`;
  if (options.renders !== undefined && !LANGUAGE.test(options.renders)) return `${options.renders} isn't a code block's language, such as mermaid`;
  return undefined;
}

/**
 * The version each package the pack builds with is pinned to: this package's own, as its release wrote them; in the
 * repository, where they are `workspace:`, the version installed.
 */
async function versions(): Promise<Record<string, string>> {
  const manifest = JSON.parse(await readFile(MANIFEST, "utf8")) as { devDependencies?: Record<string, string> };
  const found: Record<string, string> = {};
  for (const [name, written] of Object.entries(manifest.devDependencies ?? {})) {
    found[name] = written.startsWith("workspace:")
      ? (JSON.parse(await readFile(join(HERE, "../node_modules", ...name.split("/"), "package.json"), "utf8")) as { version: string }).version
      : written;
  }
  return found;
}

/** Every file under `dir`, by its path there. */
async function filesUnder(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries.filter((entry) => entry.isFile()).map((entry) => relative(dir, join(entry.parentPath, entry.name)));
}

/**
 * Writes a pack into `options.folder`.
 *
 * @returns The files written, by their paths in the folder, `/`-separated.
 * @throws CreateError when the options can't make a pack, or the folder has files in it.
 * @example
 * await createPack({ folder: "mermaid", name: "@kit/mermaid", component: "mermaid", renders: "mermaid" });
 * await createPack({ folder: "three", name: "@kit/three", component: "three", library: "three" });
 */
export async function createPack(options: PackOptions): Promise<string[]> {
  const problem = problemOf(options);
  if (problem) throw new CreateError(problem);
  if (existsSync(options.folder) && (await readdir(options.folder)).length > 0) throw new CreateError(`${options.folder} has files in it: choose a new folder, or an empty one`);
  const pinned = await versions();
  const fill = (text: string) =>
    text
      .replace(/__version:(.+?)__/g, (_, name: string) => pinned[name] ?? "latest")
      .replaceAll("__pack__", options.name)
      .replaceAll("__component__", options.component)
      .replaceAll("__language__", options.renders ?? "")
      .replaceAll("__library__", options.library ?? "");
  const parts = options.library !== undefined ? ["library"] : ["common", options.renders === undefined ? "leaf" : "renderer"];
  const written: string[] = [];
  for (const part of parts) {
    for (const path of await filesUnder(join(TEMPLATE, part))) {
      // npm leaves `.gitignore` out of published packages, so the template's has no dot.
      const to = fill(path.split(sep).join("/")).replace(/(^|\/)gitignore$/, "$1.gitignore");
      const file = join(options.folder, ...to.split("/"));
      await mkdir(dirname(file), { recursive: true });
      await writeFile(file, fill(await readFile(join(TEMPLATE, part, path), "utf8")));
      written.push(to);
    }
  }
  return written.sort();
}
