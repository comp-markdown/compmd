# @compmd/lsp

The [CompMD](https://compmd.dev/) language server, `compmd-lsp`, for editors that find it on npm: diagnostics, completion, hover, signature help, go to definition and semantic tokens for CompMD documents, with their project's configuration and packs.

```sh
npm i -g @compmd/lsp
compmd-lsp --stdio
```

npm installs the binary for your platform beside it, from one of its optional dependencies: `@compmd/lsp-<os>-<cpu>` for macOS, Linux and Windows on arm64 and x64. Elsewhere, `cargo install compmd-lsp` builds it. The binaries are on each GitHub release too, with their SHA-256 sums.

## Status

In development, released with CompMD's packages at the same version.

**Versions:**
- **0.12.0:** the first version.

## License

MIT
