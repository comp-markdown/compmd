# CompMD conformance set

Test cases for implementations of the [CompMD spec](../compmd-spec-v1.md). The format is in the spec's §13.2: one folder per case, with `case.json` (what the case needs), `input.comp.md`, and the expected tree and diagnostics.

| Area | What it covers |
|---|---|
| `base-syntax/` | Alerts, math by Pandoc's rule, wiki links when opted in (W111 for unknown names), footnotes in definition bodies, component lines in HTML blocks (W116) |
| `literal/` | Text that looks like CompMD syntax but isn't: escapes, code, colons in words, TOML |
| `frontmatter/` | The `md` key, author keys, YAML rules (F033), versions (F001, W110, W111), host metadata (W115), `from` parts |
| `containers/` | Openers, closers, slots, nesting with lists and quotes, and their recovery (F003–F005, F007, F015, F032, W108) |
| `leaves/` | Leaves, fallbacks, continuation lines (W101, F012) |
| `inline/` | Inline components in text, headings and tables (W107) |
| `params/` | Every param kind, breakpoint prefixes, classes and ids, escapes, Unicode, and broken params (F011, F017) |
| `lines/` | Malformed component lines (F016) |
| `interpolation/` | `{{ }}` in text, blocks, links and definitions (W106) |
| `definitions/` | Component definitions and signature fences: types, `input<…>`, template slots, `headings`, `runtime`, and the fence's param list for the component's box (F011, F017, F036 on a signature fence) |
| `resolve/` | Names through local definitions, imports, a stubbed registry (`context.json` `registry`), ambient packs and `@compmd/core`: F002, F013, F019–F023, F026, F029, F035, F037 (in signatures, inputs and schemas), W102, W104 (component analysis), W105, W112, W113, W117 |
| `validate/` | Params, common params and layout values, slots, interpolation paths, inputs and data sources, and `display=contents`: F006, F008–F010, F013, F014, F018, F028, F030, F031, W103, W120 |
| `expand/` | Stages 4a and 4b: definitions with params and slots, relative headings, `if`, `each` and its join modes, `value`, how values are written, URLs (W109), template slots, layout boxes and breakpoints, the component's box (the fence's params with the caller's over them, `display=contents`, host-language components' fences), input-bound params and the root's inputs, data files, remote and `host:` data (F026, F027), inputs, expression errors (F024), F025, F034, W114, W118 for a React component that throws, and origins (the `origins` feature) |
| `examples/` | The spec's own examples, and the complete example in `../examples/complete/` |

The set covers stages 1 to 4b (§11): parsing, resolving, validating, loading and expanding.

**The tree schema.** [`compmd-tree.schema.json`](compmd-tree.schema.json) (JSON Schema 2020-12) defines the parse tree (`#/$defs/parseRoot`) and the expanded tree (`#/$defs/expandedRoot`) of §12.4. Every `parse.json` and `expanded.json` here fits it, and a runner SHOULD check that it does.

A processor case's `context.json` may hold `registry` (pack name to releases: `version`, `path` to its folder in the case, the `integrity` the registry publishes), `ambient` (pack name to folder, which replaces a `registry` pack of the same name, as the processor's own pack does: §6.3) with `reportAmbient`, `allowedOrigins`, and for stage 4: `now` (the render time), `tz`, `inputs` (values given for this render), `urls` (remote URL to a file in the case) and `host` (a `host:` source to a file).

Licensed [MIT](https://opensource.org/license/mit).
