## compmd-packs/link-card/demo.comp.md:63 · round 1 · t1
> ::: grid {min=14rem gap=m}

- [x] **Hakan Serce** · 2026-11-21T15:32:28Z · c1
  In this final example, the cards have a different, height on my browser. Is this expected? Also if author wants all of them the same size, is it something they can do with the grid parameters, etc?
  - **claude:79612601** · 2026-11-21T15:33:42Z · c2
    Not expected: a bug in the link card. A grid stretches each item to its row's height, but a link card's item is its own box (the one with `stretch-link`), and the card inside it kept its content's height. The card is `{fill}` now, so it takes its box's height and cards in a row are as tall as the tallest, as `card`'s are. A test checks the tree, and a page test in Chrome and WebKit measures the three cards of this example; in the next round.

    All the same size, across rows too, isn't something the grid can say today:
    - **In one row:** as above, by default (`align-items=stretch`).
    - **Across rows:** CSS's `grid-auto-rows: 1fr`, which CompMD's vocabulary hasn't (§9.1 gives `grid` `min`, `gap`, `align-items` and the `grid-template-*` params). `grid-template-rows="repeat(2, 1fr)"` does it only when the author knows how many rows there are, which `min` changes with the width.
    - **A fixed size:** `height=9rem` on each card, which works but cuts or overflows a longer description.

    If you want it, it's a small spec change: `grid-auto-rows` on `box` and `grid` (type `tracks`), so `::: grid {min=14rem grid-auto-rows=1fr}` makes every card as tall as the tallest. I'd file it as its own task in this sprint, since the gallery (CMD-367) wants it too. Shall I?
    - **Hakan Serce** · 2026-11-21T15:35:22Z · c5
      yeah, let's do it.
      - **claude:79612601** · 2026-11-21T15:36:02Z · c7
        Filed as **CMD-369** in this sprint (the spec, the processor, the renderer with a browser test, the Author's Guide), and CMD-367, the gallery's, now depends on it.

        The page test for the row heights found a second bug, in the renderer: `stretch-link` on the link card's box doesn't mark its link, which is inside the card's own box, so a click on the card's text does nothing. The fix is in the next round too, with the test.

## compmd-packs/link-card/demo.comp.md:65 · round 1 · t2
> :: link-card[The old orchard] {href="#orchard" description="Apples in October."}

- [x] **Hakan Serce** · 2026-11-21T15:33:59Z · c3
  Another thing, what if I want to put more content in these cards with formatted text, maybe even images? Do we need a non-inline version of this as well? How does the current MDX/React versions do this?
  - **claude:79612601** · 2026-11-21T15:34:42Z · c4
    You can do it today, without a new component: `stretch-link` is a flag any box takes, so a `card` with it is a link card with whatever its body holds:

    ```md
    ::: card {stretch-link}
    ### [The harbour at dusk](harbour.comp.md)

    A walk along the old quay, **an hour** before the boats come in.

    :: #image
    ![Boats at the quay](harbour.jpg)
    ::: /card
    ```

    The first link (the title) takes the clicks; other links in the text stay links of their own.

    How the MDX libraries do it:
    - **Starlight** splits it: `<LinkCard title description href />`, props only, and `<Card title>` with children for rich content.
    - **Mintlify** and **Docusaurus**'s card take children: `<Card title href img>` with Markdown inside, and the whole card is the link.

    So, for `link-card`, I see two ways:
    - **(a) Keep it a leaf**, for the one-line case it is now, and show `::: card {stretch-link}` in its README and demo for rich content. One more example, nothing new.
    - **(b) Make it a container**, as Mintlify's: `::: link-card {title="The harbour at dusk" href="harbour.comp.md"}` with formatted text in its body and an `image` slot, the title a param again (a container has no fallback). It's `card` with the title and its link written for you.

    I'd go with (a): (b) saves only writing the title as a link, and a second way to write the same card. Which do you want?
    - **Hakan Serce** · 2026-11-21T15:35:51Z · c6
      Can you put this kind of examples into the demo as well
