---
md:
  import:
    - from: "@acme/charts@1"
      names: [chart]
---

`@acme/charts` names three library packs in `peerDependencies`: `@kit/three`, which this host has no release of; `@kit/widgets`, a pack with a document, not a library pack; and `@bad/plot`, which the host denies. Its import gets W121 for each. It is kept: `chart`, which uses none of them, renders.

:: chart[Chart]
