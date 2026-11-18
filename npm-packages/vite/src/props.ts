// What a host-language component takes, read from its source for component analysis (spec §7.1, W104): a React
// component's props from its default export's first parameter, a Svelte component's from `$props()`, each with its
// type where TypeScript states one. The TypeScript and Svelte compilers are the app's own, given by the caller.

import type TS from "typescript";

/** A prop's type, where the source states one that a signature's type can be checked against. */
export type PropType = { kind: "string" | "number" | "boolean" | "slot" } | { kind: "enum"; members: string[] };

/** A prop a component takes. */
export interface Prop {
  name: string;
  /** Its type, when the source states one this check knows. */
  type?: PropType;
}

/** What a component takes. */
export interface Props {
  props: Prop[];
  /** Whether it also takes props it doesn't name (`...rest` with no type), so any param fits. */
  rest: boolean;
}

/** The part of Svelte's compiler this reads: its modern AST's instance script, where `$props()` is. */
export interface SvelteCompiler {
  parse(source: string, options: { modern: true }): { instance: { content: unknown } | null };
}

/** The compilers the analysis reads sources with: TypeScript for both, Svelte's for a Svelte component's script. */
export interface Compilers {
  ts: typeof TS;
  svelte?: SvelteCompiler;
}

/** The type names that are a slot's: the node or the snippet a binding gives for one (spec §7.6). */
const SLOT_TYPES = new Set(["ReactNode", "ReactElement", "JSX.Element", "Snippet"]);

/** The type `node` states, when it is one a signature's type can be checked against. */
function typeOf(ts: typeof TS, node: TS.TypeNode): PropType | undefined {
  if (node.kind === ts.SyntaxKind.StringKeyword) return { kind: "string" };
  if (node.kind === ts.SyntaxKind.NumberKeyword) return { kind: "number" };
  if (node.kind === ts.SyntaxKind.BooleanKeyword) return { kind: "boolean" };
  if (ts.isParenthesizedTypeNode(node)) return typeOf(ts, node.type);
  if (ts.isTypeReferenceNode(node)) return SLOT_TYPES.has(node.typeName.getText().replace(/^React\./, "")) ? { kind: "slot" } : undefined;
  if (ts.isLiteralTypeNode(node) && ts.isStringLiteral(node.literal)) return { kind: "enum", members: [node.literal.text] };
  if (!ts.isUnionTypeNode(node)) return undefined;
  const isNothing = (member: TS.TypeNode) => member.kind === ts.SyntaxKind.UndefinedKeyword || (ts.isLiteralTypeNode(member) && member.literal.kind === ts.SyntaxKind.NullKeyword);
  const members = node.types.filter((member) => !isNothing(member));
  if (members.length === 1 && members[0]) return typeOf(ts, members[0]);
  const literals = members.map((member) => (ts.isLiteralTypeNode(member) ? member.literal : undefined));
  if (literals.every((literal) => literal !== undefined && ts.isStringLiteral(literal))) return { kind: "enum", members: literals.map((literal) => literal.text) };
  if (literals.every((literal) => literal?.kind === ts.SyntaxKind.TrueKeyword || literal?.kind === ts.SyntaxKind.FalseKeyword)) return { kind: "boolean" };
  return undefined;
}

/** The props a type node lists: a type literal, an interface or a type alias of the file, or an intersection of them. */
function membersOf(ts: typeof TS, file: TS.SourceFile, node: TS.TypeNode): Prop[] | undefined {
  if (ts.isTypeLiteralNode(node)) return propsOfMembers(ts, node.members);
  if (ts.isParenthesizedTypeNode(node)) return membersOf(ts, file, node.type);
  if (ts.isIntersectionTypeNode(node)) {
    const parts = node.types.map((part) => membersOf(ts, file, part));
    return parts.every((part) => part !== undefined) ? parts.flat() : undefined;
  }
  if (!ts.isTypeReferenceNode(node) || !ts.isIdentifier(node.typeName) || node.typeArguments) return undefined;
  const name = node.typeName.text;
  for (const statement of file.statements) {
    if (ts.isInterfaceDeclaration(statement) && statement.name.text === name && !statement.heritageClauses && !statement.typeParameters) return propsOfMembers(ts, statement.members);
    if (ts.isTypeAliasDeclaration(statement) && statement.name.text === name && !statement.typeParameters) return membersOf(ts, file, statement.type);
  }
  return undefined;
}

/** The props of a type's members: its properties, by name. */
function propsOfMembers(ts: typeof TS, members: TS.NodeArray<TS.TypeElement>): Prop[] | undefined {
  const props: Prop[] = [];
  for (const member of members) {
    if (!ts.isPropertySignature(member) || !(ts.isIdentifier(member.name) || ts.isStringLiteral(member.name))) return undefined;
    const type = member.type && typeOf(ts, member.type);
    props.push(type ? { name: member.name.text, type } : { name: member.name.text });
  }
  return props;
}

/**
 * What a binding of props takes: a component's first parameter, or the variable `$props()` is given to. A type the
 * file states lists them all; without one, the names destructured are the props, and a `...rest` takes any others.
 */
function propsOfBinding(ts: typeof TS, file: TS.SourceFile, name: TS.BindingName, type: TS.TypeNode | undefined): Props | undefined {
  const typed = type && membersOf(ts, file, type);
  if (typed) return { props: typed, rest: false };
  if (!ts.isObjectBindingPattern(name)) return undefined;
  const props: Prop[] = [];
  let rest = false;
  for (const element of name.elements) {
    if (element.dotDotDotToken) rest = true;
    else {
      const key = element.propertyName ?? element.name;
      if (!ts.isIdentifier(key) && !ts.isStringLiteral(key)) return undefined;
      props.push({ name: key.text });
    }
  }
  return { props, rest };
}

/** The function a file exports as its default: a declaration, an expression, or a function the file names. */
function defaultExport(ts: typeof TS, file: TS.SourceFile): TS.SignatureDeclaration | undefined {
  const isDefault = (statement: TS.Statement) => ts.getModifiers(statement as TS.HasModifiers)?.some((modifier) => modifier.kind === ts.SyntaxKind.DefaultKeyword) === true;
  const named = (name: string): TS.SignatureDeclaration | undefined => {
    for (const statement of file.statements) {
      if (ts.isFunctionDeclaration(statement) && statement.name?.text === name) return statement;
      if (!ts.isVariableStatement(statement)) continue;
      for (const declaration of statement.declarationList.declarations) {
        if (ts.isIdentifier(declaration.name) && declaration.name.text === name && declaration.initializer) return functionOf(declaration.initializer);
      }
    }
    return undefined;
  };
  const functionOf = (expression: TS.Expression): TS.SignatureDeclaration | undefined => {
    if (ts.isParenthesizedExpression(expression)) return functionOf(expression.expression);
    if (ts.isArrowFunction(expression) || ts.isFunctionExpression(expression)) return expression;
    return ts.isIdentifier(expression) ? named(expression.text) : undefined;
  };
  for (const statement of file.statements) {
    if (ts.isFunctionDeclaration(statement) && isDefault(statement)) return statement;
    if (ts.isExportAssignment(statement) && !statement.isExportEquals) return functionOf(statement.expression);
  }
  return undefined;
}

/** What a React component (`jsx`, `tsx`) takes: its default export's first parameter; `undefined` when it can't be read. */
export function reactProps(ts: typeof TS, source: string, language: "jsx" | "tsx"): Props | undefined {
  const file = ts.createSourceFile(`component.${language}`, source, ts.ScriptTarget.Latest, true, language === "tsx" ? ts.ScriptKind.TSX : ts.ScriptKind.JSX);
  const component = defaultExport(ts, file);
  if (!component) return undefined;
  const [param] = component.parameters;
  return param ? propsOfBinding(ts, file, param.name, param.type) : { props: [], rest: false };
}

/** What a Svelte 5 component takes: what its instance script gives `$props()`; `undefined` when it can't be read. */
export function svelteProps({ ts, svelte }: Compilers, source: string): Props | undefined {
  if (!svelte) return undefined;
  let content: { start?: unknown; end?: unknown } | undefined;
  try {
    content = svelte.parse(source, { modern: true }).instance?.content as typeof content;
  } catch {
    return undefined;
  }
  if (!content) return { props: [], rest: false };
  if (typeof content.start !== "number" || typeof content.end !== "number") return undefined;
  const file = ts.createSourceFile("component.ts", source.slice(content.start, content.end), ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  for (const statement of file.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    for (const declaration of statement.declarationList.declarations) {
      const call = declaration.initializer;
      if (call && ts.isCallExpression(call) && ts.isIdentifier(call.expression) && call.expression.text === "$props") return propsOfBinding(ts, file, declaration.name, declaration.type);
    }
  }
  return { props: [], rest: false };
}
