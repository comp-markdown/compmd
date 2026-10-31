```js component=meter
export default function mount() {}
```

```compmd component=meter signature
renders: meter
params:
  tone: calm | loud = calm
  steps: number = 5
  label: string?
```

A leaf that writes only what's required gets every default; one that writes a param keeps it; a rendered code block
gets the defaults too.

:: meter[A calm meter] {code="3"}

:: meter[A loud meter] {code="3" tone=loud label="Loud"}

```meter {alt="Three"}
3
```
