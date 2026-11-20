## docs/first-components.md:74-75 · round 1 · t1
> 4	card, link-card, gallery	compmd definitions on box, in a grid	—	no code	8 of 11 frameworks ship cards. A gallery is a grid of cards, each an image, a title and a description, so it needs no code and its images are the document's.

- [ ] **Hakan Serce** · 2026-11-20T18:43:17Z · c1
  What about, columns (like two-column, three-column) to make authors life easier. I know these are possilbe with containers/boxes already, but it still takes a lot compared to just one consistent two-columns with left and right slots, etc. Thoughts?

  Actually can you give me two examples showing two column and three column layouts for a section of a document here using existing spec defined capabilities rows/grid etc?
  - **claude:79612601** · 2026-11-20T18:52:45Z · c3
    Agreed, worth one: a `grid` with breakpoints is three things to get right (`md:grid-template-columns`, the item rule, a `box` around a column of several blocks), and 4 of 11 frameworks ship columns. My proposal: one `columns` component, a `compmd` definition on `grid` (no code, like `card`), with the first column the default slot and the others named slots, so a column holds any blocks with no `box`:

    ```md
    ::: columns
    The first column: paragraphs, a list, an image.

    :: #end
    The last column.
    :::
    ```

    `:: #middle` makes it three. Slot names follow the spec's logical directions (`start`/`end`, never left/right, §9.2), so it reads right to left too. `split=even | wide-start | wide-end` sets the widths, and below `md` the columns stack. A plain renderer shows the columns in order, which is the right fallback. If you agree it goes in the list as row 4b, with its task in Packs 1 beside `card`.
    - **Hakan Serce** · 2026-11-20T19:02:56Z · c6
      good, let's add it

## docs/first-components.md:107 · round 1 · t2
> type

- [ ] **Hakan Serce** · 2026-11-20T18:44:35Z · c2
  what about a Word Cloud?
  - **claude:79612601** · 2026-11-20T18:52:45Z · c4
    Yes, I think it belongs, as a component of its own rather than a `chart` type: its data is words with weights, not x and y, the same reason radar and sankey are left out of `chart` (*Left out of `chart`*). My proposal: `word-cloud`, a leaf over data rows, `:: word-cloud[What readers asked about most] {rows=(md.data.topics) text=topic size=count}`, the label its text alternative and its fallback a list of the words by weight. Laid out with `d3-cloud` (BSD-3, layout only), drawn as SVG by the component, so it prints, follows the colour scheme and its words stay text. Its size and how it measures text in a strict frame are to check before the task, as the research table does for the others. A `text=` param that counts the words of a passage itself could come later. If you agree it goes in the list with its task.
    - **Hakan Serce** · 2026-11-20T19:03:12Z · c7
      good

## (the review) · round 1 · t3
- [ ] **Hakan Serce** · 2026-11-20T18:56:11Z · c5
  The publishing pain is real with 25 packs, versioning, publishing, approving staged packages, etc.

  I don't want to version them all together, I want them versioned separately. But what I need is automation scripts for these:
  1- let's have a script I can run to create the initial placeholder 0.0.0 version for multiple packs with one command.

  2- let's have one config file defining the version of each of these packs. And a single command can publish the ones that changed with the correct version.

  3- also need a single command to show me what is staged, let me choose which ones I approve (like a tiny TUI of checkboxes, default all selected), and approves the staging for all automatically.

  With these, it would not be that difficult for me to manage 25 or even 100 packs.
