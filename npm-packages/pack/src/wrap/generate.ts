// A wrapped pack's signature, written at each build (docs/wrapping-an-npm-package.md, decisions 4 to 7): the package's
// props read from its types, mapped to params, kept to the allowlist; the fence between the generated comments written,
// or, without them, the author's compared; and what changed, what was left out and what the package added, reported.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { PackError } from "../pack.ts";
import { changesOf, stillNamed } from "./changes.ts";
import { WRAP_FILE, type Wrap } from "./config.ts";
import { generatedIn, opening, withGenerated } from "./generated.ts";
import { installedPackage } from "./installed.ts";
import { isParam, mapProp, type Left, type Param } from "./map.ts";
import { readProps, type Prop } from "./props.ts";
import { signatureFence } from "./signature.ts";

/** What a build of a wrapped pack says. */
export interface WrapReport {
  /** What it wrote or found, and what changed: for the author to read. */
  notes: string[];
  /** What fails the build. */
  errors: string[];
}

/** The params a URL names, which a strict frame can't fetch (decision 9). */
const URL_PARAMS = new Set(["src", "url", "href"]);

/** The params of `props` that the allowlist names, in its order, and what the package has that it doesn't name. */
export function paramsOf(props: readonly Prop[], wrap: Wrap): { params: Param[]; left: Left[]; unlisted: string[]; gone: string[] } {
  const byName = new Map(props.map((prop) => [prop.name, prop]));
  const params: Param[] = [];
  const left: Left[] = [];
  for (const name of wrap.params) {
    const prop = byName.get(name);
    if (!prop) continue;
    const mapped = mapProp(prop, wrap);
    if (isParam(mapped)) params.push(mapped);
    else left.push(mapped);
  }
  const listed = new Set(wrap.params);
  const unlisted = props.filter((prop) => !listed.has(prop.name) && isParam(mapProp(prop, wrap))).map((prop) => prop.name);
  const gone = wrap.params.filter((name) => !byName.has(name));
  return { params, left, unlisted, gone };
}

/** The installed version of `name`, as the pack in `dir` finds it. */
function installedVersion(dir: string, name: string): string {
  const installed = installedPackage(dir, name);
  if (!installed) throw new PackError(`${name} isn't installed: run npm install`);
  return installed.version;
}

const readText = (file: string): string | undefined => (existsSync(file) ? readFileSync(file, "utf8") : undefined);

/**
 * What the build says of an author's own signature fence, `fence`, which it leaves as it is: where it differs from what
 * the package's types give, when they can be read.
 */
function compared(dir: string, document: string, wrap: Wrap, fence: string | undefined): string[] {
  const notes = [`${document} has no compmd-pack:generated comments, so its signature is yours: the build leaves it`];
  if (fence === undefined) return notes;
  try {
    const component = /component=([^\s]+)/.exec(fence)?.[1] ?? "";
    const changes = changesOf(fence, signatureFence(component, paramsOf(readProps(dir, wrap), wrap).params, wrap));
    if (changes.length > 0) notes.push(`${wrap.from}'s types give another signature of \`${component}\` than yours: ${changes.join("; ")}`);
  } catch (error) {
    notes.push(`it can't be compared with ${wrap.from}'s types: ${error instanceof Error ? error.message : String(error)}`);
  }
  return notes;
}

/**
 * Writes the generated signature of the wrapped pack in `dir` into its document, `document`, and says what it did.
 *
 * @example
 * const { notes, errors } = generateSignature("qr-code", "index.comp.md", wrap);
 */
export function generateSignature(dir: string, document: string, wrap: Wrap): WrapReport {
  const file = join(dir, document);
  const text = readFileSync(file, "utf8");
  const generated = generatedIn(text, "");
  if (generated.kind === "broken") return { notes: [], errors: [`${document}: ${generated.why}`] };
  if (generated.kind === "authors") return { notes: compared(dir, document, wrap, generated.fence), errors: [] };
  const notes: string[] = [];
  const errors: string[] = [];
  const { params, left, unlisted, gone } = paramsOf(readProps(dir, wrap), wrap);
  const named = stillNamed(gone, { "demo.comp.md": readText(join(dir, "demo.comp.md")) ?? "", [WRAP_FILE]: JSON.stringify(wrap.overrides ?? {}) });
  errors.push(...named);
  for (const name of gone) if (named.every((line) => !line.includes(`\`${name}\``))) notes.push(`\`${name}\` is in ${WRAP_FILE}'s params, but ${wrap.from} has no such prop now`);
  for (const { prop, why } of left) notes.push(`\`${prop}\` isn't a param: ${why}`);
  for (const param of params) if (param.note !== undefined) notes.push(`\`${param.name}\` takes ${param.note}`);
  if (unlisted.length > 0) notes.push(`${wrap.from} has props that aren't params: ${unlisted.map((name) => `\`${name}\``).join(", ")}; add any you want to ${WRAP_FILE}'s params`);
  const url = params.find((param) => URL_PARAMS.has(param.name) && !param.optional);
  if (url) notes.push(`\`${url.name}\` names a URL, which a strict frame can't fetch: the pack needs the file in the pack, or a permissive level with its origin`);
  const fence = signatureFence(generated.component, params, wrap);
  const changes = changesOf(generated.fence, fence);
  if (changes.length > 0) notes.push(`the signature of \`${generated.component}\`: ${changes.join("; ")}`);
  const written = withGenerated(text, generated, opening(wrap.from, installedVersion(dir, wrap.from)), fence);
  if (written !== text) writeFileSync(file, written);
  return { notes, errors };
}
