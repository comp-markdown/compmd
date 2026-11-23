## (the review) · round 1 · t1
- [x] **Hakan Serce** · 2026-11-23T17:31:47Z · c1
  can we get rid of the yellow part on the corner remaining in place of the turned up corner?
  - **claude:a7b49598** · 2026-11-23T17:33:51Z · c2
    Agreed. The yellow half is there because the note is a rectangle: the spec gives `background` only a token or a CSS colour, and no box param cuts a corner, so the pack can only paint over it. As settled, the spec will let `background` take a CSS gradient, so the note cuts its own corner and the page shows through. That is a spec change with its design first (which gradients, how they are kept safe in the page's CSS, and the corner in right-to-left text), tracked as its own task; this corner is redone on it.
  - **claude:a7b49598** · 2026-11-23T21:07:13Z · c3
    Done in round 2 (f49b29a): the note now cuts its own corner with a gradient (spec §9.4), so the page shows through where the corner turns up, and the flap is the triangle the cut leaves out, turned over. A thin shadow still runs past the cut, since a box's shadow follows its rectangle.
