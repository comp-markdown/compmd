---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./walks.yaml
      as: walks
---

# Three walks

:: gallery[Three walks around the harbour] {rows=(md.data.walks)}
