## compmd-packs/badge/demo.comp.md:8 · round 1 · t1
> # badge

- [x] **Hakan Serce** · 2026-11-21T16:20:22Z · c1
  this one brings up an interesting concern here. Authors probably keep using a few badges all the time, like say bug, feature, chore are the badges, and I want to configure them once, and then use them everywhere without ever configuring. So, the best I can think of today is, authors create a compmd file with these badges defined once as new components and use them, right? Is there a better way? If not, can you actually put such as example into the demo as well, like one external file defines the 3 mentioned above (with all fancy colors, fonts, etc, to indicate the eliminated repetition) and then use them multiple times in one section in the demo doc?
  - **claude:79612601** · 2026-11-21T16:21:06Z · c2
    Yes, that's the way, and I don't see a better one in CompMD today: a file of your own that defines `bug`, `feature` and `chore`, each a `badge` with its look fixed, which every document imports. Each is a one-line definition:

    ```compmd component=bug
    ---
    :badge[**{{md.fallback}}**]{background="#ffebe9" border="1px solid #cf222e"}
    ```

    and a document writes `:bug[bug]`, `:feature[feature]`, nothing else to configure. The other ways:
    - **A pack of them,** the same definitions published, when several sites or people share them. Same mechanism, imported by name instead of by path.
    - **`class` and a stylesheet,** `:badge[bug]{.bug}`: shorter still, but the look then lives in each host's CSS, and a document looks different, or plain, wherever it's shown.

    One limit: there are no font params (CompMD's vocabulary has colours, borders, radius, shadow and spacing, not type), so the "fancy font" part is Markdown's own, `**bold**` or `*italic*`, inside the definition.

    I'll add it to the demo as you describe: `labels.comp.md` next to the demo defining the three with their colours, borders and bold text, and an example importing it and using each several times in one section (a changelog's lines). In the next round, once the ci run going now is done.
    - **Hakan Serce** · 2026-11-21T16:23:14Z · c3
      Hmm, create a task to check the font part. This seems like an essential concept we missed. Share the task id here.
      - **claude:79612601** · 2026-11-21T16:23:47Z · c4
        Filed as **CMD-370**, in this sprint, P1: type in the common params (font family by tokens such as `body`, `heading`, `mono`; size; weight; style; text colour; perhaps alignment and line height), as a design first, tried on a badge, a callout, a pull quote, a card's title and a masthead, with the contrast question, then the spec, conformance, processor, renderer and guides. The demo's `labels.comp.md` uses them once it lands; for now it uses bold and italic.
        - **Hakan Serce** · 2026-11-21T16:25:05Z · c5
          One small change, the section headlines used in the demo files are confusing. Can you make each example a H1 section and try not to use a H1 in the examples below
          - **claude:79612601** · 2026-11-21T16:25:36Z · c6
            Will do: each example a `#` section named for what it shows (in place of the bold lead-ins, and of the demo's own `# badge` title), and the examples' own headings `##` or lower. In the next round of this review, for the badge.

            The other packs' demos (card, figure, gallery, link-card, rating, sticky-note, Mermaid, wireframe) and `create-pack`'s templates are written the same way, so I'd change them all alike, with the rule in `docs/writing-a-pack.md`, decision 3, as CMD-368 (the Mermaid and wireframe demos as examples) widened to every demo. Unless you'd rather keep it to the badge.
            - **Hakan Serce** · 2026-11-21T16:26:27Z · c7
              all good. send your round 2, and I'll approve
              - **Hakan Serce** · 2026-11-21T16:45:36Z · c8
                Actually, can we describe this use in the README as well, showing the contents of such a file (labels.comp.md) in the README.
                - **claude:79612601** · 2026-11-21T16:46:15Z · c9
                  Done in round 3 (b056ad0): the README has a section, *Your own badges, defined once*, with `labels.comp.md` written out (its import of `badge`, its export, the three definitions) and a document using them.
