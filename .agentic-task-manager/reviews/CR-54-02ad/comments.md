## docs/type-params.md:110-111 · round 1 · t1
> - **G3. Line height, letter spacing, transform, decoration:** left out (decision 1); a later change can add any of
>   them as it adds the others, with tokens.

- [x] **Hakan Serce** · 2026-11-21T16:50:44Z · c1
  why are we leaving these out?
  - **claude:79612601** · 2026-11-21T16:51:12Z · c2
    Honestly, on weak grounds: I kept to what the five examples needed, and "a way to make text harder to read" is true of any type param. Looking again, three of them have real uses the examples should have had:

    - **`text-transform`** (`none`, `uppercase`, `lowercase`, `capitalize`): badges and section labels are very often uppercase ("NEW", "BETA"), and writing the text in capitals makes a screen reader spell it out. With the param, `:badge[new]{text-transform=uppercase}` reads "new".
    - **`letter-spacing`** (tokens `tight`, `normal`, `wide`; maybe an `em` length): uppercase small text wants a little space between letters, which is what makes such a label readable.
    - **`line-height`** (tokens `tight`, `normal`, `loose`, or a plain number): a pull quote or a masthead in `xl` looks loose at the page's line height, and a dense caption wants it tighter.

    The one I'd still leave out is **`text-decoration`**: an underline reads as a link, and strikethrough is Markdown's own (`~~…~~`), which a plain reader shows too.

    So I'd make it nine params: the six, plus `text-transform`, `letter-spacing` and `line-height`, with tokens as the others have, and add an uppercase badge and a tighter pull quote to the examples. Shall I?
    - **Hakan Serce** · 2026-11-21T16:52:36Z · c5
      keep all of them, it is an option for authors and component developers
      - **claude:79612601** · 2026-11-21T16:53:27Z · c8
        Done in the draft: ten params, `line-height` (`tight`, `normal`, `loose` or a number), `letter-spacing` (`tight`, `normal`, `wide` or an `em` length), `text-transform` (`none`, `uppercase`, `lowercase`, `capitalize`) and `text-decoration` (`none`, `underline`, `line-through`) added to decision 1's table, with their custom properties in decision 2, a note on what `text-transform` and `text-decoration` change, an uppercase `beta` badge and a tighter pull quote in the examples, and the gap gone. It goes in the next round with the answer on fonts (c4).
      - **claude:79612601** · 2026-11-21T16:55:11Z · c11
        In round 2 (de31611).

## docs/type-params.md:108-109 · round 1 · t2
> - **G2. Fonts the host doesn't have:** `heading` is the page's font unless the host maps it; a document can't ask
>   for a font of its own. Intended (decision 1), but worth saying in the guide.

- [x] **Hakan Serce** · 2026-11-21T16:51:27Z · c3
  should we reconsider packs containing their own fonts? or maybe accessing fonts similar to images?
  - **claude:79612601** · 2026-11-21T16:52:02Z · c4
    Yes, I think it's worth it, and the pieces are mostly there since CMD-362 and CMD-365 (a pack's own files reach the output, by its place). Three ways, in the order I'd rank them:

    1. **A pack's fonts.** A pack ships its font files (`fonts/harbour-serif.woff2`) and names them in its frontmatter (`md: fonts: { harbour-serif: { from: ./fonts/harbour-serif.woff2, fallback: heading } }`, say); a document that imports the pack writes `font-family=harbour-serif`. The host serves the files as it serves the pack's images: same place, the pack's integrity, no new origin, nothing to ask the reader. A pack's licence covers the font's (most open fonts, OFL, allow it). A theme pack, a newsletter's look, is then a pack.
    2. **A document's own fonts,** the same declaration with a path next to the document, as its images are.
    3. **Fonts from a URL**, as remote images: asked about by origin (§15). I'd leave it out: a font is fetched on every view, is a better tracker than an image, and an origin the reader refuses changes the whole page's text, not one picture.

    Each needs a **fallback token** (`body`, `heading`, `mono`), so a reader whose host doesn't load fonts, a plain renderer, or a sandboxed frame, still gets the right kind of text; and printing waits for the fonts, as it waits for components.

    How to fit it: CMD-370 keeps `font-family` taking tokens, and says a name a pack or the document declares is one too, so nothing in it changes later. The fonts themselves (the declaration, the host serving them, the renderer's `@font-face`, the fallback, print) are a design of their own: I'd file it as a task after CMD-370, in this sprint or the next, as you prefer. Or fold it into this design now, if you'd rather settle type once.
    - **Hakan Serce** · 2026-11-21T16:53:03Z · c6
      fold into the design
      - **claude:79612601** · 2026-11-21T16:55:11Z · c10
        Folded in, round 2 (de31611), as decision 7: fonts of the document's own, from a pack or next to the document, declared once in `md.fonts` with their files by weight and style and a fallback token (`body`, `heading`, `mono`); a pack exports and a document imports them as components (`fonts: [harbour-serif]`); the files served as images are (a pack's from its place, with its integrity; the document's from its folder), `woff2` and `woff` only, within the size cap; the fallback while loading, where fonts don't load and in plain renderers; never from a URL. Print waits for them (decision 6). The examples have a newsletter's look as a pack; *What changes* lists the spec sections, the processor, the renderer's `@font-face`, the sandbox. Two new gaps: weights between normal and bold, and fonts' licences.

        Decision 5 is now the proposal from c9 (the frame's root given the box's type, and `type` in the context), marked as proposed.

## docs/type-params.md:105 · round 1 · t3
> - **G1. Host-language components:** what draws in the page inherits the type; a sandboxed component, in its own

- [x] **Hakan Serce** · 2026-11-21T16:53:16Z · c7
  tell me more about this, with examples.
  - **claude:79612601** · 2026-11-21T16:54:00Z · c9
    G1 is about where a host-language component draws, since CSS's inheritance only reaches what is in the same page.

    **A component that draws in the page** (a trusted or permissive one, or the renderer's own) puts its elements inside its box, so the box's type reaches them as it reaches Markdown text:

    ```md
    :: rating[Four of five] {value=4 font-size=s color=muted}
    ```

    The rating's "4 of 5" label is small and muted, with no code in the rating for it, as long as the component doesn't set its own font. An SVG's `<text>` inherits the same way.

    **A component in a strict sandbox** draws in a frame of its own, another document, so nothing inherits:

    ```md
    ::: box {font-family=heading font-size=s}
    ```mermaid
    flowchart LR
      Quay --> Lighthouse
    ```
    ::: /box
    ```

    The diagram's labels stay in Mermaid's own font and size, whatever the box says; today the frame gets only `scheme` (light or dark), so its colours follow the page and its type doesn't. And a component that draws on a `<canvas>` (a chart library, a map) never inherits CSS at all, in a frame or not: it needs the font as a value (`"bold 14px Georgia"`).

    So I'd do both:
    - **The frame's root gets the box's type** as CSS, resolved (the tokens turned into the host's fonts and sizes, a pack's font loaded in the frame, which the host already serves pack files to since CMD-364). A component in a frame then inherits as one in the page does, with no change to it: Mermaid's labels follow `font-family=heading` if Mermaid's theme says `inherit`, which the Mermaid pack can set.
    - **The context carries it as values,** `type: { family, size, weight, style, color, lineHeight }`, the size in pixels and the colour resolved, as it carries `fit` and `scheme`, and `update` is called when it changes. A canvas chart draws its labels with them.

    A component that wants its own type keeps it: both are defaults it may ignore, as `fit` is. I'll write this in as decision 5 in the next round, unless you'd rather leave a component's type its own.
    - **Hakan Serce** · 2026-11-21T16:58:00Z · c12
      good
      - **claude:79612601** · 2026-11-21T16:58:38Z · c13
        Marked settled in decision 5; in the next round.
