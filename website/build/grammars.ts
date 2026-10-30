// TextMate grammars Shiki doesn't bundle: CompMD's syntax, injected into Markdown so every Markdown example shows it,
// and ABNF, which the spec's grammar is written in.
import type { LanguageRegistration } from "shiki";

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

/** ABNF (RFC 5234, with RFC 7405's `%s` and `%i`). */
export const ABNF: LanguageRegistration = {
  name: "abnf",
  scopeName: "source.abnf",
  patterns: [
    { match: ";.*$", name: "comment.line.semicolon.abnf" },
    { match: "^[A-Za-z][\\w-]*(?=\\s*=/?)", name: "entity.name.function.abnf" },
    { match: "=/?", name: "keyword.operator.definition.abnf" },
    { match: '%[si]?"[^"]*"|"[^"]*"', name: "string.quoted.double.abnf" },
    { match: "%[bdx][0-9A-Fa-f]+(?:(?:\\.[0-9A-Fa-f]+)+|-[0-9A-Fa-f]+)?", name: "constant.numeric.abnf" },
    { match: "<[^>]*>", name: "string.other.prose.abnf" },
    { match: "[A-Za-z][\\w-]*", name: "variable.other.rule.abnf" },
    { match: "\\d*\\*\\d*", name: "keyword.operator.repetition.abnf" },
    { match: "[/()\\[\\]]", name: "punctuation.abnf" },
  ],
  repository: {},
};
