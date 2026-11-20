---
md:
  import:
    - from: "@spike/qr-svg@0"
      names: [qr-svg]
---

# qrcode, wrapped

Each use's box has a magenta border, so the box shows beside what is drawn in it.

**No size:** the box is the column's width, as tall as the drawing.

:: qr-svg[No size] {value="https://compmd.dev/" border="2px solid #e5007d"}

**Fit to width,** `width=10rem`:

:: qr-svg[Fit to width] {value="https://compmd.dev/" width=10rem border="2px solid #e5007d"}

**Fit to height,** `height=6rem`: the box keeps the column's width.

:: qr-svg[Fit to height] {value="https://compmd.dev/" height=6rem border="2px solid #e5007d"}

**Fit to a wide box,** `width=16rem height=6rem`:

:: qr-svg[Fit to a wide box] {value="https://compmd.dev/" width=16rem height=6rem border="2px solid #e5007d"}

**Fit to a tall box,** `width=6rem height=12rem`:

:: qr-svg[Fit to a tall box] {value="https://compmd.dev/" width=6rem height=12rem border="2px solid #e5007d"}

**A box by its aspect ratio,** `width=12rem aspect-ratio=2/1`:

:: qr-svg[A box by its aspect ratio] {value="https://compmd.dev/" width=12rem aspect-ratio=2/1 border="2px solid #e5007d"}
