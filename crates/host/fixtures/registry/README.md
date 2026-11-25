# The registry rules' shared cases

The cases `compmd-host`'s registry rules hold to, natively (`registry/resolution/tests/fixtures.rs`) and as WebAssembly (`@compmd/registry`'s `rules.test.ts`) ([the registry rules](../../../../docs/registry-rules.md), decision 7). Each file is a case:

- `imports`: what the request imports, as the processor's `sources` lists it (`pack`, `range`, and `integrity` or `library`);
- `brings`: by pack, what a release of it imports in turn once it is added (library packs);
- `registry`: the npm registry's releases, each a `pack`, a `version` and its text `files`; a runner writes each one's tarball (`package/<path>`), its SHA-512 integrity and its unpacked size into the metadata, then `dist` changes what the metadata says (`null` removes a key), and `corrupt` serves another tarball than the metadata's integrity;
- `options`: the host's `maxPackSize` and `deny`, besides its one registry for every scope;
- `resolved`: what the resolution gives: each pack's releases with their files' paths, the `broken` ones by `pack@version`, the `refused` packs with why, the `problems`, and the tarballs `downloaded`, in order.
