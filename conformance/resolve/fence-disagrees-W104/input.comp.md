```jsx component=badge runtime=react@19
export default function Badge({ label, tone, emphasis }) {
  return <b className={tone}>{label}</b>;
}
```

```compmd component=badge signature
params:
  label: string
  tone: info | warn = info
  size: number?
```

:: badge[New] {label="New"}
