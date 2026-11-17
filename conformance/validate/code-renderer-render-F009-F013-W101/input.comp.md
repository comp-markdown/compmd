```compmd component=shout
renders: shout
params:
  render: string?
  alt: string?
---
**{{code}}**
```

```compmd component=tag
params:
  class: string?
  id: string?
  alt: string?
---
*A tag; `alt` is free here, as `tag` renders no language.*
```

```shout {render=source}
Shows its source: no text alternative needed.
```

```shout {render=both}
Draws, so it needs one.
```

```shout {render=code alt="Not a value"}
text
```

```shout {render=(md.doc.mode) alt="Evaluated"}
text
```

```shout {render="both" alt="Quoted"}
text
```
