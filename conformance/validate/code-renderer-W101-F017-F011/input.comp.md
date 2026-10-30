```compmd component=shout
renders: shout
params:
  loud: boolean = false
---
**{{code}}**
```

```shout
no alt
```

```shout {code=x alt="Code given"}
text
```

```shout {loud alt="Broken" tone="x}
text
```

```sh {tone="x}
A block no code renderer renders keeps its info string as written.
```
