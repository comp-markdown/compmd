# The CompMD logo

The Markdown mark (a frame, an M and an arrow, which its author dedicated to the public domain), its arrow become blocks of a page, and one component placed over the frame's corner, in the accent color. The name beside it is drawn in the mark's strokes, with "MD" in the accent color, so the files need no font.

| File | What it is | For |
|---|---|---|
| `compmd-light.svg`, `compmd-dark.svg` | The mark and the name | READMEs and the spec, on a light or a dark background |
| `mark-light.svg`, `mark-dark.svg` | The mark alone, wide | Where the name is written beside it, such as the playground's header |
| `mark-mono.svg` | The mark in one color | Print, stamps, places with one ink |
| `icon.svg` | The mark, square, without the inner blocks so the corner tile reads at 16 px; light or dark as the browser is | Favicons, avatars |

Colors: ink `#1b2430` and accent `#3557c4` on light; ink `#e4e9ef` and accent `#8da6ee` on dark: the playground's own.

On GitHub, a `<picture>` picks the file for the reader's theme:

```html
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="logo/compmd-dark.svg">
  <img src="logo/compmd-light.svg" alt="CompMD" width="300">
</picture>
```
