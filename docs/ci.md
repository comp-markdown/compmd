# CI on GitHub

**Status: accepted.**

Every check of this repository runs only on a contributor's machine: `cargo make ci` runs them all, and nothing stops a push or a pull request that breaks one. CI on GitHub runs the same command on every push to `main` and every pull request, so a break shows on the commit that made it, and a pull request shows whether it passes before it is merged.

## Decisions

1. **CI runs `cargo make ci`, the command a contributor runs.** One local command checks what CI checks: formatting, clippy with warnings as errors, the tests with the conformance set, the WebAssembly build, cargo-deny, line coverage of at least 99%, and every TypeScript package built, linted and tested. The workflow sets up the tools and calls it; it holds no check of its own, so the two can't drift.
2. **One job, on one runner.** `ubuntu-24.04`, named rather than `ubuntu-latest`, so a new image doesn't change the run unannounced. The checks run one after another, as they do locally; splitting them over parallel jobs would give an answer sooner, but each job would build the workspace again, and the workflow would have to repeat `Makefile.toml`'s list.
3. **When it runs:** on every push to `main`, on every pull request to `main`, and by hand (`workflow_dispatch`). A newer push to the same branch or pull request cancels the run still going for it (`concurrency`), since only the latest commit's answer matters.
4. **The tools are pinned, as locally.** The Rust toolchain from `rust-toolchain.toml`, with `llvm-tools-preview` for coverage; `wasm-bindgen-cli` at the version `wasm/Cargo.toml` pins (`0.2.129`); cargo-make, cargo-deny and cargo-llvm-cov at named versions, installed as prebuilt binaries (`taiki-e/install-action`); Node 24 and pnpm 12.14.0, as `package.json`'s `devEngines` names them. The README's commands section names the same versions, so a contributor's machine matches CI.
5. **Every action is pinned by its commit**, not a tag, with the tag in a comment, so an action's later release can't change what runs here; the workflow's token can only read the repository (`permissions: contents: read`), and the workflow uses no secrets. Pull requests from forks run with the same read-only token.
6. **Caches:** the Rust build (`target/`, the registry and git checkouts) by the `Cargo.lock` and the toolchain (`Swatinem/rust-cache`), and pnpm's store by `pnpm-lock.yaml` (`actions/setup-node`). A cold run builds everything; a warm one rebuilds only what changed.
7. **The submodule:** the checkout fetches submodules recursively, so the fork of markdown-rs is at the commit this repository points to. It is public, so no token is needed.
8. **The benches stay local.** The editor's bench types into a real Chrome window, and the processor's times a worker in Chrome and WebKit; on shared runners their timings vary too much to hold a budget, and they need browsers the runner would have to install. They are run before a release, and their numbers stay in the READMEs.
9. **Not a required check, and no badge.** Pushes to `main` go in directly, so CI reports on them rather than gating them; a pull request shows its result before it is merged. GitHub shows each commit's status, so the README carries no badge.
10. **Dependabot keeps the actions current**, weekly, for the GitHub Actions only (`.github/dependabot.yml`): it proposes a pull request when an action this repository pins has a new release, with the new commit. Cargo and npm dependencies are updated by hand, as now.
11. **The site is a workflow of its own** (`pages.yml`, in the website's design), which deploys after a push to `main`. It shares this workflow's setup through a local composite action (`.github/actions/setup`), so the toolchain is set up the same way in both.

## The workflow

```yaml
name: CI
on:
  push: { branches: [main] }
  pull_request: { branches: [main] }
  workflow_dispatch:
permissions: { contents: read }
concurrency: { group: "ci-${{ github.ref }}", cancel-in-progress: true }
jobs:
  ci:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@<sha> # its latest release, by tag
        with: { submodules: recursive }
      - uses: ./.github/actions/setup # the toolchain, the tools, Node, pnpm and the caches
      - run: pnpm install --frozen-lockfile
      - run: cargo make ci
```

- `.github/actions/setup/action.yml`: `rustup show` (which installs the toolchain `rust-toolchain.toml` names), `rustup component add llvm-tools-preview`, `taiki-e/install-action` for the cargo tools and `wasm-bindgen-cli`, `Swatinem/rust-cache`, `pnpm/action-setup` and `actions/setup-node` with the pnpm cache.
- A failed run shows the failing task's output in the step's log; `cargo make` names the task.

## The README

The README's commands section says what CI runs and when, with the tool versions.

## Tests

- The first run, pushed to a branch with a pull request, green; then a deliberate break (a formatting change) on the same branch, red, with the failing task named; then reverted.
- A run's time cold and warm, recorded in this document.

**The first runs (2026-10-30, pull request #1, then `main`).** Green on the first run; red on a deliberate formatting break, with the step's log naming the task (`fmt`) and showing the diff; green again once reverted. A run takes 3 to 6 minutes: 3 min 48 s cold and 3 min 20 s warm on the pull request, and 5 min 50 s for the first run on `main`, which can't use a pull request's caches. Coverage is 99.30% on the runner, as locally.

## Not in this change

- Publishing packages or the crate from CI (releases).
- Updating Cargo and npm dependencies automatically.
- Running on macOS or Windows: the processor has no platform code, and the TypeScript packages run in browsers; one Linux runner is enough until something says otherwise.
