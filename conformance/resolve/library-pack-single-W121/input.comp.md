---
md:
  import:
    - from: "@acme/charts@1"
      names: [chart]
    - from: "@acme/maps@1"
      names: [map]
---

`@acme/charts` names the library pack `@kit/three` at `^0.170`, and `@acme/maps` at `^0.171`. No version matches both, and `@kit/three` can't have two copies in one sandbox, so the first pack to ask, `@acme/charts`, decides: 0.170.2. The import of `@acme/maps` gets W121, naming both ranges and that version. It is kept: `map`, which doesn't use the library, renders.

:: chart[Chart]

:: map[Map]
