---
price: 12
---

```compmd component=badge
params:
  label: string
  count: number = 0
---
**{{label}}** {{count}}
```

:: badge[x] {label=(md.doc.price * 2)}

:: badge[y] {label="ok" count=(md.doc.price + "x")}

:: badge[z] {label=(md.doc.nope.size() > 0 ? "a" : "b")}

:: badge[w] {label=(undefined_name)}

:: badge[v] {label=(1 + 2)}
