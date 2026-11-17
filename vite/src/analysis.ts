// Component analysis (spec §7.1, §13.1): each host-language component of an app's listed packs checked against its
// signature fence, W104 where they disagree (docs/vite.md). Only what the source states is checked: a prop's name
// always, its type where TypeScript gives one; a component whose props can't be read isn't checked.

import { process as processDocument, type DeclaredType, type Definition, type Signature, type SignatureFence } from "@compmd/processor";
import { localComponents, type ComponentFile, type Pack } from "./packs.ts";
import { reactProps, svelteProps, type Compilers, type PropType, type Props, type SvelteCompiler } from "./props.ts";

/** Where in a document, 1-based. */
interface Place {
  line: number;
  column: number;
}

/** A way a component disagrees with its signature. */
export interface Mismatch {
  message: string;
  /** The declared param it is about, which the warning is placed at; the fence when there is none. */
  param?: string;
}

/** A host-language component a pack's document gives a signature fence to, with its source. */
interface Analysed {
  name: string;
  /** Its file in the pack, or the pack's document for a definition fence. */
  file: string;
  language: ComponentFile["language"];
  source: string;
}

/** A param's or a slot's prop in the React and Svelte bindings: camelCase, and `children` for the default slot (spec §7.6). */
const propName = (name: string): string => name.replace(/-([a-z0-9])/g, (_, letter: string) => letter.toUpperCase());
const slotProp = (slot: string): string => (slot === "default" ? "children" : propName(slot));

/** The binding's own prop, which no signature declares (spec §7.6). */
const CONTEXT_PROP = "compmd";

const STRING_TYPES = new Set(["string", "length", "space", "color", "tracks", "areas", "surface", "border", "radius", "name"]);

/** A declared type as a prop's type, when there is one to compare with: `input<…>` and others are passed unchecked. */
function expected(declared: DeclaredType): PropType | undefined {
  if (typeof declared === "string") {
    if (STRING_TYPES.has(declared)) return { kind: "string" };
    return declared === "number" || declared === "boolean" ? { kind: declared } : undefined;
  }
  return "enum" in declared ? { kind: "enum", members: declared.enum } : undefined;
}

/** A type as messages write it. */
const written = (type: PropType): string => (type.kind === "enum" ? `one of ${type.members.join(", ")}` : `a ${type.kind}`);

/** Whether a prop of type `actual` takes every value of the declared `wanted`. */
function fits(wanted: PropType, actual: PropType): boolean {
  if (actual.kind === "string") return wanted.kind === "string" || wanted.kind === "enum";
  if (actual.kind === "enum") return wanted.kind === "enum" && wanted.members.every((member) => actual.members.includes(member));
  return wanted.kind === actual.kind;
}

/**
 * How `props`, what the component in `file` takes, disagrees with its `signature`: a declared param or slot it has no
 * prop for, a prop whose type doesn't take the declared one, and a prop the signature doesn't declare.
 */
export function mismatches(file: string, signature: Signature, { props, rest }: Props): Mismatch[] {
  const found: Mismatch[] = [];
  const byName = new Map(props.map((prop) => [prop.name, prop]));
  const declared = new Set<string>([CONTEXT_PROP]);
  for (const param of signature.params) {
    const name = propName(param.name);
    declared.add(name);
    const prop = byName.get(name);
    if (!prop) {
      if (!rest) found.push({ param: param.name, message: `the signature declares \`${param.name}\`, which ${file} has no prop for (\`${name}\`)` });
      continue;
    }
    const wanted = expected(param.type);
    if (wanted && prop.type && !fits(wanted, prop.type)) found.push({ param: param.name, message: `the signature declares \`${param.name}\` as ${written(wanted)}, which ${file}'s prop takes as ${written(prop.type)}` });
  }
  for (const slot of signature.slots ?? []) {
    const name = typeof slot === "string" ? slot : slot.name;
    const prop = slotProp(name);
    declared.add(prop);
    if (!byName.has(prop) && !rest) found.push({ message: `the signature declares the slot \`${name}\`, which ${file} has no prop for (\`${prop}\`)` });
  }
  for (const prop of props) {
    if (!declared.has(prop.name)) found.push({ message: `${file} has the prop \`${prop.name}\`, which the signature doesn't declare` });
  }
  return found;
}

/** Where `param` is declared inside the fence at `fence`, in `lines`; the fence when it can't be found. */
function placeOf(lines: readonly string[], fence: NonNullable<SignatureFence["position"]>, param: string | undefined): Place {
  const declaration = new RegExp(`^(\\s+)${param?.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") ?? ""}\\s*:`);
  for (let line = fence.start.line + 1; param !== undefined && line < fence.end.line; line++) {
    const indent = declaration.exec(lines[line - 1] ?? "")?.[1];
    if (indent !== undefined) return { line, column: indent.length + 1 };
  }
  return { line: fence.start.line, column: fence.start.column };
}

const HOST_LANGUAGES = new Set<string>(["jsx", "tsx", "svelte"]);

/** What a component of `language` takes, from its source; `undefined` for a binding without props or a source that can't be read. */
function propsOf(compilers: Compilers, { language, source }: Analysed): Props | undefined {
  if (language === "jsx" || language === "tsx") return reactProps(compilers.ts, source, language);
  return language === "svelte" ? svelteProps(compilers, source) : undefined;
}

/**
 * The W104 warnings of `pack`, whose document's frontmatter is `md`, as lines of the build's output. The processor's
 * module must be loaded.
 */
export function analysePack(pack: Pack, md: Record<string, unknown>, compilers: Compilers): string[] {
  const text = pack.files[pack.document] ?? "";
  const { parse } = processDocument({ path: pack.document, files: pack.files, now: new Date(0).toISOString() });
  const local = localComponents(pack, md);
  const defined = new Map<string, Analysed>();
  for (const node of parse.children) {
    if (node.type !== "componentDefinition") continue;
    const definition = node as Definition;
    if (definition.source !== undefined && HOST_LANGUAGES.has(definition.language)) {
      defined.set(definition.name, { name: definition.name, file: pack.document, language: definition.language as ComponentFile["language"], source: definition.source });
    }
  }
  const lines = text.split(/\r?\n/);
  const warnings: string[] = [];
  for (const node of parse.children) {
    if (node.type !== "componentSignature") continue;
    const fence = node as SignatureFence;
    if (!fence.signature || !fence.position) continue;
    const file = local.get(fence.name);
    const component = defined.get(fence.name) ?? (file && { name: fence.name, file: file.path, language: file.language, source: pack.files[file.path] ?? "" });
    const props = component && propsOf(compilers, component);
    if (!component || !props) continue;
    for (const { param, message } of mismatches(component.file, fence.signature, props)) {
      const { line, column } = placeOf(lines, fence.position, param);
      warnings.push(`${pack.name}@${pack.version}/${pack.document}:${String(line)}:${String(column)}: W104 ${message}`);
    }
  }
  return warnings;
}

/** The compilers the analysis needs, when the app has them: TypeScript, and Svelte's for Svelte components. */
export async function loadCompilers(): Promise<Compilers | undefined> {
  let ts: Compilers["ts"];
  try {
    ts = (await import("typescript")).default;
  } catch {
    return undefined;
  }
  try {
    return { ts, svelte: (await import("svelte/compiler")) as unknown as SvelteCompiler };
  } catch {
    return { ts };
  }
}
