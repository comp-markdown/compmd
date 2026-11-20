## docs/first-components.md:74-75 · round 1 · t1
> 4	card, link-card, gallery	compmd definitions on box, in a grid	—	no code	8 of 11 frameworks ship cards. A gallery is a grid of cards, each an image, a title and a description, so it needs no code and its images are the document's.

- [x] **Hakan Serce** · 2026-11-20T18:43:17Z · c1
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
      - **claude:79612601** · 2026-11-20T19:04:25Z · c8
        Added in round 2 (row 4b, CMD-348). The two layouts with what the spec has today (§9). One correction to my sketch above: a container closes with `::: /name`, never a bare `:::` (§4.1).

        **Two columns, side by side from `md` up, stacked below:**

        ```md
        ::: grid {md:grid-template-columns="1fr 1fr" gap=m}
        ::: box
        ## What we shipped

        The new editor, and builds twice as fast.

        - Live preview
        - Shared links
        ::: /box

        ::: box
        ## What's next

        Comments, then offline mode.
        ::: /box
        ::: /grid
        ```

        **Three columns, the middle one twice as wide:**

        ```md
        ::: grid {md:grid-template-columns="1fr 2fr 1fr" gap=m}
        ::: box
        **Before**

        The old flow: export, upload, link.
        ::: /box

        ::: box
        **Now**

        Write it once; the page is live as you type, and readers see the same thing in their app.

        ![The editor, side by side with the page](editor.png)
        ::: /box

        ::: box
        **Next**

        Comments in the margin.
        ::: /box
        ::: /grid
        ```

        What an author has to get right: the track list (`1fr 2fr 1fr`), the `md:` prefix (without it the columns never stack on a phone), and a `box` around each column of more than one block, since each top-level block in a grid is one item (§9.1): forget the `box` and a heading and its paragraph become two columns. A column of one paragraph needs no `box`.

        The same two with `columns`:

        ```md
        ::: columns
        ## What we shipped
        …
        :: #end
        ## What's next
        …
        ::: /columns
        ```

        ```md
        ::: columns {split=wide-middle}
        **Before** …
        :: #middle
        **Now** …
        :: #end
        **Next** …
        ::: /columns
        ```

        That second one needs a `wide-middle` I hadn't listed: `split` becomes `even | wide-start | wide-middle | wide-end`, `wide-middle` for three columns only. I'll add it to row 4b.

## docs/first-components.md:107 · round 1 · t2
> type

- [x] **Hakan Serce** · 2026-11-20T18:44:35Z · c2
  what about a Word Cloud?
  - **claude:79612601** · 2026-11-20T18:52:45Z · c4
    Yes, I think it belongs, as a component of its own rather than a `chart` type: its data is words with weights, not x and y, the same reason radar and sankey are left out of `chart` (*Left out of `chart`*). My proposal: `word-cloud`, a leaf over data rows, `:: word-cloud[What readers asked about most] {rows=(md.data.topics) text=topic size=count}`, the label its text alternative and its fallback a list of the words by weight. Laid out with `d3-cloud` (BSD-3, layout only), drawn as SVG by the component, so it prints, follows the colour scheme and its words stay text. Its size and how it measures text in a strict frame are to check before the task, as the research table does for the others. A `text=` param that counts the words of a passage itself could come later. If you agree it goes in the list with its task.
    - **Hakan Serce** · 2026-11-20T19:03:12Z · c7
      good
      - **claude:79612601** · 2026-11-20T19:04:29Z · c9
        Added in round 2: row 10b, and its task CMD-349 in Packs 2, after the wrap command (CMD-9).

## (the review) · round 1 · t3
- [x] **Hakan Serce** · 2026-11-20T18:56:11Z · c5
  The publishing pain is real with 25 packs, versioning, publishing, approving staged packages, etc.

  I don't want to version them all together, I want them versioned separately. But what I need is automation scripts for these:
  1- let's have a script I can run to create the initial placeholder 0.0.0 version for multiple packs with one command.

  2- let's have one config file defining the version of each of these packs. And a single command can publish the ones that changed with the correct version.

  3- also need a single command to show me what is staged, let me choose which ones I approve (like a tiny TUI of checkboxes, default all selected), and approves the staging for all automatically.

  With these, it would not be that difficult for me to manage 25 or even 100 packs.
  - **claude:79612601** · 2026-11-20T19:05:03Z · c10
    Agreed: versioned apart, released together. My proposal, as decision 12 here and a revision of [releases](docs/releases.md) decision 8; four points marked **(confirm)** are yours to settle.

    **1. Placeholders, one command: `cargo make pack-placeholders card chart music …`.** For each named pack npm doesn't have yet, it writes the placeholder RELEASING.md describes (*A placeholder first*: a `package.json` at `0.0.0` that says so, a README pointing to the real versions, the MIT `LICENSE`) in a folder outside the repository, and runs `npm publish` for each, asking your one-time code once and reusing it while it is valid. It ends with the list of trusted publishers to set up (the repository, the workflow below, staging only), since npm makes them on npmjs.com; whether npm's command line can make them too is checked in the task.

    **2. One file of versions, one command that releases what changed.** **(confirm)** `compmd-packs/versions.json` is the one place a pack's version is written, `{ "card": "0.1.0", "chart": "0.2.0", … }`; each pack's `package.json` follows it (a check in `cargo make ci` fails when they differ, and the command writes them). `cargo make release-packs` asks npm which of those versions it doesn't have, shows them, and on your yes makes the tag `<pack>-v<version>` for each (each pack keeps its history), pushes the tags, and starts **one** run of **one** workflow, `release-packs.yml`, which runs `cargo make ci` once and stages every version npm doesn't have. **(confirm)** One run for the batch, not one per tag: 25 packs released together would otherwise run CI 25 times.

    **3. Approving, one command: `cargo make approve-packs`.** It lists what is staged for every `@comp-markdown` pack (`npm stage list`), as checkboxes, all selected; space toggles, Enter approves the chosen ones (`npm stage approve`), in the order they were staged, asking your one-time code once and again only when it expires. **(confirm)** The checkboxes from `@inquirer/checkbox` (MIT), a small dependency of the repository's tools only, rather than writing the terminal handling ourselves.

    **4. The two packs released today move to it.** **(confirm)** `release-mermaid.yml` and `release-wireframe.yml` go; their trusted publishers are made again for `release-packs.yml`, since npm can't change a publisher's workflow (RELEASING.md).

    Staging needs npm 11.15 or newer (yours here is 11.11), so the commands run a pinned npm, as the release workflow does. Once you've settled the four points I'll write it in, with RELEASING.md's steps, and add the tasks: the placeholders and the versions file first, since the first component's release (CMD-313, `card`) needs them.
    - **Hakan Serce** · 2026-11-20T19:12:34Z · c11
      good, make sure we have --dry-run option for each. One run, not per tag. 

      Also, we need the publish placeholder command to also set the trusted publisher properly, is that possible?

      checkboxes package is fine.

      yes  existing packs (Mermaid, Wireframe) are handled the same way, they are not special.

      sure we can upgrade npm.
      - **claude:79612601** · 2026-11-20T19:14:32Z · c13
        Done in round 3: [releases](docs/releases.md), decision 8, and tasks CMD-350 to CMD-352 in Packs 1. Every command has `--dry-run`; one run for a batch; Mermaid and wireframe move to `release-packs.yml`. And yes, the placeholder command sets the trusted publisher too: `npm trust github <package> --file release-packs.yml --repository comp-markdown/compmd --allow-stage-publish --yes` (npm 11.10 and on), once the placeholder exists; npm fits about 80 packages in one 5-minute 2FA window. A package has one publisher, so Mermaid's and wireframe's are revoked (`npm trust list`, `npm trust revoke`) and set again.

## (the review) · round 2 · t4
- [poke] **Hakan Serce** · 2026-11-20T19:13:09Z · c12
