// CompMD's syntax as a TextMate grammar Shiki doesn't bundle, injected into Markdown's, so every Markdown example shows
// it; and where a Markdown example's YAML ends. Shiki loses the injection when YAML's grammar is loaded beside
// Markdown's, so a highlighter highlights the YAML (a document's frontmatter, a `compmd` definition's signature) with
// one Shiki and the rest with another. The sandbox's highlighter and the website's spec page share both.
import type { LanguageRegistration } from "@shikijs/core";

/** The parts of a param list, `{#id .class key=value flag key=(expression) key="text"}`. */
const params = {
  begin: "\\{(?!\\{)",
  end: "\\}",
  beginCaptures: { 0: { name: "punctuation.definition.tag.compmd" } },
  endCaptures: { 0: { name: "punctuation.definition.tag.compmd" } },
  patterns: [
    { match: '"(?:[^"\\\\]|\\\\.)*"', name: "string.quoted.double.compmd" },
    { begin: "\\(", end: "\\)", name: "meta.embedded.expression.compmd", patterns: [{ include: "#parens" }, { match: '"(?:[^"\\\\]|\\\\.)*"', name: "string.quoted.double.compmd" }] },
    { match: "[#.][\\w-]+", name: "entity.other.attribute-name.compmd" },
    { match: "[\\w:-]+(?==)", name: "entity.other.attribute-name.compmd" },
    { match: "=", name: "punctuation.separator.key-value.compmd" },
    // A bare value, `gap=m`: after `=`, to the next space or the list's end.
    { match: "(?<==)[^\\s}\"(\\[{]+", name: "string.unquoted.compmd" },
  ],
};

/** CompMD's syntax, in Markdown text: component lines, inline components, param lists and `{{paths}}`. */
export const COMPMD: LanguageRegistration = {
  name: "compmd-syntax",
  scopeName: "markdown.compmd.injection",
  injectTo: ["text.html.markdown"],
  injectionSelector: "L:text.html.markdown -markup.fenced_code.block.markdown -markup.inline.raw.string.markdown",
  patterns: [
    {
      // `::: box {…}`, `:: #slot`, `::: /box`, `::: + more=params`
      // At a line's start, or where a list item's or quote's prefix ends.
      begin: "(?:^|\\G)[ \\t]{0,3}(:{2,})[ \\t]*(\\+|/?[A-Za-z][\\w-]*|#[\\w-]+)?",
      end: "$",
      beginCaptures: { 1: { name: "punctuation.definition.tag.compmd" }, 2: { name: "entity.name.tag.compmd" } },
      patterns: [{ match: "\\[[^\\]]*\\]", name: "markup.other.fallback.compmd" }, params, { match: "[\\w:-]+(?==)", name: "entity.other.attribute-name.compmd" }],
    },
    {
      // `:value[…]{…}`
      match: "(?<![\\w:]):([A-Za-z][\\w-]*)(?=[\\[{])",
      captures: { 0: { name: "punctuation.definition.tag.compmd" }, 1: { name: "entity.name.tag.compmd" } },
    },
    // `constant`, which GitHub's themes color, where `variable.other` stays the text's.
    { match: "\\{\\{[^}]*\\}\\}", name: "constant.other.placeholder.compmd" },
    params,
  ],
  repository: {
    parens: { begin: "\\(", end: "\\)", patterns: [{ include: "#parens" }] },
  },
};

/**
 * `code` split after the line `---` that ends its YAML: frontmatter between two (`frontmatter`, a Markdown document),
 * or a signature before one (a `compmd` definition).
 *
 * @example
 * splitYaml("---\nmd:\n  version: 1\n---\n# Hi", true); // ["---\nmd:\n  version: 1\n---", "# Hi"]
 */
export function splitYaml(code: string, frontmatter: boolean): [yaml: string, rest: string] {
  const lines = code.split("\n");
  const end = frontmatter ? (lines[0] === "---" ? lines.indexOf("---", 1) : -1) : lines.indexOf("---");
  return end === -1 ? (frontmatter ? ["", code] : [code, ""]) : [lines.slice(0, end + 1).join("\n"), lines.slice(end + 1).join("\n")];
}
