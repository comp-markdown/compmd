# Type in a box: its font, size, weight and text colour, and fonts of its own

**Status: proposed (2026-11-21, CMD-370).** How a document, or a `compmd` definition, sets the type of a box's text:
a badge in bold small type, a pull quote larger and in the heading font, a muted caption, a code-like label in the
monospace font. Found in CR-51: a project's own badges, defined once, can fix their colours, borders and radius, but
not their type. CompMD's common params (spec §9.2, §9.4) have surfaces, borders, radius, shadow and spacing, and
nothing for text, so a definition has only Markdown's `**bold**` and `*italic*`, and a box's text is always the
page's.

**Terms.** A box's **text** is everything it holds that is text: its paragraphs, headings, list items and inline
content, and the boxes inside it, which inherit it as CSS's inherited properties do. A **type token** is a word a
type param takes (`l`, `bold`, `heading`, `muted`), a CSS custom property with a default, as the visual tokens are
(§9.4).

## Decisions

1. **Ten common params, named after their CSS properties** (§9.2's naming rule), on every block and on an inline
   box, each taking breakpoints:

   | Param | Takes | Tokens |
   | --- | --- | --- |
   | `font-size` | a token or a length (`em`, `rem`, `%`, `px`) | `s` (0.875em), `m` (1em), `l` (1.25em), `xl` (1.5em) |
   | `font-weight` | a token | `normal`, `bold` |
   | `font-style` | a token | `normal`, `italic` |
   | `font-family` | a token, or a font the document or a pack declares (decision 7) | `body`, `heading`, `mono` |
   | `color` | a token or a CSS colour | `muted`, `accent` |
   | `text-align` | a token | `start`, `center`, `end` |
   | `line-height` | a token or a plain number (`1.4`) | `tight` (1.2), `normal` (the page's), `loose` (1.8) |
   | `letter-spacing` | a token or a length in `em` | `tight` (-0.01em), `normal` (0), `wide` (0.05em) |
   | `text-transform` | a token | `none`, `uppercase`, `lowercase`, `capitalize` |
   | `text-decoration` | a token | `none`, `underline`, `line-through` |

   The size tokens are in `em`, so a badge in a heading is small for the heading, not for the page. `text-transform`
   changes how the text shows, not the text: `:badge[new]{text-transform=uppercase}` shows NEW, and a screen reader
   and a plain reader read "new", where text written in capitals is spelt out letter by letter. `text-decoration` is
   for what Markdown has no mark for (an underline that isn't a link); a link keeps its own, and Markdown's
   `~~…~~` stays the way to strike text a plain reader should see struck.
   - **Rejected: the names of fonts on the reader's device**, `font-family="Georgia, serif"`. A document is read
     wherever it is shown, and such a font is there or isn't, so the document looks different everywhere. The three
     tokens are the host's fonts, its body text, its headings and its code; a font of the document's own comes with
     its files (decision 7).
   - **Rejected: numeric weights**, `font-weight=600`. A host's font may have two weights; `bold` always shows.
2. **The tokens are custom properties with defaults, as §9.4's,** so a host's theme changes them:
   `--compmd-font-body` (`inherit`), `--compmd-font-heading` (`inherit`), `--compmd-font-mono`
   (`ui-monospace, monospace`), `--compmd-text-s` to `--compmd-text-xl` (the sizes above),
   `--compmd-leading-tight` and `-loose`, `--compmd-tracking-tight` and `-wide` (the values above),
   `--compmd-text-muted` (`color-mix(in srgb, currentColor 65%, transparent)`) and `--compmd-text-accent`
   (`var(--compmd-accent, #0969da)`). `body` and `heading` are the page's own fonts unless the host says otherwise.
3. **They are inherited, as CSS's are:** a box's text params reach everything inside it, nested boxes included,
   until one of them sets its own. Headings keep their sizes relative to the box's (`em`), so a pull quote in `l` has
   its heading larger too.
4. **Contrast: W126 only where the processor can tell.** A `color` and a `background` both written as CSS colours on
   one box, whose contrast is below 4.5:1 (WCAG's AA for text), is W126 at the box. Tokens are built from the text
   colour and suit light and dark; the processor can't know the page's colours, so it says nothing about them.
5. **A host-language component gets its box's type two ways** (proposed in CR-54, c9), both defaults it may ignore,
   as `fit` is:
   - **As CSS:** its outer box has them, so what it draws in the page inherits them, and a sandbox gives them to the
     root of the component's frame, resolved (the tokens as the host's fonts and sizes, a declared font loaded in
     the frame), so what it draws there inherits them too. A Mermaid diagram in `{font-family=heading font-size=s}`
     has its labels so, its theme saying `inherit`.
   - **As values,** in its context: `type: { family, size, weight, style, color, lineHeight, letterSpacing }`, the
     size in pixels and the colour resolved, `update` called when they change, as for `scheme`. A chart that draws
     on a `<canvas>`, which no CSS reaches, labels its axes with them.
6. **Print and plain readers:** printed as shown, a page with fonts of its own printed once they have loaded, as it
   waits for components to draw. A plain Markdown reader shows the text in its own type.
7. **Fonts of the document's own: from a pack, or next to the document,** each declared once with its files and a
   fallback token, never from a URL.
   - **Declared** in the frontmatter, `md.fonts`, each by a name and its files by weight and style:
     ```yaml
     md:
       fonts:
         harbour-serif:
           fallback: heading
           files:
             - { from: ./fonts/harbour-serif.woff2 }
             - { from: ./fonts/harbour-serif-bold.woff2, weight: bold }
             - { from: ./fonts/harbour-serif-italic.woff2, style: italic }
     ```
     A use writes `font-family=harbour-serif`. The name is the document's, as a component's is; one a token has, or
     one declared twice, is F013.
   - **A pack's fonts** are exported and imported as its components are: `export: { fonts: [harbour-serif] }`, and
     `import: [{ from: "@acme/harbour-look@1", fonts: [harbour-serif] }]`. A definition in the pack uses its own fonts
     by name. So a newsletter's look (its fonts and the components set in them) is one pack.
   - **The files are served as images are** (§8.6, CMD-362 and CMD-365): a pack's from its place, with its integrity,
     at the pack's trust; the document's from its folder, never leaving its root. `woff2` and `woff` only, within
     the pack's size cap (§6.4).
   - **The fallback** is what shows while the font loads (CSS's `font-display: swap`), where the host doesn't load
     fonts, and in a plain renderer: `body`, `heading` or `mono`, so the text is the right kind of text whatever
     happens.
   - **Rejected: fonts from a URL,** asked about by origin as remote images are (§15). A font is fetched on every
     view, is a better tracker than an image, and an origin the reader refuses changes the whole page's text, not
     one picture.
   - **Rejected: the name of a font on the reader's device**, decision 1.

## In use

````md
<!-- a project's badges, labels.comp.md -->
```compmd component=bug
---
:badge[{{md.fallback}}]{background="#ffebe9" border="1px solid #cf222e" color="#a40e26" font-size=s font-weight=bold}
```

```compmd component=api
---
:badge[{{md.fallback}}]{font-family=mono font-size=s border=subtle}
```

```compmd component=beta
---
:badge[{{md.fallback}}]{tone=accent font-size=s font-weight=bold text-transform=uppercase letter-spacing=wide}
```

<!-- a pull quote, its large lines kept close -->
::: box {font-family=heading font-size=l font-style=italic line-height=tight text-align=center padding-block=m}
The quay floods at high water, and has for three hundred years.
::: /box

<!-- a callout's muted footnote -->
::: box {color=muted font-size=s}
Tide times are for the harbour mouth; add ten minutes for the inner basin.
::: /box

<!-- a newsletter's masthead, centred, larger on a wide screen -->
::: box {text-align=center font-family=heading font-size=l md:font-size=xl}
**The Harbour Letter**, November
::: /box

<!-- a newsletter's look, from a pack: its masthead font, and its components set in it -->
---
md:
  import:
    - from: "@harbour/look@1"
      names: [masthead]
      fonts: [harbour-serif]
---

:: masthead[The Harbour Letter]

::: box {font-family=harbour-serif font-style=italic}
A word from the harbour master.
::: /box

<!-- a card whose title is in the heading font: a definition's heading box -->
```compmd component=story-card {padding=m border=subtle border-radius=m}
slots: [default]
---
::: box {font-family=heading}
{{md.slot}}
::: /box
```
````

Each reads well in a plain reader, which shows the text without the type, as it shows a box without its border.

## What changes

- **The spec:** §9.2 (the ten params), §9.4 (type tokens, their custom properties and defaults; `font-family` taking
  tokens and declared fonts), §2 and §6 (`md.fonts`, a pack's exported fonts, an import's `fonts`), §7.4 (`type` in
  the context, a frame's root), §8.6 (font files served as images are), §10 (W126; F009 for a font that isn't
  declared, F013 for one declared twice, F026 for a missing file), §12.3 (the expanded tree's fonts, for the
  renderer to load), the changelog; conformance for each param's grammar, a breakpoint, an inline box, the
  declarations and imports, and W126.
- **The processor:** the params in `Universal`, their kinds, the fonts' declarations, exports and imports, their
  files as sources, W126 on the expanded tree, `params.json` and `llms.txt`.
- **The renderer:** the CSS, a declared font's `@font-face` with its fallback, and browser tests in Chrome and
  WebKit (a badge's size in a heading, inheritance into a nested box, a theme changing a token, a pack's font
  loading and its fallback first, print once fonts load).
- **The sandbox and the host contract:** a frame's root given the box's type, `type` in the context, the bindings
  passing it.
- **The guides:** the Author's Guide's chapter on params; the badge demo's `labels.comp.md` using them.

## Gaps

- **G1. Weights between normal and bold:** a declared font maps its files to `normal` and `bold` (decision 1), so a
  font's light or semibold files have no use. Enough for text; a later change could add tokens.
- **G2. Licences:** a font's licence must let it be shipped in a pack (most open fonts' do, OFL); the spec can't
  check it, the pack tool could warn on a font file with no licence beside it.
- **G3. Contrast against tokens and the page:** W126 sees only two CSS colours on one box; a renderer could check
  the drawn colours, which no processor can.
