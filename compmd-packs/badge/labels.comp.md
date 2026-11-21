---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
  export:
    names: [bug, feature, chore, beta]
---

# A project's labels

Four badges with their look fixed, defined once: a document imports this file and writes `:bug[bug]`, `:feature[new]`, `:chore[chore]` or `:beta[beta]`, nothing else.

```compmd component=bug
---
:badge[{{md.fallback}}]{background="#ffebe9" border="1px solid #cf222e" color="#a40e26" font-size=s font-weight=bold}
```

```compmd component=feature
---
:badge[{{md.fallback}}]{background="#dafbe1" border="1px solid #1a7f37" color="#116329" font-size=s font-weight=bold}
```

```compmd component=chore
---
:badge[{{md.fallback}}]{background=subtle border=strong border-radius=s color=muted font-size=s font-style=italic}
```

```compmd component=beta
---
:badge[{{md.fallback}}]{tone=accent font-size=s font-weight=bold text-transform=uppercase letter-spacing=wide}
```
