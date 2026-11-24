// A pack folder's `package.json` (spec §6.4): its name, a scoped npm name as the processor reads one; its version; and
// its document, the `compmd` field or `index.comp.md`, a path in the pack.

import { readFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { isPackName } from "@compmd/registry";
import semver from "semver";
import { isPackPath } from "./confined.ts";
import { PackError } from "./pack-error.ts";

/** The `package.json` at `dir`, parsed; `undefined` when there is none, or it isn't a JSON object. */
export async function manifestAt(dir: string): Promise<Record<string, unknown> | undefined> {
  let manifest: unknown;
  try {
    manifest = JSON.parse(await readFile(join(dir, "package.json"), "utf8"));
  } catch {
    return undefined;
  }
  return typeof manifest === "object" && manifest !== null && !Array.isArray(manifest) ? (manifest as Record<string, unknown>) : undefined;
}

/**
 * The pack's name and version in `manifest`, read from `dir`: a pack's name is a scoped npm name, and its version
 * semver, `0.0.0` when it has none; either becomes a folder of the bundle (`packs/<name>@<version>/`).
 *
 * @throws PackError when the name isn't a pack's, or the version isn't semver.
 */
export function identityOf(manifest: Record<string, unknown>, dir: string): { name: string; version: string } {
  const { name, version = "0.0.0" } = manifest;
  if (typeof name !== "string") throw new PackError(`${dir} isn't a pack: it has no package.json with a name`);
  if (!isPackName(name)) throw new PackError(`${dir}: \`${name}\` isn't a pack's name: a pack is a scoped npm name, \`@scope/pack\``);
  if (typeof version !== "string" || semver.valid(version) !== version) throw new PackError(`${name}: its version, \`${String(version)}\`, isn't a semver version such as \`1.2.0\``);
  return { name, version };
}

/**
 * A pack's document path, from its manifest's `compmd` (spec §6.4), or `index.comp.md`.
 *
 * @throws PackError when `compmd` names a path outside the pack.
 * @example
 * documentOf({ compmd: "./index.comp.md" }); // "index.comp.md"
 */
export function documentOf(manifest: { compmd?: unknown }): string {
  if (typeof manifest.compmd !== "string") return "index.comp.md";
  const document = posix.normalize(manifest.compmd);
  if (!isPackPath(document)) throw new PackError(`package.json's \`compmd\` is ${manifest.compmd}, which isn't a file of the pack`);
  return document;
}
