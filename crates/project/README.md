# compmd-project

Reading a [CompMD](https://compmd.dev/) project as its hosts do, one implementation for the language server, a command line, the Tauri plugin and `@compmd/vite` (as WebAssembly):

- `root_of`: a document's root folder, the nearest above it with a configuration, else the editor's workspace folder, else its own ([the language server](https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md), decision 3).
- `config_of` and `read_config`: the configuration, `compmd.config.json` or `package.json`'s `compmd` object, checked as `@compmd/vite` checks it, and `Config::translated`, what the processor takes from it: its policy, the ambient packs and the origins.
- `PackFolders`: the installed packs and the folder packs as the processor's `compmd::host::Packs`, with their integrity from the lock file (`integrities`).

The crate does no I/O of its own: it reads through `ProjectFiles`, which `DiskFiles` implements with `std::fs`.

## Status

In development, released with CompMD's packages at the same version.

**Versions:**
- **0.12.0:** the first version.
