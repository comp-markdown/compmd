---
md:
  export:
    names: [card, {name: badge, as: tag}]
---

```compmd component=card
params:
  title: string
slots: [default]
---
**{{title}}**: {{md.slot}}
```

```compmd component=badge
---
Badge.
```
