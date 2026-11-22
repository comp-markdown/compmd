---
md:
  fonts:
    body:
      files: [{ from: ./a.woff2 }]
    no-files:
      fallback: serif
    mixed:
      files:
        - { from: "https://fonts.example.com/m.woff2" }
        - { from: ./m.ttf }
        - { from: ../m.woff2 }
        - { from: ./m.woff2, weight: 600 }
        - { from: ./m.woff2 }
        - { from: ./m-again.woff2 }
    "bad name":
---

::: box {font-family=nofont}
A font nothing declares.
::: /box

::: box {font-family=no-files}
A font dropped for its declaration, without a diagnostic of its own.
::: /box
