# The registry rules in `compmd-host`

**Status:** accepted (CR-146), built in CMD-19. [Hosting](hosting.md), H1 option (c): the rules `@compmd/registry` has in TypeScript move into `compmd-host`, once, for every host: the language server and a command line natively, `@compmd/registry` as WebAssembly (CMD-422), the Tauri plugin (CMD-423).

## Decisions

1. **The rules do no I/O: they are a state machine the host drives.** A `Resolution` says what it needs next (a step), the host does it its own way and gives the answer back, until the resolution is done. A native host answers synchronously, on its own thread (the language server has no async runtime, [its decision 2](language-server.md)); `@compmd/registry` answers with `fetch` and Cache Storage, awaiting each step. The same code makes every decision; only the I/O differs. A trait for an HTTP client would have to be either synchronous, which the browser's `fetch` isn't, or async, which a native host without a runtime can't drive and which the WebAssembly build would need `wasm-bindgen-futures` across for every call.

2. **The steps are the I/O the TypeScript resolver does today, nothing more, at the registry's level** (changed 2026-11-25, CMD-422: a host's own registry, which `@compmd/registry`'s API takes, answers a pack's versions and a tarball, not npm's HTTP):

   | Step | The host | Answers with |
   |---|---|---|
   | `Missing` | asks its processor what the request still imports (`sources`) | the needed packs (`NeededPack`: pack, range, pinned integrity, library) |
   | `KeptVersions { registry, pack }` | reads its cache's version list for the pack | the list and whether it is fresh, or none |
   | `Versions { registry, pack }` | asks the registry for the pack's versions | the versions (none when it doesn't have the pack) and how long they may be kept, or the failure |
   | `KeepVersions { registry, pack, versions, max_age }` | writes the list to its cache | nothing |
   | `Kept { release }` | reads its cache's files of a release | the files (text and binary), or none |
   | `Tarball { registry, version, max_bytes }` | gets the version's tarball from the registry, stopping past `max_bytes` | the bytes, or the failure (too large among them) |
   | `Keep { release, files }` | writes the checked files to its cache | nothing |

   **npm's protocol stays the crate's**, as functions a registry that speaks it calls, natively or as WebAssembly: `metadata_url` (with `Accept: ` `ABBREVIATED`), `answered_versions` (the status and body of its answer: none on 404, the registry's failure otherwise), `versions_of` and `off_origin` (a tarball only from the registry's origin, so its headers go nowhere else). The resolution checks every tarball's bytes, whoever gives them. A host gives up on a pack after a `Duration` (`give_up`), its message in seconds as the host counts them, `0.05 s`.

   Steps that don't depend on each other come together (`next` gives a list), so a host fetches them at once, as `Promise.all` does today.

3. **Every rule is the crate's:** the resolver's loop (ask what is missing, resolve each new import once, ask again); choosing a version (`newest_matching`, and a library pack's version every range asked for it matches, [sharing library packs](sharing-library-packs.md), decision 5); the policy before any download (`deny`, the size cap against the metadata's `unpackedSize`, a SHA-512 integrity to check against); reading npm's abbreviated metadata; checking the tarball against `dist.integrity` and an import's pinned `integrity`; reading the tar (gunzip with `miniz_oxide`, the tar by hand, as `tar.ts` is, with its limits); the files a frame may load (images and fonts, `file-types.ts`); and what a registry that doesn't answer gives (a stale list when there is one). The messages are the TypeScript's.

4. **What stays the host's:** the HTTP client and its headers (a private registry's token is the host's, never the crate's), the cache and its storage, the clock (a list's freshness is the host's to say), timeouts (a host that gives up on a pack tells the resolution, `give_up(pack)`, which gives the TypeScript's message), and in a browser, the origin a request may go to (`fetch: "host"`).

5. **The processor stays out of `compmd-host`'s rules.** The `Missing` step is how the loop learns what is missing, so the rules need no processor: `@compmd/registry` asks `@compmd/processor`, a Rust host asks `compmd`. The one function both need, `newest_matching` (ranges as the processor reads them), moves from `compmd::host` to `compmd-pack`, which both crates have, and `compmd::host` re-exports it.

6. **What a resolution gives** is what `resolvePacks` gives: the releases to add to the request's packs (each with its integrity, its text files, or none for a release the processor refuses before reading, or `broken` for a tarball that doesn't match), the packs the size cap refuses (for `policy.refused`), the built modules and the files a frame may load, by place, and the problems, each with its pack.

7. **One set of fixtures.** The cases (a registry's metadata and tarballs, the policy, the imports, what the resolution gives) are files in `crates/host/fixtures/registry/`, which the crate's tests run natively, and which `@compmd/registry`'s tests run through the WebAssembly build in CMD-422, as `fixtures/respond` is shared today.

## The API

```rust
let mut resolution = Resolution::new(Options { registries, max_pack_size, .. });
loop {
    match resolution.next_steps() {
        Next::Steps(steps) => for step in steps {
            let answer = host.perform(&step);        // the host's I/O, synchronous or awaited
            resolution.give(step.id(), answer);
        },
        Next::Done(resolved) => break resolved,
    }
}
```

`Step` and `Answer` are plain data (serde), so the WebAssembly build passes them as JSON, as `@compmd/processor` does its requests.

## Modules

`crates/host/src/registry.rs` and `registry/`: `resolution.rs` (the state machine), `step.rs`, `answer.rs`, `metadata.rs` (npm's abbreviated metadata), `policy.rs` (the checks before a download), `integrity.rs` (SHA-512 with `sha2` and `base64`), `tar.rs` (gunzip and the tar), `file_types.rs`, `resolved.rs`, each with its tests; behind a feature, `registry`, so an adapter that only serves builds none of it.

## Not in this change

The WebAssembly build and `@compmd/registry` on it (CMD-422), the Tauri plugin's `registry/` proxy (CMD-423), and a native host's driver, which the language server writes for its threads (CMD-420).
