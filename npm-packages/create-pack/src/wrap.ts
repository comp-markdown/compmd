// Writes a pack of an npm package (docs/wrapping-an-npm-package.md): the template's common files, the package installed
// and pinned to the exact version it got (decision 8), its kind read from it (decision 2), the glue for that kind
// (decision 3), `compmd.wrap.json` with every prop that maps as the allowlist (decisions 5 and 7), the document with
// its generated signature (decision 4), a demo, and what a strict frame would refuse (decision 9).

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { elementTags, generatedPart, installedPackage, isParam, kindOf, mapProp, opening, readProps, risksOf, signatureFence, WRAP_FILE, type Installed, type Kind, type Param, type Wrap } from "@compmd/pack/wrap";
import { CreateError, mustBeEmpty, PACKAGE, problemOf, versions, writePart } from "./create.ts";

/** What a wrapped pack is made with. */
export interface WrappedPackOptions {
  /** The folder to write it in, which must not exist or be empty. */
  folder: string;
  /** The pack's npm package, scoped: `@kit/qr-code`. */
  name: string;
  /** The component's name, which documents use: `qr-code`. */
  component: string;
  /** The npm package to wrap, with a version or not: `react-qr-code@2.2.0`. */
  from: string;
  /** The export that is the component (`BarChart`), or a web component's tag; by default the default export, or the only element. */
  export?: string;
  /** Its kind, over what is read from the package. */
  kind?: Kind;
  /** Installs the dependencies `package.json` names, in the pack's folder: `npm install` by default. */
  install?: (folder: string) => void;
}

/** A wrapped pack, written. */
export interface WrappedPack {
  /** The files written, by their paths in the folder, `/`-separated. */
  written: string[];
  kind: Kind;
  /** What the author should read: the props left out and why, and what a strict frame would refuse. */
  notes: string[];
}

/** The props that take the fallback's text when a document gives none, in the order they are looked for. */
const LABELS = ["title", "alt", "label", "ariaLabel"];

/** The development packages each kind's glue is written and tested with. */
const KIND_DEPENDENCIES: Record<Kind, string[]> = {
  react: ["react", "react-dom", "@types/react", "@types/react-dom"],
  svelte: ["svelte"],
  "web-component": [],
  library: [],
};

/** `react-qr-code@2.2.0` as its name and version; `@scope/name` with no version. */
export function packageSpec(from: string): { name: string; version: string | undefined } {
  const at = from.lastIndexOf("@");
  return at > 0 ? { name: from.slice(0, at), version: from.slice(at + 1) || undefined } : { name: from, version: undefined };
}

const npmInstall = (folder: string): void => {
  execFileSync("npm", ["install", "--no-fund", "--no-audit"], { cwd: folder, stdio: "inherit" });
};

/** The value a demo gives a required param of `type`. */
function demoValue(type: string): unknown {
  const [member] = type.split("|").map((part) => part.trim());
  if (type.includes("|") && member) return member;
  const values: Record<string, unknown> = { number: 1, boolean: true, date: "2026-01-01", array: [], object: {}, color: "#24292f" };
  return values[type.replace(/^input<(.*)>$/, "$1")] ?? "Hello";
}

/** The demo's params: a value for each required param. */
const demoParams = (params: readonly Param[]): Record<string, unknown> =>
  Object.fromEntries(params.filter((param) => !param.optional && param.default === undefined).map((param) => [param.name, demoValue(param.type)]));

/** `params` as a use's param list: each value as JSON, which a param value may be (spec §3.1). */
const useOf = (params: Record<string, unknown>): string =>
  Object.entries(params)
    .map(([key, value]) => `${key}=${JSON.stringify(value)}`)
    .join(" ");

/** What the package's entry file has that a strict frame refuses, before it is built. */
function entryRisks(installed: Installed): string[] {
  const entry = installed.module ?? installed.main;
  if (entry === undefined) return [];
  try {
    return risksOf(`${installed.name}'s ${entry}`, readFileSync(join(installed.dir, entry), "utf8"));
  } catch {
    return [];
  }
}

/** The pack's document: its imports and export, a line on what it wraps, and its signature. */
function documentOf(options: WrappedPackOptions, from: Installed, signature: string): string {
  return `---
md:
  import:
    - from: ./dist/${options.component}.js
      names: [${options.component}]
  export:
    pack: "${options.name}"
    version: "0.1.0"
    names: [${options.component}]
---

# ${options.name}

\`${options.component}\` is [${from.name}](https://www.npmjs.com/package/${from.name}) ${from.version}, wrapped as a CompMD component. It fills its box, which a use sizes: \`width=12rem\`, \`height\`, \`aspect-ratio\`.

${signature}
`;
}

/** The demo: one use as it is, then sized, fitted in a short box, and covering its box. */
function demoOf(options: WrappedPackOptions, given: string): string {
  const use = (label: string, more = "") => `:: ${options.component}[${label}] {${[given, more].filter(Boolean).join(" ")}}`;
  return `---
md:
  import:
    - from: "${options.name}@0"
      names: [${options.component}]
---

# ${options.component}

${use(`${options.component}, as text`)}

Sized by the document: a width, a short box it is fitted in, and a box it covers.

${use(`${options.component}, wider`, "width=20rem")}

${use(`${options.component}, in a short box`, "width=100% height=6rem")}

${use(`${options.component}, covering its box`, "width=16rem height=8rem object-fit=cover")}
`;
}

/** The export that is the component: the one given, a web component's only element, or the default export. */
function exportOf(folder: string, from: string, kind: Kind, given: string | undefined): string {
  if (given !== undefined) return given;
  if (kind !== "web-component") return "default";
  const tags = elementTags(folder, from);
  const [only] = tags;
  if (tags.length === 1 && only !== undefined) return only;
  throw new CreateError(tags.length === 0 ? `${from} defines no custom element in its types: give its tag with --export` : `${from} defines ${tags.join(", ")}: choose one with --export`);
}

/**
 * Writes a pack of the npm package `options.from` into `options.folder`, and installs it.
 *
 * @throws CreateError when the options can't make a pack, the folder has files in it, or the package's component
 * can't be found.
 * @example
 * await createWrappedPack({ folder: "qr-code", name: "@kit/qr-code", component: "qr-code", from: "react-qr-code@2.2.0" });
 */
export async function createWrappedPack(options: WrappedPackOptions): Promise<WrappedPack> {
  const problem = problemOf(options);
  if (problem) throw new CreateError(problem);
  const spec = packageSpec(options.from);
  if (!PACKAGE.test(spec.name)) throw new CreateError(`${spec.name} isn't an npm package's name, such as react-qr-code`);
  await mustBeEmpty(options.folder);
  const install = options.install ?? npmInstall;
  const pinned = await versions();
  const fill = (text: string) =>
    text
      .replace(/__version:(.+?)__/g, (_, name: string) => pinned[name] ?? "latest")
      .replaceAll("__pack__", options.name)
      .replaceAll("__component__", options.component);
  const written = await writePart(options.folder, "common", fill);
  const manifestFile = join(options.folder, "package.json");
  const manifest = JSON.parse(readFileSync(manifestFile, "utf8")) as { description: string; dependencies?: Record<string, string>; devDependencies: Record<string, string> };
  const writeManifest = () => {
    writeFileSync(manifestFile, `${JSON.stringify(manifest, null, 2)}\n`);
  };
  manifest.dependencies = { [spec.name]: spec.version ?? "latest" };
  writeManifest();
  install(options.folder);
  const installed = installedPackage(options.folder, spec.name);
  if (!installed) throw new CreateError(`${spec.name} isn't installed in ${options.folder}: npm install didn't install it`);
  // One version of the pack is one build of the package (decision 8).
  manifest.dependencies = { [spec.name]: installed.version };
  manifest.description = `A CompMD pack of ${spec.name} ${installed.version}: ${options.component}.`;
  const kind = options.kind ?? kindOf(options.folder, installed);
  const needed = KIND_DEPENDENCIES[kind].filter((name) => !(name in manifest.devDependencies));
  for (const name of needed) manifest.devDependencies[name] = pinned[name] ?? "latest";
  writeManifest();
  if (needed.length > 0) install(options.folder);
  const exported = exportOf(options.folder, spec.name, kind, options.export);
  const base: Wrap = { from: spec.name, export: exported, kind, params: [] };
  const notes: string[] = [];
  let params: Param[] = [];
  if (kind !== "library") {
    const mapped = readProps(options.folder, base).map((prop) => mapProp(prop, base));
    params = mapped.filter(isParam);
    for (const left of mapped) if (!isParam(left)) notes.push(`\`${left.prop}\` isn't a param: ${left.why}`);
  }
  const label = LABELS.find((name) => params.some((param) => param.name === name && param.type === "string"));
  const wrap: Wrap = { ...base, params: params.map((param) => param.prop), ...(label === undefined ? {} : { label }) };
  writeFileSync(join(options.folder, WRAP_FILE), `${JSON.stringify(wrap, null, 2)}\n`);
  const given = kind === "library" ? { value: "Hello" } : demoParams(params);
  const use = useOf(given);
  const glue = (text: string) =>
    fill(text)
      .replaceAll("__from__", spec.name)
      .replaceAll("__version__", installed.version)
      .replaceAll("__export__", exported)
      .replaceAll("__imported__", exported === "default" ? "Wrapped" : `{ ${exported} as Wrapped }`)
      .replaceAll("__props__", label === undefined ? "next" : `({ ...next, ${label}: next["${label}"] ?? context.fallback.text })`)
      .replaceAll("__label_note__", label === undefined ? "" : `, with \`${label}\` the fallback's text when the document gives none`)
      .replaceAll("__params__", JSON.stringify(given))
      .replaceAll("__use__", use);
  written.push(...(await writePart(options.folder, "wrap/common", glue)), ...(await writePart(options.folder, `wrap/${kind}`, glue)));
  const signature =
    kind === "library"
      ? `\`\`\`compmd component=${options.component} signature\nparams:\n  value: string\n\`\`\``
      : generatedPart(opening(spec.name, installed.version), signatureFence(options.component, params, wrap));
  writeFileSync(join(options.folder, "index.comp.md"), documentOf(options, installed, signature));
  writeFileSync(join(options.folder, "demo.comp.md"), demoOf(options, use));
  written.push(WRAP_FILE, "index.comp.md", "demo.comp.md");
  notes.push(...entryRisks(installed));
  return { written: [...new Set(written)].sort(), kind, notes };
}
