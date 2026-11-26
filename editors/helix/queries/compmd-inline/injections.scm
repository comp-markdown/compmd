; The grammar's injections (syntax/injections.json), with Helix's properties.

((expression_code) @injection.content
  (#set! injection.language "cel"))

((json) @injection.content
  (#set! injection.language "json"))

((html_tag) @injection.content
  (#set! injection.language "html")
  (#set! injection.include-unnamed-children)
  (#set! injection.combined))

((latex_content) @injection.content
  (#set! injection.language "latex"))
