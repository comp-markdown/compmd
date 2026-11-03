---
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
---

# Where a page's parts come from

The page, its documents and the sandbox's frames come from one server:

```mermaid {alt="The server gives the page, the frames and the built modules"}
graph LR
  Server --> Page
  Server --> Frames
  Server --> Modules
```
