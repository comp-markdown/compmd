# compmd-pack

The names CompMD packs and their files are known by (spec §6.4), each checked once, when it is made:

- `PackName`: `@scope/name`, as npm names scoped packages; `is_reserved` for `@compmd/`.
- `ReleaseId`: a pack at one version, `@kit/mermaid@1.2.0`.
- `FilePath`: a file's path in a pack, which can't leave it.
- `Place`: a release and a file's path in it, `@kit/mermaid@1.2.0/dist/mermaid.js`.
- `Integrity`: `sha256-`, `sha384-` or `sha512-` and its digest, as written (the spec's examples shorten one, `sha512-9c1fQe…`).

The processor (`compmd`, which re-exports it as `compmd::pack`) and `compmd-host` both use it, so these mean one thing everywhere. In JSON each is what the spec writes: a name, a path and an integrity are strings, and a place is `{"pack", "version", "path"}`.

## Status

In development, released with CompMD's packages at the same version.

**Versions:**
- **0.7.0:** the first version.
