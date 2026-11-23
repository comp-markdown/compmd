## compmd-packs/gallery/README.md:45 · round 1 · t1
> pnpm --dir compmd-packs/gallery dev     # the test page: demo.comp.md, three walks from walks.yaml

- [x] **Hakan Serce** · 2026-11-21T14:33:17Z · c1
  This gives F026: `walks.yaml` doesn't exist; `md.data.walks` is null
  - **claude:79612601** · 2026-11-21T14:35:41Z · c2
    Right, and the gallery’s tests didn’t see it, since they give `walks.yaml` in the request themselves. The cause is the test page: `compmd-pack dev` gave the processor the demo’s text alone, so no demo could read its own data. It now gives the pack’s folder as the demo’s root folder (its `.comp.md`, `.md`, YAML, JSON, TOML, CSV and text files, never `node_modules` or `dist`), as a host gives a document its folder; images load by URL as before. A test checks which files the page’s script imports, and `create-pack`’s browser test still opens its packs’ pages in Chrome and WebKit: 434caf2.
  - **claude:79612601** · 2026-11-21T14:35:47Z · c3
    A correction: the fix is commit 1273e21, not the hash in my reply above.

## compmd-packs/gallery/index.comp.md:27 · round 2 · t2
> :: gallery[Three walks around the harbour] {rows=(md.data.walks)}

- [x] **Hakan Serce** · 2026-11-21T14:39:31Z · c4
  is it really rows? The demo shows 3 cards next to each other on a single row, for instance?
  - **claude:79612601** · 2026-11-21T14:40:01Z · c5
    It means rows of data, the records of a list (the design’s word: “a leaf over data rows”), not rows of the layout. In a gallery, where the layout has rows too, it reads as the second, so it is the wrong name here. I’d rename it `items`, one card per item: `:: gallery[Three walks around the harbour] {items=(md.data.walks)}`, and say so in the README and the design’s sketch. Shall I?
    - **Hakan Serce** · 2026-11-21T14:40:36Z · c6
      better. do it.
      - **claude:79612601** · 2026-11-21T14:41:56Z · c7
        Done in 3809cb8: `items`, with `as=item` in its `each`, in the definition, the demo, the README, the tests and the design’s sketch and decision 7; the prose says “side by side” where it said “in a row”.

## compmd-packs/gallery/demo.comp.md:13 · round 3 · t3
> :: gallery[Three walks around the harbour] {items=(md.data.walks)}

- [x] **Hakan Serce** · 2026-11-21T14:44:50Z · c8
  can we add more examples to the demo? how does this look if we have alot of these, can we control the height of an item, or how many we would like to have in one row, etc? These would showcase in the examples.
  - **claude:79612601** · 2026-11-21T14:45:39Z · c9
    Yes. The grid has the controls already, so the gallery can pass them on as params:

    - **`min: length = 14rem`:** the narrowest a card gets. As many sit side by side as the column holds at that width, and wrap: `min=10rem` packs a dozen photos, `min=20rem` makes two big ones.
    - **`columns: number?`:** exactly that many side by side, in place of `min`. It takes breakpoints like any param: `columns=1 md:columns=3` stacks on a phone and puts three across from `md` up (the definition writes `grid-template-columns=(columns == null ? … : "repeat(" + string(columns) + ", minmax(0, 1fr))")`).
    - **`gap: space = m`:** the space between cards.

    **An item's height** is the one with a trade-off. The cards of a row are the same height already, the tallest one's, since the grid stretches them. A fixed height (`height=18rem`) would cut off a card's text when it is longer, or leave space when it is shorter. What usually makes a gallery uneven is images of different shapes, and the card can't size its image yet, since its image is the caller's Markdown. I'd leave height out for now, or add it as each card's `min-height`, so cards are at least that tall and never cut. Which would you prefer?

    **The demo** would then have: the three walks; twelve photos at `min=10rem`; two across with `columns=2`; and one across on a phone, three from `md` up (`columns=1 md:columns=3`), each with a line saying what it shows.
    - **Hakan Serce** · 2026-11-21T14:49:34Z · c10
      how does existing MDX/React components hand the height/different sized items?

      separately, I think it starts getting really hard to see what all options a component provides, can we make sure we showcase each parameter/usecase by using render=both
      - **claude:79612601** · 2026-11-21T14:50:24Z · c11
        **How others handle it** (from what I know of them, not checked against their docs today):

        - **MUI's `ImageList`:** `cols`, `gap` and `rowHeight`, and a `variant`. `standard` crops every image to the row height (`object-fit: cover`); `masonry` keeps each image's own height, in columns; `quilted` lets an item span rows and columns.
        - **react-photo-album:** layouts `rows` (a target row height, each image scaled to fill it), `columns` and `masonry`.
        - **Card grids** (Bootstrap's, Starlight's `CardGrid`): every card in a row stretches to the tallest one, which ours do already, and the image on top is usually cropped to a fixed height or ratio.

        So the common answer to items of different sizes is **cropping each image to one shape**: a `ratio` (`ratio=4/3`), the image covering it. We can't do that today, and it's a spec gap: `object-fit` styles a component's box, and a `compmd` definition can't style the Markdown image in it. Closing it would be a spec change, for example `object-fit` and `aspect-ratio` reaching an image that is a box's only content. That's worth a design of its own, and it would help `card` and `figure` too. I'd file it as a task rather than fold it into the gallery. Masonry would need CSS columns, which no param gives; I'd leave it out.

        **`render=both`:** yes. The demo becomes one example a param: a `compmd` block with `render=both`, which shows its source and then what it draws, each with its own frontmatter importing the gallery. It covers `items` from data and written inline, `min`, `columns` with a breakpoint, `gap`, a missing description, and a row's worth of photos. I'd do the same for `card` and `figure`'s demos now, and make it the rule for every pack's demo in `docs/writing-a-pack.md`.

        So I'd do:
        1. `min`, `columns` and `gap` as params, with the demo of examples, in this review.
        2. The card and figure demos as examples, and the rule in `docs/writing-a-pack.md`.
        3. A task for cropping images in a box (a spec change).

        Go ahead with all three?
        - **Hakan Serce** · 2026-11-21T14:53:45Z · c12
          1. good
          2. good
          3. a task for cropping images in this sprint and start right away after this
          4. another task dependent on 3 to fix gallery component properly after 3
          - **claude:79612601** · 2026-11-21T15:04:39Z · c13
            All four are in hand:

            1. **Done:** `min` (14rem), `columns` (in place of `min`, with breakpoints) and `gap` (`m`), with the demo as `{render=both}` examples: the walks from data, items written inline (one without a description), a dozen photos at `min=10rem gap=s`, `columns=2`, and `columns=1 md:columns=3 gap=l`. A fix found on the way: a breakpoint value didn't reach a `compmd` component's param through an expression (`columns` in the `grid-template-columns` the gallery passes to `grid`), against spec §9.3. The processor evaluates it at each breakpoint now, with a test.
            2. **Done:** the card's and figure's demos as examples, `create-pack`'s templates' demos and the demo it writes for a wrapped package too, and the rule in `docs/writing-a-pack.md`, decision 3. The Mermaid and wireframe demos are **CMD-368**, since their page tests change with them.
            3. **CMD-366**, cropping an image to its box, in this sprint and started. Its design, `docs/cropping-images.md`, is drafted: a box whose only content is an image fits it by `aspect-ratio`, `object-fit` and `object-position`, as a component's box fits a drawing. It goes to you as a review of its own.
            4. **CMD-367**, the gallery's images cropped to one shape, depending on CMD-366.

            The commit for 1 and 2 follows once `cargo make ci` is green, which is running now; then round 4 of this review.
