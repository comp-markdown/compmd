---
md:
  export:
    pack: "@example/old"
    version: "1.0.0"
    names: [gauge]
---

```js component=gauge
export default function mount(target) {
  target.textContent = "an old gauge";
}
```

```compmd component=gauge signature
params:
  value: number
```
