# Releases

**Status: accepted.**

What this repository releases, how each is versioned, and what a release does. The steps, with what publishing 0.1.0 taught, are in [RELEASING.md](../RELEASING.md). Two things are released, apart: **the spec**, and **the implementation** (the npm packages and the crate). Drafts follow `main`; releases are frozen.

## Decisions

1. **The spec's releases are tags `spec-v<major>.<minor>`**, such as `spec-v1.0`. A tag freezes the spec as it is at that commit: the website gives it an address of its own, `/spec/1.0/`, which never changes. Until version 1 is released, the spec is a draft, published from `main` with a "Draft" banner, and `/spec/` is the draft; after it, `/spec/` is the latest release and `/spec/draft/` the draft ([the website's design](../website/docs/design.md), decision 11).
2. **The implementation's releases are tags `v<major>.<minor>.<patch>`**, such as `v0.2.0`, and its version is [semantic](https://semver.org). It is one version for every package: `@compmd/processor` and `@compmd/render` are released together, always with the same number, since the processor depends on the renderer's types; the crate `compmd` takes the same number once it is published.
3. **`0.x` until the spec's version 1 is released.** While the spec is a draft, the implementation is `0.x`, and any minor release may break, as semantic versioning allows below `1.0.0`. With the spec's version 1, the implementation becomes `1.0.0`, and a breaking change needs a new major version from then on. Each package's README says which version of the spec it implements.
4. **npm gets releases only.** A published version can never be changed, and applications pin to it, so nothing is published from `main`: no `next` versions, since the playground already shows the latest `main`.
5. **A release tag publishes from CI**, by a workflow of its own (`.github/workflows/release.yml`): it builds, runs `cargo make ci`, and stages every package with npm's trusted publishing. GitHub proves to npm which repository and workflow publishes, so no npm token is stored anywhere, and each version carries its provenance: where and from what it was built. **A staged version is installable once the author approves it**, with two-factor authentication: the trusted publishers allow staging alone, so CI builds and checks, and the author releases (decided 2026-11-02). **The crates are published by the same workflow** (decided 2026-11-03), once the npm packages are staged, with crates.io's trusted publishing: crates.io has no staging, so their job waits in a GitHub environment, `crates-io`, for the author's approval, and the crates' trusted publishers name that environment. `compmd` takes the release's version; `compmd-markdown`, the fork, its own, published only when it changed.
6. **The first version, `0.1.0`, is published by hand**, by the author: trusted publishing is set up on packages that already exist on npm.
7. **The website shows the latest release**: its install line and snippets are the latest published version's, and they run as tests against the code they come from, so a release can't leave them broken.
8. **Packs this repository publishes are released on their own**: each has a version of its own and a tag `<pack>-v<version>`, which a workflow of its own publishes the same way, by trusted publishing. The first is the Mermaid pack, `@comp-markdown/mermaid` ([its design](mermaid-pack.md), decision 9).

## A release, step by step

1. The changelog's entries for the release are written, and `cargo make ci` is green on `main`.
2. The version is set in every package's `package.json` and in `processor/Cargo.toml`, in one commit.
3. The tag `v<version>` is pushed; the release workflow publishes, and the website deploys with the new install line.

A release of the spec is a tag `spec-v<version>` on a commit whose changelog says what the version holds; the website deploys with its new address.

## The release workflow

`.github/workflows/release.yml` runs on a tag `v*`. It checks the tag is the version in both `package.json` files, runs `cargo make ci`, packs both packages with pnpm (which writes the renderer's version into the processor's dependency), and publishes them with npm 11.6.2 by trusted publishing, the renderer first.

It needs both packages to exist on npm, and each set up once on npmjs.com (the package's settings, **Trusted publisher**, GitHub Actions): the organization `comp-markdown`, the repository `compmd`, the workflow `release.yml`, no environment.

Its second job, `crates`, runs in the environment `crates-io`, once the author approves it: `compmd-markdown` from the fork when crates.io doesn't have its version, then `compmd` with the release's number, each with a token `rust-lang/crates-io-auth-action` gets by trusted publishing. Each crate's trusted publisher on crates.io names the organization `comp-markdown`, the repository `compmd`, the workflow `release.yml` and the environment `crates-io`.

## Not in this change

- A changelog for the implementation apart from the spec's: until the first release, the packages' READMEs say what changed.
