# Code and what it draws

- **Status:** Proposed (2026-11-17).
- **Decides:** what a code block that a code renderer draws (spec §7.7) shows: its code, what the renderer draws, or both together, and whether a `compmd` block can show the CompMD it holds beside what that renders.
- **Related:** spec §7.7 (code renderers), §12.3 (the expanded tree), §14 (degradation, printing, plain text), [code renderers](code-renderers.md), [printing a document](printing.md), [a blog in CompMD](blog-tool.md).

## Where it stands

- A code block whose language a code renderer renders always shows what the renderer draws; its code shows only in a plain renderer, or as the fallback when the component can't run.
- A post or a guide that explains a block ("this is what the block below draws") writes the code twice: once in a block of another language (` ```text `, or the fence indented), once in the block that renders. The two drift apart.
- The website's guides show each example's source beside its render with a component of their own (`Example.svelte`), from snapshots their build makes. That component isn't part of CompMD, so a blog post can't use it.

## Decisions

1. **`render` is a param every rendered block takes**, as `alt` is (spec §7.7): `render=source`, `render=output` or `render=both`. The default is `output`, which is what a rendered block shows today, so no document changes.

   ````md
   ```mermaid {render=both alt="Draft, then review, then published"}
   graph LR; Draft --> Review --> Published
   ```
   ````

   It is the processor's param, not the component's: the component never gets it, as it never gets `alt` directly (it gets `fallback.text`). A code renderer's signature that declares a param named `render` would never get it from a block, so it is F013. The spec says nothing today of a signature that declares `alt`, `class` or `id`; this change says the same of them (F013 for `alt` in a code renderer's signature; `class` and `id` in any), in the same row. It isn't evaluated: a bare value only, so `render=(…)` is F009, since what shows is decided when the document is written, not from data or inputs. An unknown value (`render=code`) is F009, naming the three values.

2. **`render=source` shows the code block and nothing else.** The component isn't run, and the block needs no `alt` (no W101), since it draws nothing that needs a text alternative. Its other params are still checked against the signature (F008, F009, F010), so switching it back to `output` gives no new diagnostics. This is for a post that shows a diagram's source on its own, and for an author who wants to see the code while a renderer misbehaves, without changing the block's language.

3. **`render=both` shows the code block, then what it draws, in one box.** The common params (spec §9.2) apply to that box, which holds the two; `alt` is still needed (W101), for the drawing. The order is fixed, code first: a reader reads the source, then what it makes. How they're laid out is the renderer's style, not the spec's: `@compmd/render` stacks them, and gives the box a class (`compmd-both`) so a host's or a blog's stylesheet can set them side by side on a wide page. No param for the layout: the common params' `direction` and breakpoints already lay out a box's blocks, and the two blocks inside are the box's.

4. **What falls back, falls back once.** When the component can't run (pending, W118, W121, a processor without a browser), a `both` block shows its code once, not the code twice: the drawing's fallback is the code block, which is already there. A pending component's placeholder still marks the drawing's place.

5. **The expanded tree says it on the `code` node** (spec §12.3): a rendered block gets `render`, `"output"` when absent, beside `component` and `alt`.
   - `source`: a `code` node with `render: "source"` and no `component`, so a renderer that doesn't know `render` shows the code block, as it should.
   - `both`: a `code` node with `render: "both"`, `component` and `alt`. A renderer shows the code block, then runs the component below it. A renderer that doesn't know `render` treats it as `output`: it draws, and shows the code only as the fallback, which loses the code but nothing the reader can't read in the plain Markdown.

   The schema (`compmd-tree.schema.json`) gets the field, and the conformance set cases for each value, the default, a bad value, an evaluated value, `source` without `alt` and the reserved name.

6. **Plain renderers show the code, whatever `render` says.** A plain renderer ignores the info string after the language, so every value degrades to the code block (spec §14), and GitHub draws a Mermaid block itself, as today. Nothing in §14's table changes; a sentence says so.

7. **Printing and plain text follow what shows.** On paper (spec §14, *Printing*), `both` prints the code, then the drawing (or the drawing's fallback, by `print: fallback`, once, as in decision 4); `source` prints the code. A document's plain text gives a rendered block's code once, as it does today, for every value: the code is the block's text and the drawing's fallback.

8. **`compmd` blocks: proposed as a second step, a task of its own.** A ` ```compmd ` block with no `component=` is a code block today (CompMD source shown as source). `render=output` or `render=both` on it would render the CompMD it holds as a document of its own: its own frontmatter, imports and data (resolved from the outer document's place, so relative paths work), its own inputs, its diagnostics reported at their place inside the block, and a depth limit, a `compmd` block inside it showing its source only. Its default stays `source`, unlike a code renderer's, since every ` ```compmd ` block in a document today is an example meant as source. This is what the guides' `Example.svelte` does and what a blog post about CompMD needs, but it is a nested document, with questions of its own (its `md.doc`, whether its inputs are the outer document's, how an editor's origins point into it, the sandbox's frames for its components), so it gets its own design, after this one is built. Decisions 1 to 7 don't depend on it.

## What it changes

- **The spec:** §7.7, a bullet for `render` (its values, its default, that it isn't evaluated, that `source` needs no `alt`, that `both` is one box, code first); §5.1's sentence on `class`, `id` and `alt` names `render`; §12.3, the `code` node's `render`; §14, a sentence that plain renderers show the code for every value, and the printing and plain-text sentences; the catalog's F009, F013 and W101 rows name it. A changelog entry under *Unreleased*.
- **The processor:** the fence's param list reads `render` (`validate/params/fence.rs`), `source` leaves the component out of the expanded `code` node (`expand.rs`), W101 skips `source`, and the conformance cases.
- **`@compmd/render` and the sandbox:** a `code` node with `render: "both"` renders as a box with the code block (highlighted, as any code block) and the component's mount below it; the `compmd-both` class; the fallback shown once; print styles keep the two together when they fit (`break-inside: avoid`, as for a box).
- **The blog and the guides:** the blog's guide shows `render=both` in a post; the website's guide on code renderers gets an example. `Example.svelte` stays until the `compmd` block of decision 8 replaces it.
- **Editors** ([CompMD in editors](editor-support.md)): the grammars already highlight a fence's param list; the language server completes `render`'s three values.

## Open questions

1. `both`'s layout: stacked in `@compmd/render`, side by side only by a host's stylesheet (decision 3), or a side-by-side default on wide pages?
2. Decision 8: `compmd` blocks in a task of their own after this one, or in this task?
3. Decision 1's F013 for a signature declaring `alt`, `class` or `id`: a gap this change closes while it is there, or a change of its own?
