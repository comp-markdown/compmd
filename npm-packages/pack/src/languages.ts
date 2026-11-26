// A pack's content languages (the companion §2; docs/content-languages.md, decisions 2 and 3): what `compmd.languages`
// in its `package.json` lists, checked before it is published. Each entry's `id` is a language one of its code renderers
// `renders`; its grammars are files the pack publishes that read as what they say they are; its other names are
// strings.

import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { initSync, process as processDocument, type Definition, type ParseNode, type SignatureFence } from "@compmd/processor";

/** An entry of `compmd.languages`, as the companion has it. */
export interface ContentLanguage {
  id: string;
  textmate?: string;
  treeSitter?: string;
  aliases?: string[];
  languageId?: string;
  extensions?: string[];
}

let loaded = false;

/** The processor's module, loaded once from its file. */
function loadProcessor(): void {
  if (loaded) return;
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
  loaded = true;
}

/** The languages the code renderers that `documents` (their texts, by path in the pack) define or sign `renders`. */
export function renderedLanguages(documents: Readonly<Record<string, string>>): Set<string> {
  loadProcessor();
  const languages = new Set<string>();
  for (const path of Object.keys(documents)) {
    // Only the parse tree is read, which no render time changes.
    const { parse } = processDocument({ path, files: { ...documents }, now: "1970-01-01T00:00:00Z" });
    const visit = (nodes: readonly ParseNode[]) => {
      for (const node of nodes) {
        if (node.type === "componentDefinition" || node.type === "componentSignature") {
          for (const language of (node as Definition | SignatureFence).signature?.renders ?? []) languages.add(language);
        }
        const { children } = node as { children?: ParseNode[] };
        if (children) visit(children);
      }
    };
    visit(parse.children);
  }
  return languages;
}

/** The `\0asm` a WebAssembly module starts with. */
const WASM_MAGIC = [0x00, 0x61, 0x73, 0x6d];

/** Why the TextMate grammar at `path` in `dir` isn't one, if it isn't: JSON with a `scopeName`. */
async function textmateProblem(dir: string, path: string): Promise<string | undefined> {
  try {
    const grammar = JSON.parse(await readFile(join(dir, ...path.split("/")), "utf8")) as { scopeName?: unknown };
    return typeof grammar.scopeName === "string" ? undefined : "has no `scopeName`, which a TextMate grammar names itself by";
  } catch {
    return "isn't JSON";
  }
}

/** Why the tree-sitter grammar at `path` in `dir` isn't one, if it isn't: a WebAssembly module. */
async function treeSitterProblem(dir: string, path: string): Promise<string | undefined> {
  const bytes = await readFile(join(dir, ...path.split("/")));
  return WASM_MAGIC.every((byte, i) => bytes[i] === byte) ? undefined : "isn't a WebAssembly module";
}

const isStrings = (value: unknown): boolean => Array.isArray(value) && value.every((item) => typeof item === "string");

/**
 * What is wrong with `languages`, a pack's `compmd.languages`, each saying what to fix: the pack in `dir` publishes
 * `published`, and its renderers render `rendered`.
 *
 * @example
 * await languageProblems([{ id: "mermaid", textmate: "grammars/mermaid.json" }], dir, published, new Set(["mermaid"]));
 */
export async function languageProblems(languages: unknown, dir: string, published: ReadonlySet<string>, rendered: ReadonlySet<string>): Promise<string[]> {
  if (languages === undefined) return [];
  if (!Array.isArray(languages)) return ["package.json's `compmd.languages` is a list of the pack's content languages"];
  const problems: string[] = [];
  for (const [i, entry] of languages.entries()) {
    const at = `\`compmd.languages[${String(i)}]\``;
    if (typeof entry !== "object" || entry === null || typeof (entry as { id?: unknown }).id !== "string") {
      problems.push(`${at} needs an \`id\`, the language its renderer \`renders\``);
      continue;
    }
    const language = entry as ContentLanguage & Record<string, unknown>;
    if (!rendered.has(language.id)) problems.push(`${at}'s \`id\` is ${language.id}, which none of the pack's renderers \`renders\`${rendered.size > 0 ? ` (they render ${[...rendered].join(", ")})` : ""}`);
    for (const [key, problemOf] of [["textmate", textmateProblem], ["treeSitter", treeSitterProblem]] as const) {
      const path = language[key];
      if (path === undefined) continue;
      if (typeof path !== "string") problems.push(`${at}'s \`${key}\` is a path in the pack`);
      else if (!existsSync(join(dir, ...path.split("/")))) problems.push(`${at}'s \`${key}\`, ${path}, isn't there`);
      else if (!published.has(path)) problems.push(`${at}'s \`${key}\`, ${path}, isn't published: add it to package.json's \`files\``);
      else {
        const problem = await problemOf(dir, path);
        if (problem) problems.push(`${at}'s \`${key}\`, ${path}, ${problem}`);
      }
    }
    for (const key of ["aliases", "extensions"] as const) {
      if (language[key] !== undefined && !isStrings(language[key])) problems.push(`${at}'s \`${key}\` is a list of strings`);
    }
    if (language.languageId !== undefined && typeof language.languageId !== "string") problems.push(`${at}'s \`languageId\` is a string`);
  }
  return problems;
}
