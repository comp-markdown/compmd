---
md:
  import:
    - from: "@acme/kit@1"
      names: [chip]
    - from: "@huge/pack"
      names: [big]
    - from: "@acme/kit@2"
      names: [tag]
---

The denied version and the refused pack render their fallbacks, without F002; the version the host allows renders.

:: chip[Chip]

:: big[Big]

:: tag[Tag]
