+++
id = "01a1ffde-8adc-73b8-bb8a-644200353d83"
number = 59
title = "A component’s type: its frame’s root, and type in the host contract"
status = "approved"
tasks = ["CMD-372"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T20:48:57Z
updated = 2026-11-21T22:48:28Z
+++

CMD-372, [Type in a box](docs/type-params.md) decision 5 (CR-54): a host-language component gets its box's type.

## What to check
- **The spec (§7.4, §7.6):** a paragraph, *A component's text is in its box's type* (CSS inheritance, a frame's root given the type the box shows, `type` in the context, `update` when it changes); the context's listing; the bindings' `compmd` prop; the changelog.
- **The renderer:** `contract.ts` (`TypeStyle`, `type`), `type-style.ts` (read from the box's computed style), `component.ts` (set at mount once placed, after an update once the box's new params are flushed, a second update only when it changed; `restyle()` on the handle), `scheme.ts`/`page.ts`/`region.ts` (`restyle()` for the host, after a theme change around the boxes), the React and Svelte bindings. `test/browser/component-type.test.ts` in Chrome and WebKit: the box's own params over what it inherits, the new type after an update, `restyle` updating only when it changed.
- **The sandbox:** a `type-style` message (`protocol/messages.ts`, `parent-end.ts`, `frame-end.ts`, its shape checked in `protocol/type-style.ts`); `use.ts` sends the box's type at connect and when it changed (after a render, the scheme, `restyle`); `frame/runtime.ts` sets it on the frame's root and has the components read theirs again. `frame/components.ts`: a remount reads `fit` and `type` live, as it did `scheme` (it kept `fit` from the first mount before). `test/view.test.ts` in both page modes and both browsers: a framed component's `context.type` and its drawn text follow its box, and the new type after a change.
- **The pack tool:** `draw` passes `type` (absent from an older renderer).
- **The Mermaid pack:** its labels in the box's font and size (`themeVariables`), which Mermaid measures them in; 0.1.3 in `versions.json`, its files and its README.
- **The Developer's Guide:** chapter 13, what inherits and what a canvas takes from `context.type`.
- **One thing to know:** after an update that changes the type, a framed component may update twice, first with its old type (the part reaches the frame before the type does), then with the new one.
