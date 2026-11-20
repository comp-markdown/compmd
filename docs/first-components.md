# The first components

**Status: proposed (2026-11-19); the packs decided (2026-11-20, CMD-11, decisions 10 and 11).** Which components CompMD should have first, so an author
finds what they reach for: what documentation frameworks ship, which npm component packages are used most, the widgets
pages have, and what CompMD covers already, with the list, each component's package, and how it runs.

## Decisions (proposed)

1. **The ranking weighs what people write, not only documentation.** CompMD's documents are posts, newsletters and
   notes as well as docs, so the frameworks' counts (below) are one signal; a figure with a caption, a chart or a
   calendar ranks higher than a framework count gives it, and an API field table, which only API docs need, lower.
2. **What CompMD has is not listed again.** Callouts are GitHub's alerts (spec §2.3), math is in the base syntax (the
   sandbox draws it with Temml), columns and card grids are `row` and `grid` (spec §9), Mermaid and wireframes are packs,
   and code blocks are highlighted by the host.
3. **A component without code is a `compmd` definition**, styled with the visual tokens (spec §9.4), as the spec's own
   `card` and `callout` are: no code, no sandbox, nothing to load, and its images are the document's own. Code is for
   what a definition can't do: changing when the reader clicks (tabs, a collapsible), and drawing (charts, music).
4. **A component that needs code is written in the plain binding** (spec §7.6), with no framework to load, unless the
   package it wraps is a framework's.
5. **What a component reaches outside the document, three ways, as the spec has them:**
   - **data the reader allows:** a document imports data from an origin, and the reader is asked about the origin the
     first time (spec §8.6); the component gets the data as params and stays in a strict frame, so what it's given has
     no way out. The weather widget works this way.
   - **the network its pack is given:** a permissive pack reaches the origins the host's configuration lists for it
     (spec §15), never because a document or a reader's prompt asks: the embeds, and a map's tiles.
   - **its own pack's files:** a sound font, icons, fonts and WebAssembly ship in the pack and load from the frame's
     own origin, with no network at all (spec §15, rule 2): music plays this way.
6. **The basics are built in, not in the spec:** `tabs`, `details` and `steps` are a pack the sandbox ships,
   `@compmd/basics` (the reserved scope, never fetched from a registry), ambient in every document it renders, and
   trusted, since it is the host's own code shipped with it: they run in the page, with no frame to load. The spec
   doesn't change; a processor without the pack shows their fallbacks.
7. **The `chart` component's params come first, and the library serves them** (below): `type`, `x`, `y`, `color` over
   `md.data` rows, drawn by Observable Plot.
8. **One package per kind,** by licence (MIT, BSD, ISC, Apache-2.0, or MPL-2.0 bundled unmodified), size, and whether
   it draws without the network and prints. A GPL package is not used.
9. **A host makes the packs it wants built in once, in its configuration, and no document imports them.** A pack the
   configuration lists as ambient (spec §6.3) gives every document its names, with no import in its frontmatter. The
   blog of compmd.dev lists the first list's packs once, in `website/blog/compmd.config.json`, and every post uses
   their components as it uses the site's own pack today:

   ```json
   {
     "packs": {
       "../app": { "trust": "trusted", "ambient": true },
       "@comp-markdown/card": { "ambient": true },
       "@comp-markdown/chart": { "ambient": true },
       "@comp-markdown/music": { "ambient": true },
       "@comp-markdown/embed-youtube": { "trust": "permissive", "ambient": true, "network": ["https://www.youtube-nocookie.com"] }
     }
   }
   ```

   A minimal host lists none and carries none: the weight is each host's choice, not CompMD's. What a document that
   uses none of them pays their signatures only (G10, fixed).
10. **One pack per component** (CMD-11, decided 2026-11-20): each component of the first list is a pack of its own,
    `@comp-markdown/<component>` in `compmd-packs/<component>/`, as the Mermaid and wireframe packs are, with a version
    of its own, so a fix to one releases that one only, and a host lists exactly the components it wants, each at its
    own level (an embed permissive, the rest strict). The embeds are one pack per provider (`embed-youtube`, …), sharing
    the permissive frame's code of CMD-332 as a library. The basics stay one pack, `@compmd/basics`, built into the
    sandbox (decision 6). With about 25 packs, each one's release is a tag `<pack>-v<version>`, which one workflow
    publishes for every pack, rather than a workflow each ([releases](releases.md), decision 8).
11. **The frames compile WebAssembly again** (decided 2026-11-20): `graphviz` and `jq` are WebAssembly
    (`@viz-js/viz`, `jq-wasm`), so both frame documents' policies get `'wasm-unsafe-eval'` back, in the sprint that
    brings them (CMD-324). [WebAssembly in the frames](webassembly-in-the-frames.md) found it gives a component
    nothing it doesn't have already: WebAssembly reaches only what the frame's scripts reach. 0.9.0 had taken it out
    only because nothing in the frames needed it ([the host's code highlighter](host-highlighter.md), decision 7).

## The first list

| # | Component | Written as | Wraps | Runs | Why |
|---|---|---|---|---|---|
| 1 | `tabs` (and code groups) | its own code | — | built in (decision 6) | 10 of 11 frameworks ship tabs. A slot's items carry their params (spec §7.4), so each tab's label is its item's; a code group is tabs whose items are code blocks. Its fallback is every tab in order, under its label. |
| 2 | `details` (a collapsible) | its own code | — | built in | 9 of 11 frameworks. Raw HTML's `<details>` exists, but HTML is opaque (spec §2.3): no components or Markdown structure inside it. Its fallback is its content, under its summary. |
| 3 | `steps` | its own code | — | built in | 7 of 11 frameworks, and how-tos in posts. Numbers its items and draws the line between them; its fallback is the items in order. |
| 4 | `card`, `link-card`, `gallery` | `compmd` definitions on `box`, in a `grid` | — | no code | 8 of 11 frameworks ship cards. A gallery is a grid of cards, each an image, a title and a description, so it needs no code and its images are the document's. |
| 5 | `figure` | a `compmd` definition | — | no code | Posts and newsletters caption their images: an image and its caption, the caption its text alternative too. |
| 6 | `badge` | a `compmd` definition, inline | — | no code | 6 of 11 frameworks; "new", "beta", a version. |
| 7 | `sticky-note` | a `compmd` definition | — | no code | A note pinned on the page: a tinted box, tilted, with a shadow, placed with `inset-*` (spec §9.2) where it should sit. |
| 8 | `timeline` | `compmd` definitions on `box` | — | no code | Dates and what happened, in a post or notes; a list of items is enough. |
| 9 | `rating` | its own code | — | strict | Stars from a value, half stars included, read-only: a few dozen lines, no package worth one. |
| 10 | `chart` | a leaf over data rows | `@observablehq/plot` (ISC, 95 KB) | strict | Charts from `md.data` are what data imports are for (spec §8.6). [Charts](#charts) has the params and the comparison. |
| 11 | `calendar` | a leaf over data rows | — | strict | A month, or several, with dates marked and their events listed, from data: about 150 lines over `Intl.DateTimeFormat`; `vanilla-calendar-pro` (MIT, 25 KB) if it grows. |
| 12 | `weather` | a leaf over a data import | Meteocons icons (MIT) | strict | A forecast: the document imports it from Open-Meteo, which needs no key, and the reader allows the origin once (decision 5). [Weather](#weather). |
| 13 | `graphviz` | a code renderer for ```` ```dot ```` | `@viz-js/viz` (MIT, 486 KB) | strict | The other text-to-diagram language people write, as Mermaid is; WebAssembly, which the frames allow again (decision 11), no network. |
| 14 | `data-table` | a leaf over data rows | `gridjs` (MIT, 16 KB) | strict | A table of data the reader sorts and searches. |
| 15 | `jq` | a code renderer for ```` ```jq ```` | `jq-wasm` (MIT, real jq 1.8, 330 to 500 KB) | strict | A query over JSON with `render=both`: the query, then its output; with an input the reader changes, a playground in the page. [jq](#jq). |
| 16 | `music` | a code renderer for ```` ```abc ```` | `abcjs` (MIT, 148 KB) | strict, its sound font in the pack | Sheet music from text, which a plain renderer shows as its source, and played offline. [Music](#music). |
| 17 | `chord` | a leaf | `svguitar` (MIT, 68 KB) with `@tombatossals/chords-db` (MIT) | strict | A guitar or ukulele chord diagram from its name (`Am7`) or a fingering; transposed with `chord-symbol` (MIT). |
| 18 | `chord-sheet` | a code renderer for ```` ```chordpro ```` | its own parser | strict | Lyrics with chords over them, in ChordPro (`[C]` inline chords, the main directives), transposable. Its own parser, since the complete one, `chordsheetjs`, is GPL-2.0. |
| 19 | `guitar-tab` | a leaf over a Guitar Pro file, or a code renderer for ```` ```alphatex ```` | `@coderline/alphatab` (MPL-2.0, 311 KB, its music font 313 KB) | strict, drawing only | Tabs and scores from Guitar Pro 3 to 8, MusicXML or alphaTex. Playing them waits for G6. |
| 20 | `qr-code` | a leaf | `qrcode` (MIT, 9 KB) | strict | A link on paper: newsletters, posters and slides, printed. |
| 21 | the embeds | leaves, one per provider | — | permissive | YouTube, Spotify, Vimeo, SoundCloud, a Bluesky or X post, CodePen. [Embeds](#embeds) has which, why and how. |

How they are grouped into packs is the packs' design (CMD-11): the basics are `@compmd/basics` (decision 6); the
`compmd` definitions can be one pack, the music components another, and the embeds are their own, permissive pack.

## Charts

**The params**, over `md.data` rows, the label in brackets its title and accessible name:

```md
:: chart[Sales by month] {type=bar rows=(md.data.sales) x=month y=amount color=region}
```

| Param | Value |
|---|---|
| `type` | `bar`, `line`, `area`, `scatter`, `histogram`, `heatmap`, `box`, `pie`, `donut` (a bubble chart is `scatter` with `size`) |
| `rows` | the records |
| `x`, `y` | field names; a histogram takes `x` alone, a heatmap `x`, `y` and `value` |
| `color` | a field splitting the rows into series: the legend and the colours |
| `stack` | `grouped`, `stacked` or `normalized`, for bars and areas; stacked when `color` is set |
| `size` | a field for a bubble's radius |
| `value` | a field for a heatmap's cells and a pie's slices |
| `horizontal` | bars and boxes across instead of up |
| `xlabel`, `ylabel` | the axes' titles; the field's name otherwise |
| `legend` | `auto` or `none` |
| `height`, `description` | the chart's height; a longer text alternative than its label |

**Observable Plot draws them.** Each param is a Plot channel or option almost one to one (`fill` for `color`, `r` for
`size`, its stack transform and facets for stacked and grouped, `bin`, `cell`, `box`), it takes records as they are,
it draws SVG, which prints, follows the colour scheme with CSS (`currentColor`), writes ARIA labels, and fetches and
evaluates nothing: 95 KB gzipped with the parts of d3 it uses. Pie and donut, which Plot lacks, are drawn with
d3-shape's `pie` and `arc`, already in the bundle.

**Left out of `chart`:** radar, sankey, treemap, funnel, gauge and candlestick take data of other shapes (links, a
hierarchy, open-high-low-close), so each is a component of its own when it's asked for, and Apache ECharts (Apache-2.0,
199 KB for four kinds, 368 KB for all) draws them all if one library is wanted for them.

| Library | Licence | Gzipped | Output | Kinds | Data | Verdict |
|---|---|---|---|---|---|---|
| Observable Plot 0.6.19 | ISC | 95 KB | SVG | bar, line, area, scatter, histogram, heatmap, box, bubble, maps; no pie | records | chosen |
| Apache ECharts 6.1.0 | Apache-2.0 | 199 to 368 KB | canvas or SVG | all of these, radar to calendar heatmaps | `dataset` with one series per colour | for the other kinds |
| Vega-Lite 6.5.0 | BSD-3 | 273 KB | SVG or canvas | most, no radar or sankey | records | compiles expressions with `Function`, so it needs `vega-interpreter` in a frame |
| billboard.js 4.1.5 | MIT | 105 KB | SVG | radar, gauge, funnel, treemap; no heatmap or box | wide columns | `Function` in its bundle |
| Chart.js 4.6.1 | MIT | 51 to 71 KB | canvas | the usual ones | one dataset per series | canvas only: prints at screen resolution, no text to read |
| ApexCharts 8.9.1 | its own | 282 KB | SVG | many | | not open source: a platform that lets people configure charts needs a paid OEM licence |

## Music

- **Playing offline.** abcjs's synth fetches one mp3 per note from a URL (by default its author's GitHub Pages), which
  a strict frame refuses. The pack ships the notes instead: a sound font as an ES module (the MIDI.js format, each
  note's mp3 as a `data:` URI), decoded with Web Audio's `decodeAudioData` and put in abcjs's note cache, built from
  `abcjs/src` so the cache is reachable. One instrument trimmed to the notes music uses is about 0.5 to 1 MB
  (FluidR3's piano is 1.8 MB gzipped for all 88 notes), and loads only when the reader presses play.
- **The reader starts it.** A browser keeps audio suspended until the reader acts, and the gesture counts in the frame
  it happens in, so the play button is the component's own, in its frame; `sandbox="allow-scripts"` is enough, and no
  `allow="autoplay"` is needed.
- **Guitar Pro files** are drawn by alphaTab with its workers off and its music font from the pack. Its player runs
  its synthesizer in a web worker, which frames don't allow (`worker-src 'none'`), so playing them waits for G6.

## jq

`jq-wasm` is jq 1.8 compiled to WebAssembly (MIT, 180,000 downloads a week); its `jq-wasm/inline` entry has the
WebAssembly in it, so nothing is fetched, and it needs no worker. The JSON comes from a data import or a param:

````md
```jq {render=both input=(md.data.releases)}
map(select(.stable)) | .[0].version
```
````

## Weather

The document imports the forecast; the component draws it, with an icon per WMO weather code (a dozen Meteocons SVGs,
MIT, in the pack):

```yaml
md:
  data:
    - from: https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=3
      as: forecast
```

Open-Meteo's data is CC BY 4.0, so the component credits it; its free use is for non-commercial sites, under 10,000
calls a day (G8).

## Embeds

**First:** YouTube, Spotify, Vimeo, SoundCloud, a Bluesky or X post, and CodePen, the ones posts and newsletters embed
most, by what blog and newsletter platforms offer (a judgement: no data on it was found). Mastodon waits: each
instance is an origin of its own, so no fixed list of origins can allow it.

**How each works:** a leaf takes the share URL the author copies (`:: embed-youtube[Talk: CompMD in ten minutes]
{url="https://youtu.be/…"}`), shows a facade (the label, the provider's name, a play button), and opens the provider's
player in a frame only when the reader clicks it, so nothing reaches the provider before. Its fallback is a link.

| Provider | Share URL → player | Origin to allow | Player's `allow` | Facade |
|---|---|---|---|---|
| YouTube | `youtu.be/ID` → `https://www.youtube-nocookie.com/embed/ID` | `https://www.youtube-nocookie.com` | `accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share`, `allowfullscreen`, `referrerpolicy="strict-origin-when-cross-origin"` | its label, or the thumbnail `i.ytimg.com/vi/ID/hqdefault.jpg` (a request to Google, so only as the reader's choice) |
| Spotify | `open.spotify.com/{track,album,playlist,episode,show}/ID` → `https://open.spotify.com/embed/…/ID` | `https://open.spotify.com` | `autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture` | its label; 152 or 352 px tall |
| Vimeo | `vimeo.com/ID` → `https://player.vimeo.com/video/ID?dnt=1` | `https://player.vimeo.com` | `autoplay; fullscreen; picture-in-picture; clipboard-write; encrypted-media; web-share` | its label |
| SoundCloud | a track's URL → `https://w.soundcloud.com/player/?url=…` | `https://w.soundcloud.com` | `autoplay; encrypted-media` | its label |
| Bluesky, X | a post's URL → `https://embed.bsky.app/embed/…`, `platform.twitter.com` | `https://embed.bsky.app`, `https://platform.twitter.com` | none | the post's text, which the author quotes in the label: the facade is the post itself |
| CodePen | `codepen.io/U/pen/S` → `https://codepen.io/U/embed/S?default-tab=result` | `https://codepen.io` | `allowfullscreen` | its label |

**What the providers' terms ask:** YouTube's let a player be opened by its play button (III.F.3.b) but not changed or
blocked (III.I.6), and the page says it is YouTube's (III.F.2.a); a facade that opens the unchanged player on a click
does both. Vimeo's, Spotify's and SoundCloud's terms on facades are to read before the pack is built (G2).

## Later

| Kind | Packages | Waits for |
|---|---|---|
| Image zoom, a lightbox, a before-and-after slider | `medium-zoom`, `photoswipe`, `img-comparison-slider` (all MIT) | a component reading the document's images (G1, CMD-266) |
| 3D models | `@google/model-viewer` (Apache-2.0) | the same, for model files, and WebGL in the frames |
| Maps | `leaflet` (BSD-2), `maplibre-gl` (BSD-3), with `pmtiles` | tiles from the network (a permissive pack), or a tile file in the pack (G3) |
| Playing Guitar Pro files | `@coderline/alphatab`'s player | a synthesizer without a worker (G6) |
| Mastodon posts | — | an origin per instance (Embeds) |
| A live code playground | `@codesandbox/sandpack-react` | a hosted bundler, in a frame |
| An API field table, a tooltip, a banner, a file tree | — | 2 to 4 frameworks each, docs only (decision 1); `kbd` is in raw HTML's allowlist already |

## Gaps

- **G1. A component and the document's images.** A strict frame loads images only from `data:` and `blob:`, so a
  component written in code can't show an image the document links. CMD-266 designs how the host hands them in.
- **G2. Embeds in a permissive frame.** A permissive frame's second policy allows frames from `strict.html` only
  (`npm-packages/sandbox/src/frame/narrow.ts`), so the embeds pack needs it to add the pack's configured origins to
  `frame-src`; and the host's frame for the pack has to delegate the players' features (`allow="autoplay;
  encrypted-media; fullscreen; picture-in-picture"`) for the frame to pass them on. Before building it: Vimeo's,
  Spotify's and SoundCloud's terms on facades.
- **G3. Maps.** A map's tiles come from a server, which only a permissive pack reaches; a tile file in the pack (PMTiles)
  makes a small offline map possible, its size to measure.
- **G5. The numbers.** Weekly downloads count every install, including those pulled in by other packages (cytoscape and
  elkjs come with Mermaid), so they say what is used, not what authors chose; no study of what docs authors use most
  was found.
- **G6. A synthesizer without a worker.** alphaTab's player needs a web worker; its `AlphaSynth` may run in the frame
  itself with a ScriptProcessor output, to prototype, or the frames allow a worker from their own origin, which is a
  change to their policy and its design (*WebAssembly in the frames* weighed the same for code).
- **G7. ChordPro's licence.** The complete parser, `chordsheetjs`, is GPL-2.0, which the pack can't ship (decision 8),
  so `chord-sheet` parses the subset posts use itself.
- **G8. Weather's terms.** Open-Meteo's free use is non-commercial; a host that is commercial needs its paid plan, or
  another source the document names, since the document says where its data comes from.
- **G9. Audio in a sandboxed frame.** Starting Web Audio from a click in a cross-origin frame with
  `sandbox="allow-scripts"` works in Chrome; WebKit's behaviour is to test in the music pack's browser tests. FluidR3's
  sound font is CC BY 3.0, which the pack credits.

- **G10. An ambient pack's code costs documents that don't use it.** The processor lists as a document's host-language
  components every one it can name, an ambient pack's included, used or not (`host_components`,
  `crates/processor/src/resolve.rs`). So the sandbox compiles each of them for every view, and the blog gives every post
  its page script, frames and all, since a post is live when that list isn't empty (`npm-packages/blog/src/render.ts`):
  that is why the blog's plain first post was given one in its first round. Decision 9 needs the list to hold only the
  components the document's expanded tree uses (its examples' included), so a post that uses none loads nothing:
  CMD-268, before the packs are many. **Fixed:** the processor now lists the components the document uses: those its
  body invokes or draws a code block with, in any branch whatever the inputs, and those the definitions it uses
  invoke, each example's apart (`crates/processor/src/resolve/reached.rs`). Not the expanded tree's: a branch the
  last render didn't show, which an input can show, would have no code.

Round 1's G4, tabs and collapsibles in the core pack, is decision 6 now.

## What the review changed

From CR-19's comments, in round 2 and, for c30, round 3:

| Comment | Where it is |
|---|---|
| c1: components that need the network | Decision 5, replacing "only what runs in a strict frame"; the weather widget (12) and the embeds (21) |
| c2, c18: music, guitar chords, Guitar Pro, playing | `music` (16), `chord` (17), `chord-sheet` (18), `guitar-tab` (19); [Music](#music); G6, G7, G9 |
| c3: weather, calendar, sticky note, gallery, rating | `weather` (12), `calendar` (11), `sticky-note` (7), `gallery` (4), `rating` (9) |
| c4: a rich chart with an ergonomic API | Decision 7 and [Charts](#charts): the params, then the library |
| c5: a jq renderer | `jq` (15) and [jq](#jq) |
| c6: the document's images | G1, and the task CMD-266 |
| c7: tabs and the rest built in, not in the spec | Decision 6, which settles round 1's G4 |
| c8, c24: a permissive embeds pack, with the details | `embeds` (21), [Embeds](#embeds), G2 |
| c30: the packs built in for a host, configured once, as for the blog | Decision 9, and G10, what it needs first |

## The research

Read on 2026-11-19. Downloads are npm's for the week of 12 to 18 November 2026
(`api.npmjs.org/downloads/point/last-week/<name>`), licences from the registry, sizes gzipped from Bundlephobia (the
package's entry point) or measured from a build of the parts a component uses.

### What documentation frameworks ship

Y: built in; P: an official option or partly; blank: none; ?: not checked.

| Kind | Docusaurus | Nextra | Starlight | Mintlify | Fumadocs | VitePress | GitBook | Docus | Material for MkDocs | Rspress | sphinx-design | Y |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Callout | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 11 |
| Tabs | Y | Y | Y | Y | Y | | Y | Y | Y | Y | Y | 10 |
| Collapsible | Y | ? | Y | Y | Y | Y | Y | Y | Y | ? | Y | 9 |
| Cards | P | Y | Y | Y | Y | | Y | Y | Y | | Y | 8 |
| Steps | | Y | Y | Y | Y | | Y | Y | | Y | ? | 7 |
| Badge | | | Y | Y | | Y | ? | Y | | Y | Y | 6 |
| Code group | P | P | P | Y | Y | Y | | Y | P | P | Y | 5 |
| Mermaid | P | Y | | Y | Y | | Y | | Y | ? | | 5 |
| Icons | | | Y | Y | | | ? | Y | Y | | Y | 5 |
| Math | P | Y | | ? | Y | P | Y | ? | Y | ? | | 4 |
| File tree | | Y | Y | Y | Y | | | P | | | | 4 |
| Columns | | | P | Y | | | Y | | Y | | Y | 4 |
| API fields | | P | | Y | Y | | P | Y | | | | 3 |
| Banner | | Y | | Y | Y | | | | | | | 3 |
| Captioned image | | | | Y | P | | | | Y | | | 2 |
| Tooltip | | | | Y | | | | | Y | | | 2 |
| Video or embed | | | | P | | | Y | Y | | | | 2 |

Sources: [Docusaurus](https://docusaurus.io/docs/markdown-features), [Nextra](https://nextra.site/docs/built-ins),
[Starlight](https://starlight.astro.build/components/using-components/), [Mintlify](https://mintlify.com/docs/components),
[Fumadocs](https://fumadocs.dev/docs/ui/components), [VitePress](https://vitepress.dev/guide/markdown),
[GitBook](https://gitbook.com/docs/llms.txt), [Docus](https://docus.dev/en/essentials/components),
[Material for MkDocs](https://squidfunk.github.io/mkdocs-material/reference/),
[Rspress](https://rspress.rs/ui/components/index.md), [sphinx-design](https://sphinx-design.readthedocs.io/en/latest/).

What authors ask for: Starlight's [discussion of more built-in components](https://github.com/withastro/starlight/discussions/1491)
(a quiz, click-to-zoom images, cards, `kbd`, math, captions and video; Steps, FileTree and Badge were added after it),
VitePress's [code groups](https://github.com/vuejs/vitepress/issues/728), and the
[steps](https://github.com/withastro/docs/discussions/885) Starlight added later. Cloudflare's
[style guide](https://developers.cloudflare.com/style-guide/build-the-page/components/) lists tabs, details, link
cards, badges, YouTube, package-manager tabs, glossary tooltips and directory listings as its most used.

The widgets of the first list's second half (weather, a calendar, a sticky note, a rating) are no framework's and no
popular package's: they are a page's own, written in a few dozen to a few hundred lines.

### Component packages for documents

| Kind | Package | Weekly downloads | Licence | Framework | Gzipped | Needs |
|---|---|---|---|---|---|---|
| Charts | `chart.js` | 17.7 M | MIT | none | 70 KB | canvas |
| | `echarts` | 11.2 M | Apache-2.0 | none | 199 to 368 KB | canvas or SVG |
| | `vega-lite` | 2.3 M | BSD-3 | none | 273 KB with Vega | SVG or canvas; `Function` |
| | `@observablehq/plot` | 1.5 M | ISC | none | 95 KB with d3 | SVG |
| | `recharts` | 70.6 M | MIT | React | 177 KB | SVG |
| Diagrams | `mermaid` | 28.1 M | MIT | none | 159 KB | (the Mermaid pack) |
| | `@viz-js/viz` | 571 K | MIT | none | 486 KB | WebAssembly |
| | `@terrastruct/d2` | 127 K | MPL-2.0 | none | 6 MB | WebAssembly |
| | `@excalidraw/excalidraw` | 1.6 M | MIT | React | 353 KB | canvas |
| Tables | `gridjs` | 84 K | MIT | none | 16 KB | |
| | `tabulator-tables` | 271 K | MIT | none | 105 KB | |
| | `ag-grid-community` | 5.7 M | MIT | none | 392 KB | |
| Music | `abcjs` | 176 K | MIT | none | 148 KB | SVG; a sound font, fetched by default |
| | `@coderline/alphatab` | 30 K | MPL-2.0 | none | 311 KB, a 313 KB font, a 977 KB sound font | workers to play |
| | `tone` | 447 K | MIT | none | 83 KB | a worker for its clock unless told otherwise |
| Chords | `svguitar` | 2.2 K | MIT | none | 68 KB | SVG |
| | `vexchords` | 0.9 K | MIT | none | 26 KB | SVG |
| | `@tombatossals/chords-db` | 5.8 K | MIT | none | data | |
| | `chord-symbol` | 5.4 K | MIT | none | 18 KB | |
| | `chordsheetjs` | 6.4 K | GPL-2.0 | none | 95 KB | |
| jq | `jq-wasm` | 181 K | MIT | none | 332 KB (and its loader), 503 KB inlined | WebAssembly |
| | `jq-web` | 56 K | ISC | none | 978 KB | WebAssembly, fetched beside the script |
| Calendars | `@fullcalendar/core` and `daygrid` | 4.3 M | MIT | none | 53 KB | |
| | `vanilla-calendar-pro` | 135 K | MIT | none | 25 KB | |
| | `cally` | 58 K | MIT | web components | 10 KB | a date picker, no events |
| QR codes | `qrcode` | 43.4 M | MIT | none | 9 KB | |
| Timelines | `vis-timeline` | 312 K | Apache-2.0 or MIT | none | 122 KB | |
| Images | `medium-zoom` | 755 K | MIT | none | 3 KB | the images |
| | `photoswipe` | 870 K | MIT | none | 17 KB | the images |
| | `img-comparison-slider` | 61 K | MIT | web component | 4 KB | the images |
| Maps | `leaflet` | 12.0 M | BSD-2 | none | 43 KB | tiles |
| | `maplibre-gl` | 11.4 M | BSD-3 | none | 300 KB | WebGL, tiles |
| Embeds | `lite-youtube-embed` | 124 K | Apache-2.0 | web component | 2 KB | a frame, the network |
| Icons | `@meteocons/svg` | 12 K | MIT | none | each icon its own SVG | |
| 3D | `@google/model-viewer` | 955 K | Apache-2.0 | web component | 73 KB, and Three.js | WebGL, the model |

What each package needs (canvas, WebGL, the network, workers) is from its documentation and its source as read, not
tested in a frame; the music pack's tests settle what playing needs.
