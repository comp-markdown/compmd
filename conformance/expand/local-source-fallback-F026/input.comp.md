---
md:
  data:
    - from: ./missing.json
      as: stats
      fallback: ./cached.json
---

{{md.data.stats.n}}
