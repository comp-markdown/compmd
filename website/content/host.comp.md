```compmd component=meter signature
params:
  value: number
  label: string
```

```jsx component=meter runtime=react@19
export default function Meter({ value, label }) {
  return (
    <label style={{ display: "grid", gap: 6 }}>
      <span>{label}: <strong>{value}%</strong></span>
      <progress max="100" value={value} style={{ width: "100%" }} />
    </label>
  );
}
```

:: meter[Docs: 72%] {label="Docs" value=72}
