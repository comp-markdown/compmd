```jsx component=badge runtime=preact@10
export default function Badge() { return <b>new</b>; }
```

```compmd component=badge signature
params: {}
```

```jsx component=tag
export default function Tag() { return <i>tag</i>; }
```

```compmd component=tag signature
params: {}
```

The host runs React 19 and Svelte 5, not Preact: `badge` renders its fallback, `tag` its component.

:: badge[New]

:: tag[Tag]
