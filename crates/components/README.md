# compmd-components

The component reader of [CompMD](https://compmd.dev/): what a host-language component takes, read from its source with [oxc](https://oxc.rs/), for component analysis (spec §7.1, W104).

- **React** (JSX, TSX): the default export's first parameter, the names it destructures and a `...rest`, or the props a type literal, an interface or a type alias of the file lists.
- **Svelte 5**: what the instance `<script>` gives `$props()`, the same way.
- **Types**, where TypeScript states one a signature's type can be checked against: `string`, `number`, `boolean`, string literals and their unions, and a slot's `ReactNode`, `ReactElement`, `JSX.Element` or `Snippet`.

It reads what the source states, and guesses nothing: a component whose props it can't read is left unchecked. `ComponentReader` implements the processor's `compmd::analysis::ComponentProps`:

```rust
use compmd::{Ambient, ParseOptions, Processor};
use compmd_components::ComponentReader;

let resolved = Processor::new("a.comp.md", text)
    .parse(&ParseOptions::default())
    .analysing(ComponentReader)
    .resolve(files, packs, &Ambient::default());
```

`@compmd/processor/components` is the same reader as WebAssembly.

## Status

In development, released with CompMD's packages at the same version.

**Versions:**
- **0.12.0:** the first version.
