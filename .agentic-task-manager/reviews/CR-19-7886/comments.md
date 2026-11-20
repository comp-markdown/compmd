## docs/first-components.md:20 · round 1 · t1
> Only what runs in a strict frame is in the first list: no network, no frames, images only from data: and

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c1
  Hmm. If there are popular components that require network access, like a weather widget or something like that, we should include that. CompMD supports networking (and securely does it).
  - **claude:79612601** · 2026-11-19T19:03:18Z · c9
    Agreed: decision 5 goes. One line from the spec shapes how: a component gets the network only as a permissive pack the host configures, never on a reader’s prompt (§15); what the reader consents to is a document’s data from an origin (§8.6), which reaches components as params. So a weather widget is a strict component over a data import (Open-Meteo needs no key): the reader is asked about the origin once, and the widget never touches the network. Components that need the network themselves (embeds, map tiles) are permissive packs.
    - **Hakan Serce** · 2026-11-19T19:13:01Z · c17
      good

## docs/first-components.md:38 · round 1 · t2
> Drawing only: playing it needs a sound font from the network.

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c2
  I love this. We should absolutely support it. Guitar chords, etc. And I would like to investigate the music playing part better as well, compmd supports network anyway, it is just a user consent if they want to play the music.
  - **claude:79612601** · 2026-11-19T19:03:19Z · c10
    Will do: guitar chords (chord diagrams, and ChordPro sheets with chords over lyrics) go in beside ABC, and playing gets a section of its own. Playing may need no network at all: a sound font shipped in the pack is the pack’s own file, which a strict frame loads (§15, rule 2). I’ll research what abcjs’s synth needs (its sound font’s format, Web Audio in a sandboxed frame, a click to start), and permissive or a reader prompt only if it can’t be offline.
    - **Hakan Serce** · 2026-11-19T19:14:51Z · c18
      We should also consider Guitar Pro files (input) supported this way as well, if there is no existing component we can wrap we can create a backlog item for this as well.
      - **claude:79612601** · 2026-11-19T19:23:22Z · c27
        Yes. alphaTab (`@coderline/alphatab`) reads Guitar Pro 3 to 7, MusicXML and its own text format, draws tabs and scores, and plays them with a sound font; I’ll check its licence, its size, and whether it runs in a strict frame (it uses workers and an audio worklet by default, which the frames don’t allow). If it can’t, the Guitar Pro part goes to the backlog as a task of its own.

## docs/first-components.md:25 · round 1 · t3
> The first list

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c3
  I am thinking there might be other ones, like a weather widget,  a calendar widget (with month(s) view and marking event date(s)), also maybe a sticky note widget, a card/gallery widget with images/titles, descriptions, etc (we can do this without js even), ratings/stars ? Surprised these didn't show up already.
  - **claude:79612601** · 2026-11-19T19:03:19Z · c11
    They didn’t show up because I ranked by what docs frameworks ship and what npm’s component packages are, and these are a page’s widgets, not either. They go in: a weather widget (over a data import, above), a calendar (a month or months, with dates marked from data), a sticky note, a card gallery (image, title, description: a `compmd` definition, so its images show as the document’s, no frame), and a rating (stars). I’ll look for what else of that kind is missing too.
    - **Hakan Serce** · 2026-11-19T19:15:29Z · c19
      good

## docs/first-components.md:47-50 · round 1 · t4
> Charts. Chart.js (MIT, 70 KB gzipped, the most used) draws on a canvas, which prints at the screen's resolution; ECharts (Apache-2.0) has the most kinds but is 368 KB gzipped; Vega-Lite (BSD-3) is a JSON spec, and would fit a code renderer (```vega-lite) better than a leaf; Recharts is React's. Highcharts and amCharts are commercial, and ApexCharts' licence is no longer MIT.

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c4
  Let's make sure the chart component we choose gives a rich enough set of charts and a very ergonomic api.
  - **claude:79612601** · 2026-11-19T19:03:20Z · c12
    Agreed. I’ll compare the candidates (Observable Plot, ECharts, Vega-Lite, Chart.js and others) on the kinds of chart each draws and on how a document would write one, and propose the `chart` params first (`type`, `x`, `y`, series, …, over `md.data` rows), so the library serves the API rather than the other way round.
    - **Hakan Serce** · 2026-11-19T19:16:41Z · c20
      good. Answers are good, but I actually want to see the output for eac comment reflected inthe second round of the diff in the doc.

## docs/first-components.md:25 · round 1 · t5
> The first list

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c5
  Should we have a jq renderer, this could be great to play with json inputs?
  - **claude:79612601** · 2026-11-19T19:03:20Z · c13
    Yes: a code renderer for ```` ```jq ````, its input JSON from a data import or a param, shown with `render=both` (the query, then its output), and with an input the reader can change it is a playground in the page. I’ll find a jq that runs in a strict frame (jq compiled to WebAssembly, or a JavaScript port) and its licence and size.
    - **Hakan Serce** · 2026-11-19T19:17:15Z · c21
      awesome, again waiting in the doc update (second round, just like others)

## docs/first-components.md:76-79 · round 1 · t6
> G1. A component and the document's images. A strict frame's policy allows images from data: and blob: only, so a component can't show an image the document links. A zoom, a gallery or a slider needs the host to hand the images in (as the slot's items, already loaded), which no design says yet.

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c6
  we should have a task to dive deep into this, maybe as part of the figure pack in our list, when we are testing that we might have to figure this out?
  - **claude:79612601** · 2026-11-19T19:03:20Z · c14
    Agreed. One narrowing: `figure` and the card gallery are `compmd` definitions, so their images are the document’s and need no frame. G1 is for code components that take images (zoom, a lightbox, a before-and-after slider). I’ll add a task to dive into it, related to CMD-11 (the first packs), for when the image components are built.
    - **Hakan Serce** · 2026-11-19T19:17:58Z · c22
      share task id here
      - **claude:79612601** · 2026-11-19T19:23:22Z · c26
        CMD-266, “Images in a component’s frame: a zoom, a lightbox, a before-and-after slider”, in Packs & Blog beside CMD-11.

## docs/first-components.md:83-85 · round 1 · t7
> G4. Tabs and collapsibles in the core pack. They are what authors reach for first, and every page with one would load a frame for it. Implemented by the processor in @compmd/core, like the input components (spec §8.7), they'd cost nothing, but that is a spec change; as a pack, nothing changes. To decide before the pack is built.

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c7
  we should probably have these as built-in? I don't want to put these into the spec, but we can make them a pack together and include in the @compmd/render/sandbox as a built in?
  - **claude:79612601** · 2026-11-19T19:03:20Z · c15
    Agreed, no spec change: a pack the sandbox ships as a built-in, `@compmd/…` (the reserved scope, never fetched), ambient, and trusted since it is the host’s own code shipped with it, so tabs and a collapsible run in the page with no frame to load. The basics (tabs, details, steps) go in it; G4 becomes a decision.
    - **Hakan Serce** · 2026-11-19T19:18:45Z · c23
      good.

## docs/first-components.md:25 · round 1 · t8
> The first list

- [x] **Hakan Serce** · 2026-11-19T19:01:44Z · c8
  We can have a permissive embeds pack with embed-youtube, embed-spotify, embed-xyz components.
  - **claude:79612601** · 2026-11-19T19:03:21Z · c16
    Agreed: a permissive embeds pack, `embed-youtube`, `embed-spotify` and the like, each loading its player only when the reader clicks it (the reader’s action, and nothing sent before). It needs one sandbox change: a permissive frame’s second policy allows frames from `strict.html` only (`narrow.ts`), so it would add the pack’s configured origins to `frame-src`. That goes in the doc as the gap it waits for.
    - **Hakan Serce** · 2026-11-19T19:19:53Z · c24
      good, but just agreing is not enough, do the due dilligence and write the related part of the document with proper details such as which specific embeds we should support in the beginning, etc.
      - **claude:79612601** · 2026-11-19T19:23:22Z · c28
        Agreed; round 2 has the embeds part written out: which embeds come first and why, each one’s iframe origin and `allow` attributes, the click-to-load facade and what each provider’s terms say about it, and the sandbox change it needs.

## (the review) · round 1 · t9
- [x] **Hakan Serce** · 2026-11-19T19:20:59Z · c25
  This is good. Just a general feedback, waiting for the document updates for each agreed update in second round.
  - **claude:79612601** · 2026-11-19T19:23:22Z · c29
    Understood: round 2 changes the document for every point agreed here (c1 to c8, with c18 and c24), each traceable to its comment. Two research passes are running for it, charts and the rest; the round follows them.

## docs/first-components.md:37 · round 2 · t10
> The first list

- [x] **Hakan Serce** · 2026-11-19T21:08:03Z · c30
  As an author, I would want most of these as built in that I never need to specifially declare in every document. But technically, this isn't great, because it would mean carrying a heavy weight for even users that will use the minimal compmd capabilities. So, this should go into @compmd/host/sandbox/???  built-in configuration, correct? 

  As an example, for the blog section in compmd.dev, I would like to configure @compmd/blog once to have allof these packs available as built in. So that, I can use them in every post without ever referring to them in the frontmatter. Does that sound accurate?
  - **claude:79612601** · 2026-11-19T21:12:16Z · c32
    Accurate, and it needs nothing new: a host’s configuration makes a pack ambient (spec §6.3, `"ambient": true` in `compmd.config.json`), and its names are then usable in every document with no import, as the blog uses the site’s own pack already. So `website/blog/compmd.config.json` lists the packs once, ambient, and every post uses them; a document that names none pays for their signatures only. The spec stays as it is (decision 6’s `@compmd/basics` is the same mechanism, with the sandbox shipping the pack). One thing I haven’t verified: what a page loads for a post that uses none of an ambient pack’s components (a strict pack’s code goes only to the frames that use it; a trusted pack is built into the page’s script). I’ll write that into round 3 as a decision, checked against the code, with the blog as the example.
  - **claude:79612601** · 2026-11-19T21:18:16Z · c38
    Checked, and I need to correct c32: today a document that uses none of an ambient pack’s components pays for more than their signatures. The processor lists every host component a document can name, an ambient pack’s included (`host_components`, `crates/processor/src/resolve.rs`), so the sandbox compiles them all and the blog gives every post its page script (`render.ts`: live when that list isn’t empty). Round 3 (ce4e215) has your plan as decision 9, with the blog’s configuration as its example, and this as G10, with its fix as CMD-268 in this sprint: the list holds only the components the expanded tree uses.

## (the review) · round 2 · t11
- [poke] **Hakan Serce** · 2026-11-19T21:12:12Z · c31

## (the review) · round 2 · t12
- [poke] **Hakan Serce** · 2026-11-19T21:12:20Z · c33

## (the review) · round 2 · t13
- [poke] **Hakan Serce** · 2026-11-19T21:12:21Z · c34

## (the review) · round 2 · t14
- [poke] **Hakan Serce** · 2026-11-19T21:12:22Z · c35

## (the review) · round 2 · t15
- [poke] **Hakan Serce** · 2026-11-19T21:12:22Z · c36

## (the review) · round 2 · t16
- [poke] **Hakan Serce** · 2026-11-19T21:12:22Z · c37
