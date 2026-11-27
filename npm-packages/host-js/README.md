# @compmd/host

The serving contract of a [CompMD](https://compmd.dev/) host over the Fetch API: `respond(request)` answers with a bundle's frames and the packs the bundle or a store holds, with the headers that sandbox them, and refuses anything else, saying why. For workers, Node, and dev servers: `@compmd/vite`'s dev server serves its bundle with it. What a request asks for, and the headers, are decided by `compmd-host`, the same contract in Rust, compiled to WebAssembly (77 KB): this package only reads the file and makes the `Response`, so the two can't drift, and both run [the same cases](https://github.com/comp-markdown/compmd/tree/main/crates/host/fixtures/respond) ([hosting](https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md), decisions 1 and 8).

```ts
import { folderBundle, respond } from "@compmd/host/node"; // in Node: loads the WebAssembly from its file

const bundle = await folderBundle("dist/compmd"); // what @compmd/vite or buildBundle wrote
// on the bundle's origin, with any prefix it is mounted under stripped first
const answer = await respond(request, bundle);   // a Response
```

**What it answers**, and nothing else:

- only `GET`; any other method is `405`, with why in plain text;
- `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<release>/<path>` (`/packs/@kit/mermaid@1.2.0/dist/mermaid.js`) of a pack the bundle's manifest lists or `store` holds, each part percent-decoded first (`my%20chart.js` is `my chart.js`), where a path has no empty, `.` or `..` part, decoded or not; anything else, the manifest included, is `404`, with why; every refusal readable from any origin (`Access-Control-Allow-Origin: *`);
- each file with `Content-Security-Policy: sandbox allow-scripts`, `Access-Control-Allow-Origin: *` (frames are opaque origins), `Access-Control-Expose-Headers: *`, `X-Content-Type-Options: nosniff`, its type by extension, and `Cache-Control: max-age=31536000, immutable` (a frame's path names its build, a pack's its version).

**The WebAssembly** loads once, by itself, on the first `respond` (a load that fails is tried again by the next request): from its file next to the package's code, which a browser or a worker that serves the package's files fetches. In Node, import from `@compmd/host/node`, which reads it from its file as it is imported (Node's `fetch` reads no `file:` URL). A worker that bundles its modules imports `@compmd/host/compmd_host_wasm_bg.wasm` and gives it with `initSync({ module })` before the first request.

**A bundle elsewhere** (a worker's assets, a key-value store) is any `Bundle`: its frames' `hash`, `frameFile(path)` and `packFile(place)`, the files of packs it holds. **A store** of packs fetched later, such as from a registry, is a `Store`, `file(place)`, asked when the bundle hasn't the pack. A `Place` is the processor's: `{ pack, version, path }`. `folderBundle` reads the manifest as `compmd-host` does, and refuses one of another format, or with a frames' hash or a name that isn't one, saying why.

**A project** is read as `compmd-project` reads it, with `@compmd/host/project`, for a build such as `@compmd/vite`'s: `configOf(folder, files)` and `readConfig(value, source)`, the configuration checked; `installedPacks(root, files)` and `folderPacks(keys, folder, files)`, the packs found; `integrities` and `lockFileOf`, the lock file's; `rootOf`, a document's root folder. `files` is the host's file system, `{ text(path), isDir(path) }`. In Node the module loads the first time one is used; in a page or a worker, `init()` first.

It shares its version with the other `@compmd` packages ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.12.0:** `@compmd/host/project`, reading a project with `compmd-project`, in the same module; `@compmd/host/registry`, the registry rules' `Resolution` as `compmd-host` decides it, with `loaded`, `answeredVersions` and `offOrigin`, for `@compmd/registry`.
- **0.11.0:** request paths are percent-decoded, part by part, before they are read; a `Place` is `{ pack, version, path }`, as the processor has it (breaking: `{ release, path }` before); `folderBundle` checks the manifest as `compmd-host` does; a WebAssembly load that fails is tried again.
- **0.10.0:** the same as 0.9.0.
- **0.9.0:** the same as 0.8.0.
- **0.8.0:** the first version: `respond`, `compmd-host`'s decision as WebAssembly, with `init` and `initSync`, and `@compmd/host/node`'s `folderBundle`. (`0.0.0` is a placeholder with no code.)
