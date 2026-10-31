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

### 0. Prerequisites

A release publishes only packages that npm and this repository's workflow already know. Before a package's first release, or after its trusted publisher changed, go through [prerequisites, once per package](#prerequisites-once-per-package): the package created on npm by a first version published by hand, trusted publishing set up for `release.yml`, the workflow publishing it, and each checked. `@compmd/sandbox` still needs them.

### 1. Prepare

- The [prerequisites](#prerequisites-once-per-package) hold for every package the release publishes.
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

## Prerequisites, once per package

Trusted publishing works only for a package that already exists on npm, so a new package is created by publishing its first version by hand, as `@compmd/render` and `@compmd/processor` were at `0.1.0`, and as `@compmd/sandbox` is to be at the version the others have. Then npm and the workflow are set up to publish it from then on. Each step is the author's, with their npm account; none is in the workflow.

**npm, once for the scope:**
- The `@compmd` organization exists on npm, and the publishing account is a member that may publish to it.
- The account has two-factor authentication on, for authorization and writes (below, "What publishing 0.1.0 taught").

**The package's first version, by hand:**

1. Build and pack it, from the repository, with the version the other packages have:

   ```sh
   pnpm --dir sandbox build
   pnpm --dir sandbox pack --pack-destination ../packages
   ```

2. Check the tarball before publishing: `tar tzf` for its files (`dist/`, and for the sandbox `protocol.schema.json`), and `tar xzf <file> -O package/LICENSE` for its licence, which must be MIT; its `package.json` names the other packages' versions, not `workspace:*`.
3. Try it without publishing, from outside the repository: `(cd ~ && npm publish <path>/compmd-sandbox-<version>.tgz --access public --dry-run)`.
4. Publish from outside the repository, after `npm login`. Dependencies go first: the renderer, then the processor, then the sandbox.

   ```sh
   (cd ~ && npm publish <path>/compmd-sandbox-<version>.tgz --access public)
   ```

5. Wait for npm's staging to end (below): the version is `latest` in `curl -s https://registry.npmjs.org/-/package/@compmd%2fsandbox/dist-tags`.

**Trusted publishing, on npmjs.com:**
- the package's **Settings** → **Trusted publisher** → **GitHub Actions**;
- the organization `comp-markdown`, the repository `compmd`, the workflow `release.yml`, no environment;
- allowed actions: publish only.

**The workflow, in this repository:**
- `release.yml` checks the package's version against the tag, packs it, and publishes it after the packages it depends on;
- it installs what the package's tests need (the sandbox's: Playwright's WebKit), as `ci.yml` does;
- `RELEASING.md` lists it, in the table above and in the checks after a release.

**Validated:** npm checks the trusted publisher only at the workflow's first publish of the package. Once that release's workflow is green, the version is `latest` in the registry (`curl`, as above), and the package's page on npmjs.com shows its provenance, from `comp-markdown/compmd` and `release.yml`. If its publish fails with the packages before it already published, rerunning the workflow would publish those again, which npm refuses: fix the trusted publisher, and publish that one tarball by hand, as above.

## What publishing 0.1.0 taught

- **npm needs two-factor authentication to publish.** Without it, `npm publish` fails with `403 … Two-factor authentication or granular access token with bypass 2fa enabled is required to publish packages`, and nothing is uploaded. Turn on 2FA for authorization and writes (a security key, a passkey or an authenticator app), rather than a token that bypasses it.
- **npm stages a new package's first version.** The registry first shows a placeholder, `0.0.0-stage`, as `latest`, with "an operational version … is awaiting a staged release". The real version follows by itself: for `0.1.0`, within about an hour, with nothing to approve. `npm install` fails until then, so publish packages that depend on each other together.
- **Each package has its own `LICENSE`.** Without one, pnpm packs the repository's root `LICENSE`, which is the CC BY 4.0 licence of the spec's text, not the code's MIT.
- **npm doesn't run inside this repository.** Its `devEngines` names pnpm, so npm stops with `EBADDEVENGINES`. Run npm from another folder (`cd ~`), and ask the registry with `curl` (as above) rather than `npm view`.
- **The crates go in order.** `compmd` depends on `compmd-markdown` by path here and by version on crates.io, and crates.io rejects it until that version is published.
