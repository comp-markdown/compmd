; inherits: markdown

; CompMD's, with Helix's names for the captures of syntax/tokens.json.

(frontmatter_key) @keyword.directive

(component_marker) @punctuation.special

(container_closer
  "/" @punctuation.special)

(component_slot
  "#" @punctuation.special)

(param_continuation
  "+" @punctuation.special)

(component_name) @tag

(slot_name) @label

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

; A definition's name, `card` of `component=card`, is a component's name, not a bare value.
((info_string
  (param
    key: (param_key
      name: (param_name) @_key)
    value: (bare_value) @tag))
  (#eq? @_key "component"))

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

(definition_separator) @punctuation.delimiter
