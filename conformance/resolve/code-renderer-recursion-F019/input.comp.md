````compmd component=shout
renders: shout
---
**{{code}}**

```shout {alt="Again"}
louder
```
````

`shout` renders `shout` blocks, and its own body holds one: rendering it leads back to `shout`, so that block is F019 and shows its fallback, the code block. The block below renders once.

```shout {alt="Hello"}
hello
```
