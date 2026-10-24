```jsx component=total runtime=react@19
export default function Total({ rows }) {
  return <b>{rows.reduce((sum, row) => sum + row.amount, 0)}</b>;
}
```

```compmd component=total signature
params:
  rows: array
```

Total: :total[1,234]{rows=([{"amount": 1000}, {"amount": 234}])}

:: total[Total unknown] {rows=([null])}
