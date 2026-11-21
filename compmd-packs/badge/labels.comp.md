---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
  export:
    names: [bug, feature, chore]
---

# A project's labels

Three badges with their look fixed, defined once: a document imports this file and writes `:bug[bug]`, `:feature[new]` or `:chore[chore]`, nothing else.

```compmd component=bug
---
:badge[**{{md.fallback}}**]{background="#ffebe9" border="1px solid #cf222e"}
```

```compmd component=feature
---
:badge[**{{md.fallback}}**]{background="#dafbe1" border="1px solid #1a7f37"}
```

```compmd component=chore
---
:badge[*{{md.fallback}}*]{background=subtle border=strong border-radius=s}
```
