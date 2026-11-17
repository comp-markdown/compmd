```jsx component=map
export default function Map({ place }) {
  return <iframe title={place} src={`https://maps.example.com/?q=${place}`} />;
}
```

```compmd component=map signature
params:
  place: string
print: sideways
```

```compmd component=note
print: fallback
---
A note.
```
