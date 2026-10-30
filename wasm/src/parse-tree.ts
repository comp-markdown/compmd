// The parse tree (spec §12.1, §12.2) as JSON holds it (§12.4), as `compmd-tree.schema.json` describes it.

/** A place in the document: lines and columns 1-based, the offset 0-based; columns and offsets count code points. */
export interface Point {
  line: number;
  column: number;
  offset?: number;
}

export interface Position {
  start: Point;
  end: Point;
}

/** A param as written. */
export interface Param {
  key: string | null;
  breakpoint: string | null;
  kind: "bare" | "string" | "json" | "expr" | "flag" | "class" | "id";
  value: string | null | StringPart[];
  position?: Position;
}

/** A `{{path}}`, inline or on a line of its own (`block`). */
export interface Interpolation {
  type: "interpolation";
  path: string;
  block: boolean;
  position?: Position;
}

/** A part of a quoted param, a URL or a data `from`. */
export type StringPart = { type: "text"; value: string; position?: Position } | Interpolation;

export interface ParseContainer {
  type: "componentContainer";
  name: string;
  params: Param[];
  children: ParseNode[];
  slots: ParseSlot[];
  position?: Position;
  closePosition?: Position;
  autoClosed?: true;
  recovered?: true;
}

export interface ParseSlot {
  type: "componentSlot";
  name: string;
  children: ParseNode[];
  position?: Position;
}

export interface ParseLeaf {
  type: "componentLeaf" | "componentInline";
  name: string;
  params: Param[];
  fallback: ParseNode[];
  position?: Position;
  recovered?: true;
}

/** A declared param of a signature. */
export interface Declaration {
  name: string;
  type: DeclaredType;
  optional: boolean;
  default: { kind: "bare" | "string" | "json" | "expr" | "flag" | "yaml"; value: unknown } | null;
  doc: string | null;
}

export type DeclaredType = string | { enum: string[] } | { input: DeclaredType };

export interface Signature {
  params: Declaration[];
  /** `null` when the component takes no slots; a template slot is `{name, each, as}`. */
  slots: (string | { name: string; each: string; as: string })[] | null;
  headings: "relative" | "absolute";
  /** A code renderer's languages, names of one (spec §7.7); its `params` then include `code`, the block's text. */
  renders?: string[];
}

/** A definition fence: `children` for `compmd`, `source` and `runtime` for host languages. */
export interface Definition {
  type: "componentDefinition";
  name: string;
  language: string;
  signature: Signature | null;
  params: Param[];
  children?: ParseNode[];
  source?: string;
  runtime?: string | null;
  position?: Position;
}

export interface SignatureFence {
  type: "componentSignature";
  name: string;
  signature: Signature | null;
  position?: Position;
}

/** The frontmatter. */
export interface Yaml {
  type: "yaml";
  value: string;
  parsed?: Record<string, unknown>;
  position?: Position;
}

/** Any other mdast node (https://github.com/syntax-tree/mdast); one made by a recovery rule is `recovered`. */
export interface MdastParseNode {
  type: string;
  children?: ParseNode[];
  /** A link's or an image's URL with interpolations. */
  urlParts?: StringPart[];
  recovered?: true;
  position?: Position;
  [field: string]: unknown;
}

export type ParseNode =
  | ParseContainer
  | ParseSlot
  | ParseLeaf
  | Definition
  | SignatureFence
  | Interpolation
  | Yaml
  | MdastParseNode;

export interface ParseRoot {
  type: "root";
  children: ParseNode[];
  position?: Position;
}
