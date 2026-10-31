# Releasing

How to release, step by step. What is released and why it is done this way is decided in [docs/releases.md](docs/releases.md).

Three things are released, each on its own:

| What | Where | How |
|---|---|---|
| The implementation: `@compmd/render`, `@compmd/processor`, `@compmd/sandbox` | npm | a tag `v<version>`; `.github/workflows/release.yml` publishes |
| The implementation: the crates `compmd-markdown`, `compmd` | crates.io | by hand, after the tag |
| The spec | the website, `/spec/<version>/` | a tag `spec-v<version>`; the Pages workflow deploys |

The website follows `main` and needs no release: every push that touches what it is built from deploys it.

## The implementation

One version for the npm packages and the crate `compmd`, semantic, `0.x` until the spec's version 1. `compmd-markdown`, the fork of markdown-rs, has a version of its own, raised only when its code changes.

### 1. Prepare

- `cargo make ci` is green on `main`.
- Each package's README says what changed, until the implementation has a changelog of its own.
- The version, in one commit: `render/package.json`, `wasm/package.json`, `sandbox/package.json` and `processor/Cargo.toml`.
- If the fork changed since its last release: its version in `markdown-rs/Cargo.toml` (on its `compmd` branch), and the same version in the `compmd-markdown` dependency of `processor/Cargo.toml` and `markdown-rs/mdast_util_to_markdown/Cargo.toml`.
- Push: the fork first, then this repository.

### 2. Tag, and the workflow publishes to npm

```sh
git tag v0.1.1
git push origin v0.1.1
```

The release workflow:
1. checks the tag is the version in every package's `package.json`;
2. installs Playwright's WebKit, for the sandbox's browser tests, and runs `cargo make ci`;
3. packs the packages with pnpm, which writes the versions in place of their `workspace:*` dependencies on each other;
4. publishes them with npm's trusted publishing: the renderer, then the processor, then the sandbox, which depends on both. No token is stored; each version carries its provenance.

Check, once it is green:

```sh
curl -s https://registry.npmjs.org/-/package/@compmd%2frender/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fprocessor/dist-tags
curl -s https://registry.npmjs.org/-/package/@compmd%2fsandbox/dist-tags
```

Each gives the new version as `latest`.

### 3. Publish the crates, by hand

A published crate version can't be deleted, only yanked, so dry-run first. Publishing needs `cargo login` once, with a crates.io token that has the `publish-new` and `publish-update` scopes; the token stays with the author.

1. Only if the fork changed: publish `compmd-markdown` first, since `compmd` depends on it:

   ```sh
   cargo publish --manifest-path markdown-rs/Cargo.toml -p compmd-markdown
   ```

2. Then `compmd`. Its dry run builds it against the published `compmd-markdown`, so it only works once that is on crates.io:

   ```sh
   cargo publish --dry-run -p compmd
   cargo publish -p compmd
   ```

docs.rs builds the documentation a few minutes later.

## The spec

1. The spec's changelog says what the version holds, and `cargo make ci` is green.
2. Tag the commit and push the tag:

   ```sh
   git tag spec-v1.0
   git push origin spec-v1.0
   ```

3. The Pages workflow deploys the site with the version at `/spec/1.0/`, frozen. `/spec/` shows the latest release, and `/spec/draft/` the draft.

## A package's first version, by hand

Trusted publishing works only for a package that already exists on npm. So a new package's first version is published by hand, as `0.1.0` was, and as `@compmd/sandbox`'s first version is to be:

1. Build and pack it, from the repository:

   ```sh
   pnpm --dir render build
   pnpm --dir render pack --pack-destination ../packages
   ```

2. Check the tarball before publishing: `tar tzf` for its files, and `tar xzf <file> -O package/LICENSE` for its licence, which must be MIT.
3. Publish from outside the repository, after `npm login`. Dependencies go first: the renderer before the processor.

   ```sh
   (cd ~ && npm publish <path>/compmd-render-0.1.0.tgz --access public)
   ```

4. Set up trusted publishing on npmjs.com:
   - the package's **Settings** → **Trusted publisher** → **GitHub Actions**;
   - the organization `comp-markdown`, the repository `compmd`, the workflow `release.yml`, no environment;
   - allowed actions: publish only.

   npm validates it at the workflow's first publish.

## What publishing 0.1.0 taught

- **npm needs two-factor authentication to publish.** Without it, `npm publish` fails with `403 … Two-factor authentication or granular access token with bypass 2fa enabled is required to publish packages`, and nothing is uploaded. Turn on 2FA for authorization and writes (a security key, a passkey or an authenticator app), rather than a token that bypasses it.
- **npm stages a new package's first version.** The registry first shows a placeholder, `0.0.0-stage`, as `latest`, with "an operational version … is awaiting a staged release". The real version follows by itself: for `0.1.0`, within about an hour, with nothing to approve. `npm install` fails until then, so publish packages that depend on each other together.
- **Each package has its own `LICENSE`.** Without one, pnpm packs the repository's root `LICENSE`, which is the CC BY 4.0 licence of the spec's text, not the code's MIT.
- **npm doesn't run inside this repository.** Its `devEngines` names pnpm, so npm stops with `EBADDEVENGINES`. Run npm from another folder (`cd ~`), and ask the registry with `curl` (as above) rather than `npm view`.
- **The crates go in order.** `compmd` depends on `compmd-markdown` by path here and by version on crates.io, and crates.io rejects it until that version is published.
