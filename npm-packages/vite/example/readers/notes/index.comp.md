---
md:
  export:
    pack: "@example/notes"
    version: "1.0.0"
    names: [aside]
---

A pack only the readers' page lists: its configuration's, not the app's.

```compmd component=aside
params:
  title: string = Aside
---
**{{title}}**
```
