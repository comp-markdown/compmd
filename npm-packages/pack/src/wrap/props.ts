// A wrapped component's props, read from its package's TypeScript types with the checker (docs/wrapping-an-npm-package.md,
// decision 4): only those the package's own files declare, never React's, Svelte's or the DOM's (decision 5), each with
// its type as the mapping reads it, whether it is optional, its doc and its JSDoc `@default`.

import { dirname, join } from "node:path";
import ts from "typescript";
import { PackError } from "../pack.ts";
import type { Wrap } from "./config.ts";

/** A prop's type, as much of it as the mapping to a param reads (decision 6). */
export type PropType =
  | { kind: "string" | "number" | "boolean" | "date" | "array" | "object" | "function" | "color" }
  /** A union of string literals, `"L" | "M"`. */
  | { kind: "literals"; values: string[] }
  /** A union of other kinds: `string | ((d) => any)`. */
  | { kind: "union"; branches: PropType[] }
  /** Anything else, as TypeScript writes it. */
  | { kind: "other"; text: string };

/** A prop the package declares. */
export interface Prop {
  name: string;
  type: PropType;
  optional: boolean;
  doc?: string;
  /** Its JSDoc `@default`, as written: the only default the build trusts (decision 6). */
  default?: string;
}

/** The file the checker reads the props from, beside the pack's own: it never reaches the disk. */
const ENTRY = "compmd-wrap-props.ts";

/** What imports the component, and names its props `Props`. */
export function entryOf(wrap: Wrap): string {
  const from = JSON.stringify(wrap.from);
  if (wrap.kind === "web-component") return `import ${from};\nexport type Props = HTMLElementTagNameMap[${JSON.stringify(wrap.export)}];\n`;
  const runtime = wrap.kind === "svelte" ? "svelte" : "react";
  const imported = wrap.export === "default" ? "Component" : `{ ${wrap.export} as Component }`;
  return `import type { ComponentProps } from "${runtime}";\nimport ${imported} from ${from};\nexport type Props = ComponentProps<typeof Component>;\n`;
}

/** The folders whose declarations are the package's own: its own, and its `@types` package's. */
function ownFolders(name: string): string[] {
  const types = name.startsWith("@") ? `@types/${name.slice(1).replace("/", "__")}` : `@types/${name}`;
  return [`/node_modules/${name}/`, `/node_modules/${types}/`];
}

const OPTIONS: ts.CompilerOptions = {
  target: ts.ScriptTarget.ES2022,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  jsx: ts.JsxEmit.ReactJSX,
  lib: ["lib.es2023.d.ts", "lib.dom.d.ts"],
  strict: true,
  skipLibCheck: true,
  noEmit: true,
  types: [],
};

/** `type` without `undefined`, `null` and `void`, which only say it is optional. */
function defined(type: ts.Type): ts.Type[] {
  const parts = type.isUnion() ? type.types : [type];
  return parts.filter((part) => (part.flags & (ts.TypeFlags.Undefined | ts.TypeFlags.Null | ts.TypeFlags.Void)) === 0);
}

/**
 * Whether a prop is declared with a colour type: CSS's (`CSSProperties["color"]`, `Property.Color`), as written, since
 * the alias's name is gone from the type once `undefined` joins it.
 */
function isColour(symbol: ts.Symbol): boolean {
  return (symbol.declarations ?? []).some((declaration) => {
    const written = (ts.isPropertySignature(declaration) || ts.isPropertyDeclaration(declaration)) && declaration.type ? declaration.type.getText() : "";
    return /\bColor\b|\["\w*[cC]olor"\]|Color$/.test(written);
  });
}

function typeOf(type: ts.Type, checker: ts.TypeChecker): PropType {
  const parts = defined(type);
  if (parts.length === 0) return { kind: "other", text: checker.typeToString(type) };
  if (parts.every((part) => part.flags & ts.TypeFlags.BooleanLiteral)) return { kind: "boolean" };
  if (parts.every((part) => part.isStringLiteral())) return { kind: "literals", values: parts.map((part) => (part.isStringLiteral() ? part.value : "")) };
  if (parts.length > 1) {
    // `boolean` is `true | false` inside a union too: one branch, not two.
    const booleans = parts.filter((part) => part.flags & ts.TypeFlags.BooleanLiteral);
    const rest = parts.filter((part) => !(part.flags & ts.TypeFlags.BooleanLiteral)).map((part) => typeOf(part, checker));
    return { kind: "union", branches: booleans.length > 0 ? [{ kind: "boolean" }, ...rest] : rest };
  }
  const [only] = parts as [ts.Type];
  if (only.flags & (ts.TypeFlags.String | ts.TypeFlags.TemplateLiteral)) return { kind: "string" };
  if (only.flags & (ts.TypeFlags.Number | ts.TypeFlags.NumberLiteral)) return { kind: "number" };
  if (only.flags & (ts.TypeFlags.Boolean | ts.TypeFlags.BooleanLiteral)) return { kind: "boolean" };
  if (only.getCallSignatures().length > 0) return { kind: "function" };
  if ((only.symbol as ts.Symbol | undefined)?.name === "Date") return { kind: "date" };
  if (checker.isArrayType(only) || checker.isTupleType(only)) return { kind: "array" };
  if (only.flags & ts.TypeFlags.Object) return { kind: "object" };
  return { kind: "other", text: checker.typeToString(only) };
}

/** The `@default` of `symbol`'s JSDoc, as written: from its declarations, which a generic's instance keeps. */
function defaultOf(symbol: ts.Symbol): string | undefined {
  const tags = (symbol.declarations ?? []).flatMap((declaration) => ts.getJSDocTags(declaration));
  const tag = tags.find((found) => found.tagName.text === "default" || found.tagName.text === "defaultValue");
  const text = typeof tag?.comment === "string" ? tag.comment : tag?.comment === undefined ? undefined : ts.getTextOfJSDocComment(tag.comment);
  return text === undefined || text.trim() === "" ? undefined : text.trim();
}

/**
 * The props of the component `wrap` names, in the pack in `dir`, where its package is installed, in the order the
 * package declares them.
 *
 * @throws PackError when the checker can't find the component, saying what TypeScript said.
 */
/** A program of the one file `text`, in the pack in `dir`, which the checker reads with the pack's packages. */
function programOf(dir: string, text: string): { program: ts.Program; source: ts.SourceFile | undefined } {
  const entry = join(dir, ENTRY);
  const host = ts.createCompilerHost(OPTIONS);
  const readFile = host.readFile.bind(host);
  const getSourceFile = host.getSourceFile.bind(host);
  host.readFile = (file) => (file === entry ? text : readFile(file));
  host.fileExists = (file) => file === entry || ts.sys.fileExists(file);
  host.getSourceFile = (file, language) => (file === entry ? ts.createSourceFile(file, text, language) : getSourceFile(file, language));
  host.getCurrentDirectory = () => dir;
  host.getDefaultLibLocation = () => dirname(ts.getDefaultLibFilePath(OPTIONS));
  const program = ts.createProgram([entry], OPTIONS, host);
  return { program, source: program.getSourceFile(entry) };
}

/** Whether `symbol` is declared in one of the package's own files. */
const declaredIn = (symbol: ts.Symbol, own: readonly string[]): boolean =>
  (symbol.declarations ?? []).some((declaration) => own.some((folder) => declaration.getSourceFile().fileName.includes(folder)));

/**
 * The custom elements the package `from` defines, by their tags, from its types (`HTMLElementTagNameMap`): a web
 * component's (decision 2).
 *
 * @example
 * elementTags("viewer", "@google/model-viewer"); // ["model-viewer"]
 */
export function elementTags(dir: string, from: string): string[] {
  const { program, source } = programOf(dir, `import ${JSON.stringify(from)};\nexport type Tags = HTMLElementTagNameMap;\n`);
  const checker = program.getTypeChecker();
  const alias = source?.statements.find(ts.isTypeAliasDeclaration);
  if (!alias) return [];
  return checker
    .getPropertiesOfType(checker.getTypeAtLocation(alias))
    .filter((tag) => declaredIn(tag, ownFolders(from)))
    .map((tag) => tag.name);
}

export function readProps(dir: string, wrap: Wrap): Prop[] {
  const { program, source } = programOf(dir, entryOf(wrap));
  const checker = program.getTypeChecker();
  const alias = source?.statements.find(ts.isTypeAliasDeclaration);
  const problems = ts.getPreEmitDiagnostics(program, source).map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, " "));
  if (!alias || problems.length > 0) throw new PackError(`the types of ${wrap.from}'s ${wrap.export} can't be read: ${problems.join("; ") || "no props type"}`);
  const props = checker.getTypeAtLocation(alias);
  const own = ownFolders(wrap.from);
  return checker
    .getPropertiesOfType(props)
    .filter((symbol) => declaredIn(symbol, own))
    .map((symbol): Prop => {
      const doc = ts.displayPartsToString(symbol.getDocumentationComment(checker)).trim();
      const found = defaultOf(symbol);
      return {
        name: symbol.name,
        type: isColour(symbol) ? { kind: "color" } : typeOf(checker.getTypeOfSymbolAtLocation(symbol, alias), checker),
        optional: (symbol.flags & ts.SymbolFlags.Optional) !== 0,
        ...(doc ? { doc } : {}),
        ...(found === undefined ? {} : { default: found }),
      };
    });
}
