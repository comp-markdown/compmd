---
rating: 2
---

An example, its code then what it renders, in a box of its own; its frontmatter is its own, so `rating` there is 4:

```compmd {render=both padding=s .demo}
---
rating: 4
---
Rated :value[4 of 5]{of=(string(md.doc.rating) + " of 5")}.
```

What it renders alone, an example without frontmatter:

```compmd {render=output}
**Bold**, {{md.version}}.
```

Source, as every `compmd` block is by default:

```compmd
Rated {{md.doc.rating}}.
```

- In a list item, indented:

  ```compmd {render=output}
  An item's example.
  ```
