---
md:
  inputs:
    docs: number = 72
---

# Release status

Two meters, drawn by a component the document defines itself, so they run strict wherever they are shown.

```compmd component=meter signature
params:
  value: number
  label: string
```

```jsx component=meter runtime=react@19
export default function Meter({ value, label }) {
  const shown = Math.min(Math.max(Number(value) || 0, 0), 100);
  return (
    <label style={{ display: "grid", gap: 4 }}>
      <span>{label}: <strong>{shown}%</strong></span>
      <progress max="100" value={shown} />
    </label>
  );
}
```

:: meter[Docs: 72%] {label="Docs" value=(md.input.docs)}

:: meter[Tests: 90%] {label="Tests" value=90}

:: number-input[Docs done, in percent: 72] {bind=docs}
