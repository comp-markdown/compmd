# CompMD conformance set

Test cases for implementations of the [CompMD spec](../spec/compmd-spec-v1.md). The format is in the spec's §13.2: one folder per case, with `case.json` (what the case needs), `input.comp.md`, and the expected tree and diagnostics.

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
| `definitions/` | Component definitions and signature fences: types, `input<…>`, template slots, `headings`, `print`, `runtime`, and the fence's param list for the component's box, a signature fence's too (F011, F017) |
| `resolve/` | Names through local definitions, imports, a stubbed registry (`context.json` `registry`), ambient packs and `@compmd/core`: F002, F013 (a common param, `lang` or `dir` declared too), F019–F023, F026, F029, F035, F036 (a box styled by both fences), F037 (in signatures, inputs and schemas), W102 (also for a runtime the host doesn't run, `context.json` `policy.runtimes`), W104 (component analysis), W105, W112, W113, W117, W122 (a signature's `print`), W121 (a pack's `peerDependencies` against `policy.runtimes` and `policy.libraries`, and library packs: a second version of a `single` one, one that can't be had) |
| `validate/` | Params, common params and layout values (`object-fit` and `object-position` among them, and a signature fence's), slots, interpolation paths, inputs and data sources, and `display=contents`: F006, F008–F010, F013, F014, F018, F028, F030, F031, W103, W120 |
| `expand/` | Stages 4a and 4b: definitions with params and slots, relative headings, `if`, `each` and its join modes, `value`, how values are written, URLs (W109), a definition's relative destinations (a pack's place with `pack`, a file of the root folder seen from the document, the caller's paths; F026 past the root), template slots, layout boxes and breakpoints, the component's box (the fence's params with the caller's over them, `display=contents`, host-language components' fences, a signature fence's defaults, `object-fit` and `object-position`), host-language components' defaults, input-bound params and the root's inputs, data files, remote and `host:` data (F026, F027), inputs, expression errors (F024), F025, F034, W114, W118 for a React component that throws, and origins (the `origins` feature) |
| `examples/` | The spec's own examples, and the complete example in `../spec/examples/complete/` |

The set covers stages 1 to 4b (§11): parsing, resolving, validating, loading and expanding.

**The tree schema.** [`compmd-tree.schema.json`](compmd-tree.schema.json) (JSON Schema 2020-12) defines the parse tree (`#/$defs/parseRoot`) and the expanded tree (`#/$defs/expandedRoot`) of §12.4. Every `parse.json` and `expanded.json` here fits it, and a runner SHOULD check that it does.

A processor case's `context.json` may hold `registry` (pack name to releases: `version`, `path` to its folder in the case, the `integrity` the registry publishes), `ambient` (pack name to folder, which replaces a `registry` pack of the same name, as the processor's own pack does: §6.3) with `reportAmbient`, `policy` (the host's policy for packs, §6.4 and §15: `packs` with their trust, `imports` as `any` or `listed`, `deny` with each pack's `version` range and `reason`, and `refused`, each pack the host left out with why), `allowedOrigins` (the origins remote data may come from, or `"any"` for every origin, §15), and for stage 4: `now` (the render time), `tz`, `inputs` (values given for this render), `urls` (remote URL to a file in the case) and `host` (a `host:` source to a file). `pending` names what the host is still getting (§6.4, §8.6): `packs` by name, `remote` URLs and `host` sources.

Licensed [MIT](https://opensource.org/license/mit).
