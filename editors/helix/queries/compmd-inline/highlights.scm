; Markdown's inlines, as Helix's queries for tree-sitter-markdown's inline grammar highlight them. They are written out:
; Helix's `inherits` can't name `markdown.inline`.

[
  (code_span)
  (link_title)
] @markup.raw.inline

[
  (emphasis_delimiter)
  (code_span_delimiter)
] @punctuation.bracket

(emphasis) @markup.italic

(strong_emphasis) @markup.bold

(strikethrough) @markup.strikethrough

[
  (link_destination)
  (uri_autolink)
] @markup.link.url

[
  (link_text)
  (image_description)
] @markup.link.text

(link_label) @markup.link.label

[
  (backslash_escape)
  (hard_line_break)
] @constant.character.escape

(image ["[" "]" "(" ")"] @punctuation.bracket)
(image "!" @punctuation.special)
(inline_link ["[" "]" "(" ")"] @punctuation.bracket)
(shortcut_link ["[" "]"] @punctuation.bracket)

; CompMD's, with Helix's names for the captures of syntax/tokens.json.

(inline_component
  (component_marker) @punctuation.special)

(component_name) @tag

(fallback
  [
    "["
    "]"
  ] @punctuation.bracket)

(param_list
  [
    "{"
    "}"
  ] @punctuation.bracket)

(param_key
  (param_name) @attribute)

(breakpoint) @attribute.builtin

(param
  [
    "."
    (class_name)
  ] @attribute)

(param
  [
    "#"
    (id_name)
  ] @attribute)

(param
  "=" @operator)

(param_key
  ":" @operator)

(string) @string

(string_escape) @constant.character.escape

(bare_value) @string.special

(expression
  [
    "("
    ")"
  ] @punctuation.special)

(interpolation
  [
    "{{"
    "}}"
  ] @punctuation.special)

(interpolation_path) @variable

(wiki_link
  (link_destination) @markup.link.url)

(wiki_link
  (link_text) @markup.link.text)
