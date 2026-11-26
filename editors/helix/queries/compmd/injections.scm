; The grammar's injections (syntax/injections.json), with Helix's properties.

([
  (inline)
  (pipe_table_cell)
] @injection.content
  (#set! injection.language "compmd-inline")
  (#set! injection.include-unnamed-children))

((minus_metadata) @injection.content
  (#set! injection.language "yaml")
  (#set! injection.include-unnamed-children))

((plus_metadata) @injection.content
  (#set! injection.language "toml")
  (#set! injection.include-unnamed-children))

((expression_code) @injection.content
  (#set! injection.language "cel"))

((json) @injection.content
  (#set! injection.language "json"))

((definition_signature) @injection.content
  (#set! injection.language "yaml"))

((definition_body) @injection.content
  (#set! injection.language "compmd"))

(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content
  (#set! injection.include-unnamed-children))

((html_block) @injection.content
  (#set! injection.language "html")
  (#set! injection.include-unnamed-children)
  (#set! injection.combined))
