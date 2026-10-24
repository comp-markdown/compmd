---
md:
  data:
    - from: https://api.example.com/status.json
      as: status
    - from: https://allowed.example.org/data.json
      as: allowed
---

{{md.data.status}}
