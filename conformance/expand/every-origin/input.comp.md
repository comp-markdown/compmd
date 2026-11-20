---
md:
  data:
    - from: https://status.example.com/today.json
      as: status
    - from: https://team.example.org/people.json
      as: team
---

Today: {{md.data.status.summary}}

On call: {{md.data.team.oncall}}
