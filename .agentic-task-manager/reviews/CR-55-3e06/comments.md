## (the review) · round 1 · t1
- [x] **Hakan Serce** · 2026-11-21T17:00:27Z · c1
  this component is a bit bland, can you check what we have in MDX and React ones that we can borrow here to make it look better.
  - **claude:79612601** · 2026-11-21T17:01:16Z · c2
    Agreed, it's a list with a border. What the libraries do:

    - **MUI Lab's `Timeline`**: each item a *dot* on a *connector* line, the date as *opposite content* on the other side of the line, and items alternating sides (`position="alternate"`); dots coloured or outlined, an icon in one.
    - **Mantine's `Timeline`**: a bullet per item on the line (an icon or a number inside), a title, text, and a small muted timestamp under it; the items up to `active` drawn in the accent colour, the line solid, dashed or dotted.
    - **Ant Design's `Timeline`**: coloured dots, a label (the date) on the start side or alternating, a "pending" last item.
    - **Flowbite / Tailwind UI**: the date small and muted above the title, the title bold, a dot on a thin line, sometimes a card per item.

    The common parts: **a dot per item on a continuous line, the date set apart (small, muted, often on the other side of the line), a title, then text,** and a way to mark the current item.

    With today's vocabulary a `compmd` definition can do most of it, as an item component instead of list items:

    ```md
    ::: timeline
    ::: event {date=1897 title="The harbour wall"}
    Built in a single summer, of granite from the quarry.
    ::: /event

    ::: event {date=1952 title="After the storm" tone=accent}
    The quay is rebuilt, a metre higher.
    ::: /event
    ::: /timeline
    ```

    - **The date in a column of its own** on the start side of the line (a grid of two tracks), and above the title on a phone, where there's no room.
    - **A dot on the line**, a small round box placed by position (the W119 change made that possible on a box with text in its flow), `tone=accent` filling it for the current item.
    - **A title** in bold, then whatever the item holds.

    Two things need CMD-370's type params, which are next: the date **small and muted**, and the dates **aligned to the line**. So I'd rebuild the timeline this way after CMD-370 lands (making CMD-319 depend on it), keeping the plain list form for a quick timeline. Alternating sides I'd leave out for now: it reads badly on a phone and in a plain reader. Does that direction suit you?
    - **Hakan Serce** · 2026-11-21T17:06:10Z · c3
      makes sense, go ahead with type changes then, do the changes you can and then let's keep this blocked on the type tasks.
      - **claude:79612601** · 2026-11-21T17:06:49Z · c4
        Will do: CMD-319 now depends on CMD-370. Once CMD-370 (in progress) is committed, I rebuild the timeline as `event` items (the date in its own column, a dot on the line, a title, `tone` for the current one) as far as today's params go, commit it here, and block the task on the type params for the date's look.
      - **claude:79612601** · 2026-11-21T19:49:54Z · c5
        Round 2 (7866449), the timeline rebuilt as agreed, on CMD-370's type params (in review as CR-57, so this waits on it):

        - **`event`**, a second component: a round dot placed on the line, the `date` small and muted (`color=muted font-size=s`), the `title` in bold (optional), then the event's text. `tone=accent` fills the dot with the accent colour for the current one.
        - **The quick form stays:** `::: timeline` around a list, the markers on the line.
        - **The demo,** five `#` sections: events along a line, the current event, more to say or only a date, from data, a quick timeline.
        - **Tests:** the tree (the dot's place, the date's type, the title, `tone`, data, F009); a page test in Chrome and WebKit, the dot centred on the line within a pixel and round, the current dot another colour, the date smaller than the text.
        - **One limit, in the README:** the dots are placed for the timeline's own room (`padding-inline-start=m`), so a use that changes that room moves the line away from them.
