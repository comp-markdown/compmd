---
md:
  version: 1
  export:
    names: [sparkline]
---

```js component=sparkline
export default function mount(target, { params, fallback }) {
  target.textContent = fallback.text;
  return { update: (next) => { target.textContent = String(next.params.values.length); } };
}
```

```compmd component=sparkline signature
params:
  values: array
```
