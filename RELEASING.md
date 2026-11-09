# Releasing

How to release, step by step. What is released and why it is done this way is decided in [docs/releases.md](docs/releases.md).

Four things are released, each on its own:

| What | Where | How |
|---|---|---|
| The implementation: `@compmd/render`, `@compmd/processor`, `@compmd/sandbox`, `@compmd/registry`, `@compmd/devtools`, `@compmd/vite`, `@compmd/pack`, `@compmd/create-pack` | npm | a tag `v<version>`; `.github/workflows/release.yml` publishes |
| The implementation: the crates `compmd-markdown`, `compmd` | crates.io | the same tag `v<version>`; the workflow publishes once the author approves its `crates` job |
| The Mermaid pack, `@comp-markdown/mermaid` | npm | a tag `mermaid-v<version>`; `.github/workflows/release-mermaid.yml` publishes |
| The spec | the website, `/spec/<version>/` | a tag `spec-v<version>`; the Pages workflow deploys |

The website follows `main` and needs no release: every push that touches what it is built from deploys it.

## The implementation

One version for the npm packages and the crate `compmd`, semantic, `0.x` until the spec's version 1. `compmd-markdown`, the fork of markdown-rs, has a version of its own, raised only when its code changes.

### 0. Prerequisites

A release publishes only packages that npm and this repository's workflow already know. Before a package's first release, or after its trusted publisher changed, go through [prerequisites, once per package](#prerequisites-once-per-package): the package created on npm by a first version published by hand, trusted publishing set up for `release.yml`, the workflow publishing it, and each checked. Every package the workflow publishes has them, since `0.2.0`.

### 1. Prepare

- The [prerequisites](#prerequisites-once-per-package) hold for every package the release publishes.
- `cargo make ci` is green on `main`.
- Each package's README says what changed, until the implementation has a changelog of its own.
- The version, in one commit: `render/package.json`, `wasm/package.json`, `sandbox/package.json`, `registry/package.json`, `devtools/package.json`, `vite/package.json`, `pack/package.json`, `create-pack/package.json` and `processor/Cargo.toml`.
- If the fork changed since its last release: its version in `markdown-rs/Cargo.toml` (on its `compmd` branch), and the same version in the `compmd-markdown` dependency of `processor/Cargo.toml` and `markdown-rs/mdast_util_to_markdown/Cargo.toml`.
- On Linux, once the version is committed: `cargo make linux` and `cargo make linux site` pass, so neither the release's checks nor its site deploy fail on the runner alone ([CI](docs/ci.md), decision 12).
- Push: the fork first, then this repository.

### 2. Tag, the workflow stages on npm, and the author approves

```sh
git tag v0.1.1
git push origin v0.1.1
```

The release workflow:
1. checks the tag is the version in every package's `package.json` and in `processor/Cargo.toml`;
2. installs Playwright's WebKit, for the sandbox's, the registry's, the Vite plugin's and create-pack's browser tests, and runs `cargo make ci`;
3. packs the packages with pnpm, which writes the versions in place of their `workspace:*` dependencies on each other;
4. stages them with npm's trusted publishing (`npm stage publish`): the renderer, then the processor, then the sandbox, which depends on both, then the registry, which needs the processor, then the Developer Tools, which depend on those four, then the Vite plugin, which depends on the first three, then `@compmd/pack`, which depends on all of those, then create-pack, whose packs pin it. No token is stored; each version carries its provenance. A version npm already has is left as it is, so a package whose first version was published by hand at the release's version is skipped, and a release that failed halfway can run again; a version it staged and nobody approved or rejected yet can't be staged again, so settle those first.

A staged version isn't installable until the author approves it, with two-factor authentication: on npmjs.com, or `(cd ~ && npm stage list @compmd/render)` then `(cd ~ && npm stage approve <stage-id>)` for each. Approve in the order they were staged, so a package is never on npm before those it depends on. One that is wrong is rejected (`npm stage reject <stage-id>`) and its fix released as the next version.

Check, once each is approved:

```sh
curl -s https://registry.npmjs.org/-/package/@compmd%2frender/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fprocessor/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fsandbox/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fregistry/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fdevtools/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fvite/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fpack/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fcreate-pack/dist-tags
```

Each gives the new version as `latest`.

### 3. The crates: the author approves, the workflow publishes

Once the npm packages are staged, the workflow's second job, `crates`, waits in the GitHub environment `crates-io` for the author's approval: on the run's page on GitHub, **Review deployments**, then approve. crates.io has no staging, and a published crate version can't be deleted, only yanked, so this approval is the last look. The job then publishes with crates.io's trusted publishing (no token is stored; `rust-lang/crates-io-auth-action` gets one for the job and revokes it after):

1. `compmd-markdown`, only when its version (`markdown-rs/Cargo.toml`) isn't on crates.io yet, since `compmd` depends on it;
2. `compmd`, at the release's version;
3. `tauri-plugin-compmd`, at the release's version, with the frames it embeds: the release job builds them (`cargo make tauri-frames`) from the packages it just checked and hands them to this job as an artifact, and the script stops before publishing anything without them. It is published without `cargo publish`'s build, which on Linux would need WebKitGTK: the Tauri check built and tested the same commit before the release went on. Git ignores the frames, and cargo counts an ignored file a package holds as uncommitted, so the plugin is published with `--allow-dirty`.

A version crates.io already has is skipped, so a release that failed halfway can run again. The job runs `.github/scripts/publish-crates.sh`, which reads each crate's version with `cargo metadata --no-deps`, since the fork's checkout has no `Cargo.lock`, and stops before publishing anything when it can't read one. Try it before a release, from the repository: `.github/scripts/publish-crates.sh --dry-run` packages what it would publish (`cargo package --list`, with the same flags, so it refuses what publishing would), and publishes nothing; the release job runs it too, before it stages the npm packages. In a fresh clone (`git clone --recurse-submodules`), the fork has no lock file, as on the runner. Check: `cargo search compmd` gives the new version. docs.rs builds the documentation a few minutes later.

`0.4.0`'s job read the fork's version with `cargo pkgid`, which needs a lock file, got none, and stopped at `compmd-markdown` without publishing; `compmd` 0.4.0 was published by hand from the tag, as 0.3.0 was. `0.6.0`'s job published `compmd`, then cargo refused `tauri-plugin-compmd`: its frames, ignored by git, counted as uncommitted, and the dry run of then didn't package anything to show it. The plugin's 0.6.0 was published by hand from the tag.

**Once, before the first release that publishes a crate this way** (each step the author's):
- On crates.io, each crate's **Settings** → **Trusted Publishing** → **GitHub**: the owner `comp-markdown`, the repository `compmd`, the workflow `release.yml`, the environment `crates-io`. For `compmd`, `compmd-markdown` and `tauri-plugin-compmd`: `compmd-markdown`'s code is the fork's, but it is published from this repository's workflow.
- On GitHub, the repository's **Settings** → **Environments** → `crates-io`: the author as its required reviewer, and only `v*` tags allowed to deploy to it.

**By hand**, when the workflow can't (a crate's first version, which trusted publishing needs to exist): dry-run first, from the tag's commit, with `cargo login` once (a token with the `publish-new` and `publish-update` scopes, kept by the author):

```sh
git worktree add ../compmd-v0.4.0 v0.4.0 && git -C ../compmd-v0.4.0 submodule update --init
cargo publish --dry-run -p compmd --manifest-path ../compmd-v0.4.0/Cargo.toml
cargo publish -p compmd --manifest-path ../compmd-v0.4.0/Cargo.toml
rm -rf ../compmd-v0.4.0 && git worktree prune
```

Run them from this repository's root, where its tags are. Git won't remove a worktree with a submodule, even deinitialized (`git worktree remove` says *working trees containing submodules cannot be moved or removed*), so the last line deletes its folder and has git forget it.

`compmd` 0.3.0 was published so, on 2026-11-03, after its npm release, and 0.4.0 the same day: from the tag, so the crate is the same code as the packages.

**A placeholder first**, for a crate whose first version needs the release's packages, as `tauri-plugin-compmd` needs the sandbox's frames of the same version: a `0.0.0` with no code, published by hand from outside the repository (a `Cargo.toml` that says so, an empty `lib.rs`, a README pointing to the real versions, and the MIT `LICENSE`; `cargo publish --allow-dirty` outside git). That creates the crate, so its trusted publisher can be set up, and the release workflow publishes its first real version. `tauri-plugin-compmd` 0.0.0 was published so on 2026-11-09, before 0.6.0. Yank the placeholder once the real version is out: `cargo yank --version 0.0.0 tauri-plugin-compmd`.

## The Mermaid pack

`@comp-markdown/mermaid` (`packs/mermaid/`) has a version of its own, for its component's params and forms ([its design](docs/mermaid-pack.md), decision 2).

1. **Once, before its first release** (done for `0.1.0`, 2026-11-02): the `@comp-markdown` organization exists on npm, and the publishing account may publish to it. The first version is published by hand, from outside the repository, as for any new package (below, [prerequisites](#prerequisites-once-per-package)): `pnpm --dir packs/mermaid build`, `pnpm --dir packs/mermaid pack --pack-destination ../packages`, the tarball checked (`dist/mermaid.js`, `dist/LICENSES.md`, `index.comp.md`, the README and its MIT `LICENSE`), then `npm publish` it. Then its trusted publisher is set up for the workflow `release-mermaid.yml`.
2. **Each release:** the version in `packs/mermaid/package.json` and in `index.comp.md`'s `export.version`, the same in both, in one commit, with what changed in its README; `cargo make ci` green.
3. Tag and push the tag:

   ```sh
   git tag mermaid-v0.2.0
   git push origin mermaid-v0.2.0
   ```

   The workflow checks the tag is the pack's version, runs `cargo make ci`, packs the pack and stages it with npm's trusted publishing.
4. Approve it, as for the `@compmd` packages: `(cd ~ && npm stage list @comp-markdown/mermaid)`, then `npm stage approve <stage-id>`.
5. Check: `curl -s https://registry.npmjs.org/-/package/@comp-markdown%2fmermaid/dist-tags` gives the new version as `latest`.

## The spec

1. The spec's changelog says what the version holds, and `cargo make ci` is green.
2. Tag the commit and push the tag:

   ```sh
   git tag spec-v1.0
   git push origin spec-v1.0
   ```

3. The Pages workflow deploys the site with the version at `/spec/1.0/`, frozen. `/spec/` shows the latest release, and `/spec/draft/` the draft.

## Prerequisites, once per package

Trusted publishing works only for a package that already exists on npm, so a new package is created by publishing its first version by hand, as `@compmd/render` and `@compmd/processor` were at `0.1.0`, `@compmd/sandbox`, `@compmd/registry`, `@compmd/vite`, `@compmd/pack` and `@compmd/create-pack` at `0.2.0`. A package can instead be created with a placeholder, as `@compmd/devtools` was before `0.3.0` (below). Then npm and the workflow are set up to publish it from then on. Each step is the author's, with their npm account; none is in the workflow.

**npm, once for the scope:**
- The `@compmd` organization exists on npm, and the publishing account is a member that may publish to it.
- The account has two-factor authentication on, for authorization and writes (below, "What publishing 0.1.0 taught").

**The package's first version, by hand:**

1. Build and pack it, from the repository, with the version the other packages have:

   ```sh
   pnpm --dir sandbox build
   pnpm --dir sandbox pack --pack-destination ../packages
   ```

2. Check the tarball before publishing: `tar tzf` for its files (`dist/`, for the Developer Tools its workers `dist/worker.js` and `dist/views-worker.js` beside `dist/index.js`, for the sandbox `protocol.schema.json`, for the Vite plugin `client.d.ts` and `compmd.config.schema.json`, for `@compmd/pack` `tsconfig.pack.json`, for create-pack `template/`, whose `gitignore` has no dot), and `tar xzf <file> -O package/LICENSE` for its licence, which must be MIT; its `package.json` names the other packages' versions, not `workspace:*`.
3. Try it without publishing, from outside the repository: `(cd ~ && npm publish <path>/compmd-sandbox-<version>.tgz --access public --dry-run)`.
4. Publish from outside the repository, after `npm login`. Dependencies go first: the renderer, then the processor, then the sandbox, then the registry, then the Developer Tools, then the Vite plugin, then `@compmd/pack`, then create-pack.

   ```sh
   (cd ~ && npm publish <path>/compmd-sandbox-<version>.tgz --access public)
   ```

5. Wait for npm's staging to end (below): the version is `latest` in `curl -s https://registry.npmjs.org/-/package/@compmd%2fsandbox/dist-tags`.

**Trusted publishing, on npmjs.com:**
- the package's **Settings** → **Trusted publisher** → **GitHub Actions**;
- the organization `comp-markdown`, the repository `compmd`, the workflow `release.yml` (for the Mermaid pack, `release-mermaid.yml`), no environment;
- allowed actions: `npm stage publish` alone, which npm always allows; not `npm publish` or `npm dist-tag`, so nothing the workflow does is installable before the author approves it;
- a new trusted publisher must publish within 2 days or it expires. One made before 2026-05-20 allows `npm publish` alone; its allowed actions can be edited in its settings to staging alone, as `@compmd/render`'s and `@compmd/processor`'s were for `0.2.2`. Its connection (the organization, the repository, the workflow, the environment) can't be edited: a wrong one is deleted and made again, as the Mermaid pack's was, made at first with `release.yml` instead of `release-mermaid.yml`. A workflow npm can't match to a trusted publisher fails with `ENEEDAUTH … This command requires you to be logged in`.

**The workflow, in this repository:**
- `release.yml` checks the package's version against the tag, packs it, and stages it after the packages it depends on;
- it installs what the package's tests need (the sandbox's, the registry's, the Vite plugin's and create-pack's: Playwright's WebKit), as `ci.yml` does;
- `RELEASING.md` lists it, in the table above and in the checks after a release.

**Validated:** npm checks the trusted publisher only at the workflow's first publish of the package. Once that release's workflow is green and the version approved, the version is `latest` in the registry (`curl`, as above), and the package's page on npmjs.com shows its provenance, from `comp-markdown/compmd` and `release.yml`. If its publish fails with the packages before it already published, fix the trusted publisher and run the workflow again: it skips the versions npm already has. A trusted publisher that doesn't allow what the workflow does fails with `403 … OIDC permission denied for this action`: so `0.2.1` failed at the sandbox, whose publisher allowed staging while the workflow still ran `npm publish`.

**A first version at a release's version.** A new package depends on the others at the release's version, so its first version, by hand, is that version: publish the new packages by hand first, set up their trusted publishers, then push the tag. The workflow publishes the rest and skips the new ones. `0.2.0` went this way: `@compmd/sandbox`, `@compmd/registry`, `@compmd/vite`, `@compmd/pack` and `@compmd/create-pack` by hand, then `v0.2.0` for `@compmd/render` and `@compmd/processor`. Until the tag's workflow has published them, the new packages don't install, so the two go together.

**A placeholder first.** A new package that needs what the release brings can't be published by hand ahead of it: `@compmd/devtools` needs the sandbox's observer, new in `0.3.0`. So its first version was a placeholder, `0.0.0`, with no code: a `package.json` that says so, a README pointing to the real versions, and the MIT `LICENSE`, published by hand from outside the repository. That created the package; its trusted publisher was set up; and `v0.3.0` staged its first real version with the rest, with provenance. Deprecate the placeholder once the real version is out: `(cd ~ && npm deprecate @compmd/devtools@0.0.0 "A placeholder with no code: install 0.3.0 or later")`.

## What publishing 0.1.0 taught

- **npm needs two-factor authentication to publish.** Without it, `npm publish` fails with `403 … Two-factor authentication or granular access token with bypass 2fa enabled is required to publish packages`, and nothing is uploaded. Turn on 2FA for authorization and writes (a security key, a passkey or an authenticator app), rather than a token that bypasses it.
- **npm stages a new package's first version.** The registry first shows a placeholder, `0.0.0-stage`, as `latest`, with "an operational version … is awaiting a staged release". The real version follows by itself: for `0.1.0`, within about an hour, with nothing to approve. `npm install` fails until then, so publish packages that depend on each other together.
- **Each package has its own `LICENSE`.** Without one, pnpm packs the repository's root `LICENSE`, which is the CC BY 4.0 licence of the spec's text, not the code's MIT.
- **npm doesn't run inside this repository.** Its `devEngines` names pnpm, so npm stops with `EBADDEVENGINES`. Run npm from another folder (`cd ~`), and ask the registry with `curl` (as above) rather than `npm view`.
- **The crates go in order.** `compmd` depends on `compmd-markdown` by path here and by version on crates.io, and crates.io rejects it until that version is published. The workflow publishes them in that order.
