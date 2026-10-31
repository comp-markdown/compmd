# Handling pack loading states

**Status: placeholder (2026-10-31).** Not designed yet; this page says what the design is to settle.

Everything a document needs loads in stage 4a, before anything is evaluated (spec §11), so an import is either there or F026, and data is either loaded or its `fallback`. A processor that fetches while a reader reads, such as one that gets packs from a registry at run time ([packs from a registry](packs-from-a-registry.md)), has a third state: the import is on its way. Today it can only render as if it had failed, and render again when it arrives (§8.5).

## To settle

- **Three states, not two:** loading, loaded and can't be had, for a pack and its components, and whether data that arrives late is the same.
- **What a reader sees** while a use is loading, and how that differs from its fallback when the pack can't be had: a mark in the expanded tree (§12.3), the fallback with `aria-busy`, a placeholder the host styles.
- **What a host and a component learn:** events or a status for loading, loaded and failed; whether a component sees a pending use in its slot.
- **Diagnostics:** none while loading; F026 once it can't be had; whether a host may wait before reporting.
- **What changes** in the spec (§4.5, §8.5, §11, §12.3), the processor (`sources`, `OpenDocument`), `@compmd/render`, `@compmd/sandbox` and the editor.
