```compmd component=card {.card padding=s md:padding=m}
slots: [default]
---
{{md.slot}}
```

```jsx component=chart runtime=react@19 {height=16rem}
export default function Chart() { return null; }
```

```compmd component=chart signature {height=16rem}
params: {}
```

```compmd component=broken {padding=s padding=m title="x}
---
Text.
```
