# Code and what it draws

- **Status:** Accepted (2026-11-17).
- **Decides:** what a code block that a code renderer draws (spec §7.7) shows: its code, what the renderer draws, or both together; and how a `compmd` block shows the CompMD it holds beside what that renders.
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

   It is the processor's param, not the component's: the component never gets it, as it never gets `alt` directly (it gets `fallback.text`). A code renderer's signature that declares a param named `render` would never get it from a block, so it is F013. The spec says nothing today of a signature that declares `alt`, `class` or `id`, and the processor gives each a meaning by form (a block's `alt` never reaches the component, a leaf's does; `class=x` goes to a component that declares it, `.x` to the box). This change says the same of them as of `render`: F013 for `alt` in a code renderer's signature, and for `class` and `id` in any, in the same catalog row (decided in review). A declared common param (`padding`) is left as it is. It isn't evaluated: a bare value only, so `render=(…)` is F009, since what shows is decided when the document is written, not from data or inputs. An unknown value (`render=code`) is F009, naming the three values.

2. **`render=source` shows the code block and nothing else.** The component isn't run, and the block needs no `alt` (no W101), since it draws nothing that needs a text alternative. Its other params are still checked against the signature (F008, F009, F010), so switching it back to `output` gives no new diagnostics. This is for a post that shows a diagram's source on its own, and for an author who wants to see the code while a renderer misbehaves, without changing the block's language.

3. **`render=both` shows the code block, then what it draws, in one box.** The common params (spec §9.2) apply to that box, which holds the two; `alt` is still needed (W101), for the drawing. The order is fixed, code first: a reader reads the source, then what it makes. How they're laid out is the renderer's style, not the spec's: `@compmd/render` stacks them, and gives the box a class (`compmd-both`) so a host's or a blog's stylesheet can set them side by side on a wide page; stacked is the default on every width. No param for the layout: the common params' `direction` and breakpoints already lay out a box's blocks, and the two blocks inside are the box's.

4. **What falls back, falls back once.** When the component can't run (pending, W118, W121, a processor without a browser), a `both` block shows its code once, not the code twice: the drawing's fallback is the code block, which is already there. A pending component's placeholder still marks the drawing's place.

5. **The expanded tree says it on the `code` node** (spec §12.3): a rendered block gets `render`, `"output"` when absent, beside `component` and `alt`.
   - `source`: a `code` node with `render: "source"` and no `component`, so a renderer that doesn't know `render` shows the code block, as it should.
   - `both`: a `code` node with `render: "both"`, `component` and `alt`. A renderer shows the code block, then runs the component below it. A renderer that doesn't know `render` treats it as `output`: it draws, and shows the code only as the fallback, which loses the code but nothing the reader can't read in the plain Markdown.

   The schema (`compmd-tree.schema.json`) gets the field, and the conformance set cases for each value, the default, a bad value, an evaluated value, `source` without `alt` and the reserved name.

6. **Plain renderers show the code, whatever `render` says.** A plain renderer ignores the info string after the language, so every value degrades to the code block (spec §14), and GitHub draws a Mermaid block itself, as today. Nothing in §14's table changes; a sentence says so.

7. **Printing and plain text follow what shows.** On paper (spec §14, *Printing*), `both` prints the code, then the drawing (or the drawing's fallback, by `print: fallback`, once, as in decision 4); `source` prints the code. A document's plain text gives a rendered block's code once, as it does today, for every value: the code is the block's text and the drawing's fallback.

8. **A ` ```compmd ` block takes `render` too, and renders the CompMD it holds as an example: a document of its own.** A ` ```compmd ` block with no `component=` is a code block today, CompMD source shown as source. With `render=output` or `render=both` it is an **example**: the processor processes its text as a `.comp.md` document and shows what that renders (with `both`, after the code, as decision 3). This is what the guides' `Example.svelte` does, and what a post about CompMD needs.

   ````md
   ```compmd {render=both}
   ---
   rating: 4
   ---
   Rated :value[4 of 5]{of=(string(md.doc.rating) + " of 5")}.
   ```
   ````

   - **The default stays `source`**, unlike a code renderer's `output`: every ` ```compmd ` block written so far is an example meant as source, so no document changes. A definition fence (` ```compmd component=card `) isn't an example; `render` on it is F008, as any key a definition fence doesn't take.
   - **Its param list** is read as a rendered block's (spec §7.7): `render`, the common params, `class` and `id`, which apply to the example's box; any other key is F008 (an unknown param). Today the processor ignores what follows `compmd` on a block that isn't a definition; nothing written so far is a param list there, so this breaks nothing. No `alt`: the example is content, read as it is, not a drawing.
   - **Its own document, by the rule for imported documents** (spec §6.3, *scope is lexical*). Its frontmatter is its own: `md.doc` is the example's keys, its `md` key its version, imports, data and inputs, and an example without frontmatter is a version 1 document with none. Nothing of the outer document reaches it: not the outer `md.doc`, not its imports, data, inputs or definitions. Its `component=` fences define components for it alone. What it imports and the data files it reads resolve from the outer document's place, so `./sales.csv` is next to the outer file, and they are loaded with the outer document's permissions (a processor that may not fetch for the outer document may not for the example).
   - **Its inputs are its own, and the reader can change them.** Its `inputs` are declared in its frontmatter, as a document's, and start at their defaults; the values whoever runs the processor gives (a command-line argument, a URL query) are the outer document's only. Its input components change its inputs, and the processor renders the example again (spec §8.5), not the outer document. In the expanded tree the example's root lists its inputs, as a root does (spec §12.3), and an input param's `input` names an input of the nearest root above it, so a renderer knows which document `set` changes. How a processor keeps an example's values between renders is its business, as for a document (spec §8.7).
   - **Its diagnostics are the outer document's**, at their place in the block: line and column in the outer file, as the text is written there (the fence's indentation counted). So a broken example shows in the editor where it is written, and strict mode fails on it as on any fatal error. A post that shows an error on purpose writes it with `render=source`.
   - **One level.** A ` ```compmd ` block inside an example shows its source, whatever its `render` says (W123, a new warning: "an example inside an example shows its source"), so an example can't hold examples to any depth.
   - **The expanded tree:** the `code` node, `lang: "compmd"`, gets `render` and `document`, the example's expanded root (`type: "root"`, its `children` and its `inputs`). A renderer shows the code block (for `both`), then the root's children in the example's box. A renderer that doesn't know `document` shows the code block, which is the example's source, so nothing is lost.
   - **Origins** (spec §12.3) of the example's nodes are ranges of the outer document, where their text is written in the block, so an editor keeps an example's rendered parts through an edit as it does any node's.
   - **An example's components are its own** (decided 2026-11-17). A renderer finds host-language components by name, and an example can define or import a `card` that isn't the document's `card`. So each example has a number, its place among the document's examples from 1, which its expanded root gives as `example`; the processor's list of host-language components gives each one's example, absent for the document's own; and a renderer looks a component up among those of the nearest root above it. An example's `card` and the document's are two entries, which never meet.
   - **Components and the sandbox:** an example's host-language components run as the outer document's do ([running components in the sandbox](running-components-in-the-sandbox.md)): in frame mode in the outer document's frame, which the example renders in, and in host mode each use that isn't trusted in a frame of its own. Their trust levels are those the host gives the outer document's packs, so an example can't do more than its document.
   - **Plain renderers** show the code, as for decision 6. **Plain text** gives the code once, as decision 7. **Printing:** `both` prints the code, then the example; `output` the example.

## What it changes

- **The spec:** §7.7, a bullet for `render` (its values, its default, that it isn't evaluated, that `source` needs no `alt`, that `both` is one box, code first); §5.1's sentence on `class`, `id` and `alt` names `render`; §12.3, the `code` node's `render`; §14, a sentence that plain renderers show the code for every value, and the printing and plain-text sentences; the catalog's F008, F009, F013 and W101 rows name it. Examples (decision 8): a new §7.8, *Examples*, for ` ```compmd ` blocks, §6.3's lexical scope naming them, §8.7's sentence that an example's inputs are its own, §12.3's `document` on the `code` node and input params naming the nearest root's input, W123 in the catalog. A changelog entry under *Unreleased*.
- **The processor:** the fence's param list reads `render` (`validate/params/fence.rs`), `source` leaves the component out of the expanded `code` node (`expand.rs`), W101 skips `source`. Examples: a ` ```compmd ` block's param list read; its text processed as a document from the outer document's place, its diagnostics moved to their place in the outer file, its root under `document`; W123; inputs given to an example by the API. The conformance cases for both.
- **`@compmd/render` and the sandbox:** a `code` node with `render: "both"` renders as a box with the code block (highlighted, as any code block) and the component's mount below it; the `compmd-both` class; the fallback shown once; print styles keep the two together when they fit (`break-inside: avoid`, as for a box). A `code` node with `document` renders the example's root in that box, its input components setting the example's inputs.
- **The blog and the guides:** the blog's guide shows `render=both` in a post, and an example; the authors' guide shows both (chapter 17), the developers' guide how a host keeps an example's inputs (`exampleSet`, `exampleInputs`). The website's `Example.svelte` stays: it shows examples on the site's own Svelte page, which isn't a CompMD document, so a ` ```compmd ` block can't take its place there.
- **Editors** ([CompMD in editors](editor-support.md)): the processor's hints complete `render`'s values on a code renderer's block, and an example's diagnostics are at their place in the document. The grammars and the language server are tasks of their own, still to come: they highlight a ` ```compmd ` block's text as CompMD, and complete an example's `render`, whose hint has no component to name, unlike a rendered block's.

## Decided in review

(2026-11-17) `both` is stacked by default (decision 3); `compmd` examples are part of this change (decision 8); F013 covers a declared `render`, `alt` (on a code renderer), `class` and `id` together, in this change (decision 1). The design was approved with an example's inputs as proposed: the reader can change them, and each example keeps its own (decision 8). An example's components are its own (option (a), decided 2026-11-17): its root's `example` number, the processor's host-language components listed with it, and renderers and the sandbox finding them by it.
