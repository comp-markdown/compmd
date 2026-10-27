# CompMD

CompMD (Component Markdown) is a Markdown flavor that adds components, containers, slots and layout to CommonMark and GFM, and stays readable in any plain Markdown renderer. Files end in `.comp.md`.

**Status: draft.** Version 1 isn't published yet, and a change can still break documents written against the draft; the [changelog](CHANGELOG.md) says when one does.

| File | What it holds |
|---|---|
| [compmd-spec-v1.md](compmd-spec-v1.md) | The specification, version 1 (draft) |
| [CHANGELOG.md](CHANGELOG.md) | Every change to the spec, newest first |
| [llms.txt](llms.txt) | The AI authoring guide: what an AI agent reads before writing CompMD (measured below) |
| [conformance/](conformance/README.md) | Test cases any implementation can run (spec §13.2) |
| [examples/](examples/complete/release.comp.md) | A complete document that uses every construct, with its data files |
| [processor/](processor/README.md) | `compmd`: the processor in Rust, which parses, validates and expands documents (in development) |
| [render/](render/README.md) | `@compmd/render`: an expanded tree as a live page, with the host contract's bindings; the reference rendering of the built-ins |
| [existing-specs-comparison.md](existing-specs-comparison.md) | The spec compared with MDX, Markdoc, Nuxt MDC and Comark, MyST, Quarto, Livebook's Live Markdown and Active Markdown |

The spec is a document format of its own and doesn't depend on any application.

## Publishing version 1

*Proposed; to be confirmed.* Version 1 is published when every item below is done. Publishing freezes version 1: later changes are minor versions that only add (§2.2).

**When.** Publishing needs an outside reason: another implementer, users asking, or a partner. Until then the spec stays a draft in this repository, with its own changelog and conformance set.

**Checklist** (status as of 2026-10-24):

| | Item | Status |
|---|---|---|
| 1 | No open issues: every item in §17 is decided or deferred with a reason | To do: record keys are open |
| 2 | The tree schema (§12.4), and every expected tree in the conformance set fits it | Done |
| 3 | The changelog, with a version line in the spec | Done |
| 4 | The conformance set covers every construct and every diagnostic code | Done: 168 cases, all 56 codes |
| 5 | An implementation passes the whole set, including the `js` case (the plain binding) | Partly: the prototype passes all 166 cases it can run; the `js` case needs a plain-JavaScript runner, and the `origins` case an implementation of the `origins` feature |
| 6 | Plain renderers: every case degrades cleanly in a CommonMark and a GFM renderer | Done (the prototype's degradation report: 167/167 each, before the `origins` case) |
| 7 | The AI authoring guide, `llms.txt`, measured with an AI writing documents | Done (see below) |
| 8 | A review of the whole spec, start to end, with its editor | To do |
| 9 | License files: [`LICENSE`](LICENSE) (CC BY 4.0) for the text, [`LICENSE-CODE`](LICENSE-CODE) (MIT) for the conformance set and code | Done |
| 10 | The media type `text/markdown; variant=compmd` submitted to IANA's Markdown Variants registry (RFC 7764) | To do, at publication |
| 11 | The spec, conformance set and examples in `comp-markdown/compmd`, tagged `v1.0.0`; applications that implement it use it pinned (for example, as a git submodule) | Partly: the repository exists as a public draft; tagged `v1.0.0` at publication |

## The AI authoring guide

AI models know MDX, Markdoc and MDC far better than CompMD, so [llms.txt](llms.txt) gives an agent everything it needs to write CompMD in one file: ten rules, the syntax by example, the `md.` namespace and the CEL profile, data, inputs, layout, definitions, the habits from other formats that go wrong here, and every diagnostic code with its fix. It summarizes the spec and is checked against it: every example in it passes a processor with no errors.

**How it was measured (2026-10-23).**

- **Tasks:** 12 short descriptions, written for the measurement: six documents (release notes, a team page, a sales report, a tool's overview page, an event invitation, a changelog that defines its own component) and six app screens (a sales dashboard with two inputs, a task board, a settings form, an overdue-tasks list with a sticky header, a searchable team directory, a screen with a local `tsx` component). Each task names what it may use: a pack `@acme/ui` (`card`, `callout`, `badge`, `chart`, `table`, with their params and slots) and three data files (`team.yaml`, `sales.csv`, `tasks.csv`, with their fields).
- **Writers:** Claude Sonnet and Claude Haiku, one fresh agent per task, given only the guide and the task, with no other files and no web.
- **Baseline:** the same 24 runs given a one-paragraph description of CompMD instead of the guide.
- **Checking:** each document went through a prototype processor (stages 1 to 4b) with the pack and the data files. A document is valid when it has no `F` diagnostic. A writer whose document had diagnostics got them back once, as the processor reports them, and rewrote it.
- **Targets**, agreed before measuring: at least 11 of 12 valid on the first try, and 12 of 12 after one round of diagnostics, for each model.

**Results.**

| Writer | Valid, first try | Valid after one round | Warnings |
| --- | --- | --- | --- |
| Sonnet, with the guide | 12 of 12 | 12 of 12 | none |
| Haiku, with the guide | 12 of 12 | 12 of 12 | one W106, fixed in one round |
| Sonnet, baseline | 0 of 12 | not measured | |
| Haiku, baseline | 0 of 12 | not measured | |

- **Both targets are met for both models.** The documents use the hard parts, not workarounds: filtered and sorted `each` with `empty` slots, inputs bound to input components, `value` with formatting functions, a definition styled from its fence, breakpoints, a host component with a signature fence.
- **The one mistake with the guide** was `{{md.date(md.doc.date, "long")}}`: a function call in an interpolation, which shows as written (W106). Rule 5 now names this case with its fix.
- **Without the guide, models write other formats' syntax.** The most frequent baseline diagnostics: components used without importing the pack (F002, 48), guessed params such as `each {items=…}`, `if {test=…}` and inputs with `label=` (F008, 40), malformed component lines such as bare `:::` closers and `::: component name` definitions (F016, 25), and computed `{{ }}` (W106, 27).
- **CompMD's syntax wasn't the cause** of any failure with the guide. Without it, failures come from what a model can't guess (names, params, the definition fence), not from syntax that misleads it.
- **Limits:** one run per task and model, tasks written by the guide's author, and the prototype as the only processor. A later measurement should use tasks from real documents and more runs.

## License

Copyright © 2026 Hakan Serce.

- The specification text: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), in [`LICENSE`](LICENSE).
- The conformance set and any code: [MIT](https://opensource.org/license/mit), in [`LICENSE-CODE`](LICENSE-CODE).
