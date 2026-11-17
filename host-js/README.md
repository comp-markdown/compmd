# @compmd/host

The serving contract of a CompMD host over the Fetch API: `respond(request)` answers with a bundle's frames and the packs the bundle or a store holds, with the headers that sandbox them, and refuses anything else, saying why. For workers, Node, and dev servers: `@compmd/vite`'s dev server serves its bundle with it. `compmd-host` is the same contract in Rust, over `http` types, and both run [the same cases](https://github.com/comp-markdown/compmd/tree/main/host/fixtures/respond) ([hosting](https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md), decisions 1 and 8).

```ts
import { respond } from "@compmd/host";
import { folderBundle } from "@compmd/host/node";

const bundle = await folderBundle("dist/compmd"); // what @compmd/vite or buildBundle wrote
// on the bundle's origin, with any prefix it is mounted under stripped first
const answer = await respond(request, bundle);   // a Response
```

**What it answers**, and nothing else:

- only `GET`; any other method is `405`, with why in plain text;
- `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<release>/<path>` (`/packs/@kit/mermaid@1.2.0/dist/mermaid.js`) of a pack the bundle's manifest lists or `store` holds, where a path has no empty, `.` or `..` part; anything else, the manifest included, is `404`, with why; every refusal readable from any origin (`Access-Control-Allow-Origin: *`);
- each file with `Content-Security-Policy: sandbox allow-scripts`, `Access-Control-Allow-Origin: *` (frames are opaque origins), `Access-Control-Expose-Headers: *`, `X-Content-Type-Options: nosniff`, its type by extension, and `Cache-Control: max-age=31536000, immutable` (a frame's path names its build, a pack's its version).

**A bundle elsewhere** (a worker's assets, a key-value store) is any `Bundle`: its frames' `hash`, `frameFile(path)` and `packFile(place)`, the files of packs it holds. **A store** of packs fetched later, such as from a registry, is a `Store`, `file(place)`, asked when the bundle hasn't the pack.

It shares its version with the other `@compmd` packages ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.7.1:** first version.
