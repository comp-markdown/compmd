---
md:
  import:
    - from: "@acme/charts@2"
      names: [legend]
---

The pack needs Svelte 5.57 or later and three.js, which this host doesn't have, so its import gets W121 for each; d3 7.9 meets its range. The import is kept: `legend`, which needs neither, renders.

:: legend[Legend]
