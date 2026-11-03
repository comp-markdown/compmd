// A TextMate grammar Shiki doesn't bundle: ABNF, which the spec's grammar is written in. CompMD's own syntax is the
// sandbox's (`@compmd/sandbox/grammar`), shared with the pages' highlighter.
import type { LanguageRegistration } from "shiki";

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
