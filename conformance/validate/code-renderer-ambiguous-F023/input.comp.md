---
md:
  import:
    - from: ./one.comp.md
      names: "*"
    - from: ./two.comp.md
      names: "*"
---

```shout {alt="Hi"}
hi
```

A local definition is a level above wildcard imports, so its language has one renderer:

```compmd component=whisper
renders: whisper
---
{{code}}
```

```whisper {alt="Quiet"}
quiet
```
