---
md:
  import:
    - from: "@acme/look"
      names: [masthead]
      fonts: [{ name: harbour-serif, as: serif }]
  fonts:
    harbour-serif:
      fallback: heading
      files:
        - { from: ./fonts/harbour-serif.woff2 }
        - { from: ./fonts/harbour-serif-bold.woff2, weight: bold }
    unused:
      files:
        - { from: ./fonts/unused.woff }
---

::: box {font-family=harbour-serif}
Our own serif.
::: /box

::: box {font-family=serif md:font-family=mono}
The pack's serif, renamed.
::: /box

:: masthead[The Harbour]
