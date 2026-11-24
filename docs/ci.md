# CI on GitHub

**Status: accepted.**

Every check of this repository runs only on a contributor's machine: `cargo make ci` runs them all, and nothing stops a push or a pull request that breaks one. CI on GitHub runs the same command on every push to `main` and every pull request, so a break shows on the commit that made it, and a pull request shows whether it passes before it is merged.

## Decisions

1. **CI runs `cargo make ci`, the command a contributor runs.** One local command checks what CI checks: formatting, clippy with warnings as errors, the tests with the conformance set, the crates' API docs with no broken intra-doc link (`docs`), the WebAssembly build, cargo-deny, line coverage of at least 99%, and every TypeScript package built, linted and tested. Clippy, coverage, the docs and the WebAssembly builds run `--locked`, so a `Cargo.lock` that doesn't match the manifests fails them instead of being rewritten (added 2026-11-24, audit S30). The workflow sets up the tools and calls it; it holds no check of its own, so the two can't drift. Coverage runs the tests, the conformance set's among them, so `ci` doesn't run them a second time; the documentation examples, which coverage leaves out, run on their own (`doctest`). The packages' lint and tests keep their dependency order too: in any order (`--no-sort`) they gained 6 s on the runner's four cores, and tests sensitive to timing failed under the extra load.
2. **One job, on one runner.** `ubuntu-24.04`, named rather than `ubuntu-latest`, so a new image doesn't change the run unannounced. The checks run one after another, as they do locally; splitting them over parallel jobs would give an answer sooner, but each job would build the workspace again, and the workflow would have to repeat `Makefile.toml`'s list. Measured again (2026-11-06, *Speed* below): with the checks themselves faster, a second job would save less than its own setup costs.
3. **When it runs:** on every push to `main`, on every pull request to `main`, and by hand (`workflow_dispatch`). A newer push to the same branch or pull request cancels the run still going for it (`concurrency`), since only the latest commit's answer matters.
4. **The tools are pinned, as locally.** The Rust toolchain from `rust-toolchain.toml`, with `llvm-tools-preview` for coverage; `wasm-bindgen-cli` at the version the workspace's `Cargo.toml` pins in `[workspace.dependencies]` (`0.2.129`); cargo-make, cargo-deny and cargo-llvm-cov at named versions, installed as prebuilt binaries (`taiki-e/install-action`); Node 24 and pnpm 12.14.0, as `package.json`'s `devEngines` names them. The README's commands section names the same versions, so a contributor's machine matches CI.
5. **Every action is pinned by its commit**, not a tag, with the tag in a comment, so an action's later release can't change what runs here; the workflow's token can only read the repository (`permissions: contents: read`), and the workflow uses no secrets. Pull requests from forks run with the same read-only token.
6. **Caches:** the Rust build (`target/`, the registry and git checkouts) by the `Cargo.lock` and the toolchain (`Swatinem/rust-cache`), and pnpm's store by `pnpm-lock.yaml` (`actions/setup-node`). A cold run builds everything; a warm one rebuilds only what changed. Playwright's WebKit and the system libraries it needs are cached too (`.github/actions/webkit`), by Playwright's version and the runner image's: their download from apt's mirrors took 11 to 41 s, and once hung for over 15 minutes. While GitHub rolls out an image, runners have either version, so a runner without its own image's cache restores one another image saved with the same Playwright (added 2026-11-18, after a runner on the other version missed the cache and the mirror timed out twice). A download comes from Ubuntu's own archive, not the runner's regional mirror, which the image lists first: on 2026-11-18 the regional one served packages at under 100 KB/s, so 125 MB couldn't come in 10 minutes and no cache was ever saved. On a hit, the libraries are installed from their saved packages with nothing downloaded; on a miss, Playwright installs them with apt, whose downloads are saved, and apt retries a mirror that doesn't answer within 30 s. The step stops after 10 minutes, so a download that still hangs fails the run instead of holding it.
7. **The submodule:** the checkout fetches submodules recursively, so the fork of markdown-rs is at the commit this repository points to. It is public, so no token is needed.
8. **The benches stay local.** The editor's bench types into a real Chrome window, and the processor's times a worker in Chrome and WebKit; on shared runners their timings vary too much to hold a budget, and they need browsers the runner would have to install. They are run before a release, and their numbers stay in the READMEs.
9. **Not a required check, and no badge.** Pushes to `main` go in directly, so CI reports on them rather than gating them; a pull request shows its result before it is merged. GitHub shows each commit's status, so the README carries no badge.
10. **Dependabot keeps the actions current**, weekly, for the GitHub Actions only (`.github/dependabot.yml`): it proposes a pull request when an action this repository pins has a new release, with the new commit. Cargo and npm dependencies are updated by hand, as now.
11. **The site is a workflow of its own** (`pages.yml`, in the website's design), which deploys after a push to `main`. It shares this workflow's setup through a local composite action (`.github/actions/setup`), so the toolchain is set up the same way in both. Its check of the built site (`website/scripts/check.ts`) isn't part of `cargo make ci`, whose minutes every change would pay: `cargo make site` runs the site's build and check as `pages.yml` does, and a change to `website/`, `guides/` or the spec runs it before it is pushed, besides `cargo make ci` (added 2026-11-06, after a check left behind by a website change was found only once pushed); `cargo make linux site` runs it with the runner's fonts (decision 12).
12. **Linux failures show locally, in a container like the runner.** Some checks fail only on the runner: at 0.4.0 the site's check found `/` (phone, light) a pixel wider than the window, with the runner's fonts, and passed on a Mac. `cargo make linux` runs a check on Linux before a push, with Docker:
    - **The machine** (`tools/linux-ci/Dockerfile`): Ubuntu 24.04 on x86-64, as `ubuntu-24.04` is; the runner image's Chrome (Google's `google-chrome-stable`) and its fonts, the `fonts-*` packages CI listed on the runner (2026-11-06): DejaVu, which its `sans-serif`, `serif`, `monospace` and `system-ui` are, Lato, Liberation, Noto Color Emoji and Ubuntu Console (without DejaVu, `sans-serif` is Liberation Mono, which made 0.4.0's page 8px too wide, not 1px); and the tools at CI's versions, the cargo tools as their releases' prebuilt binaries. On an Arm Mac x86-64 runs emulated: slower, but Chrome and its fonts are the runner's.
    - **The code**: a commit, HEAD by default or `--ref <commit>`, checked out from the repository's git folder (mounted read-only, also for a submodule or worktree) with its tags and the markdown-rs submodule. Not the working tree, so nothing the host built or installed for macOS gets in; commit first, then check before pushing.
    - **What it runs**: `cargo make ci` (with WebKit installed, as CI does), `site` (the site's build and check, as `pages.yml` runs them), `tauri` (the Tauri check with WebKitGTK under a virtual display, as `tauri.yml` runs it), or any command given.
    - **Caches**: the Rust build, cargo's registry, pnpm's store and Playwright's browsers in named volumes (`compmd-linux-*`), so a second run reuses them.

13. **The Tauri check is a workflow of its own** (`tauri.yml`, added 2026-11-09; [the Tauri plugin](tauri-plugin.md), decisions 9 and 10): `cargo make tauri-check` on `ubuntu-24.04`, with WebKitGTK and GTK from apt and the app under `xvfb-run`. It builds `tauri-plugin-compmd` with its frames and the example app (`crates/tauri/examples/tauri-app/`), and runs the app with `--check` and `--check-broken` in a real webview, which serves the sandbox's frames from a scheme of the app's own: WebKit's `'self'` and its cache there are what no browser test shows. It runs on a push or a pull request that changes what it covers (the sandbox, the renderer, the processor, the Mermaid pack, the plugin or the example), by hand, and before every release, which calls it (`workflow_call`) and waits for it. It isn't part of `cargo make ci`, whose minutes every change would pay for Tauri's build. Its first step, `tauri-plugin`, also runs cargo-deny over the plugin, against its own `crates/tauri/deny.toml` (added 2026-11-24, audit S30): the root's licences, with MPL-2.0 allowed only for the five crates Tauri brings in under it (cssparser, cssparser-macros, dtoa-short, selectors, option-ext), and `proc-macro-error`'s advisory ignored, since it comes through Tauri's GTK on Linux with no upgrade. Its Cargo projects are cached on their own (`Swatinem/rust-cache`, two workspaces). macOS's WKWebView is checked by hand before a release, with the same command: a macOS runner costs about ten times the minutes.
14. **The links of the published packages are checked, with lychee** (added 2026-11-16). npmjs.com, crates.io and docs.rs show a package's README and API docs away from the repository, where a link relative to it breaks: the published READMEs, the Rust doc comments and the TypeScript doc comments that ship in the `.d.ts` link to the repository's files on GitHub (`https://github.com/comp-markdown/compmd/blob/main/<path>`, `/tree/main/` for a folder) and to the spec and the guides on https://compmd.dev/, never relatively. The docs that aren't published (`docs/`, the guides, the root README) keep their relative links, read on GitHub and the site. What is checked is listed in `lychee-inputs.txt`, how in `lychee.toml`: the anchors too; the 429 of a site that limits a checker's rate accepted; the sources' placeholder hosts and local servers left out, and the tests; npmjs.com left out, since it answers a checker 403 on every page. **The website's and the playground's sources too** (added 2026-11-19), their `.svelte` files read as text: a link to the repository's files in them is written out in full, not built from a constant, so it is checked as the published docs' are; their links to the site's own pages stay relative, and the site's analytics are left out, which a check would count as a visit.
    - **Locally, offline:** `cargo make links` checks a link to this repository's files on GitHub, or to the spec on compmd.dev, against the file here, with its anchor (lychee's `--remap`; the registry companion's `#registry-…` ids against `compmd-registry-v1.md`), so a renamed file or section shows before a push. It needs lychee (0.24.2); it isn't part of `ci`, which needs no tool beyond decision 4's.
    - **In CI, with the network:** a job of its own in `ci.yml`, `links`, on pushes to `main` only, not on pull requests: a site that is down for a moment would fail a pull request that changed nothing of it. It runs lychee 0.24.2 with `lycheeverse/lychee-action`, pinned by its commit (decision 5), on the same inputs and configuration, and fails the run on a broken link. A link to a file a commit adds is checked once that commit is on `main`, as the job runs after the push. It is one of the two checks CI has that `cargo make ci` doesn't (decision 1), since it needs the network; the other is the spec's revision (decision 15), which a contributor's work in progress may not pass yet.
    - **Intra-doc links:** the crates deny a broken one (`[workspace.lints.rustdoc]`, the plugin's `[lints.rustdoc]`); `cargo make docs`, part of `ci`, builds the docs with every feature, as docs.rs does (`compmd-host`'s `[package.metadata.docs.rs]`), and `tauri-plugin` the plugin's.
15. **The spec on `main` is a revision** (added 2026-11-17, [releases](releases.md), decision 1). The `spec-revision` job runs `node tools/spec-revision/spec-revision.ts check` on pushes to `main`: it fails when the spec or its companion changed since the tag of the revision their header names, when the tag is missing, or when the changelog has entries under `### Unreleased`. It isn't part of `cargo make ci`, so a spec change is written and checked locally before its revision is cut (`cargo make spec-revision`), and a pull request may carry one. The repository's own tools in `tools/` are type-checked (`tools/packs/tsconfig.json`, added 2026-11-24, audit B27) and tested by `cargo make ci` (`scripts`).

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
  links: # decision 14
    if: github.event_name == 'push'
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@<sha>
      - uses: lycheeverse/lychee-action@<sha> # with lychee 0.24.2
        with: { args: --config lychee.toml --files-from lychee-inputs.txt, fail: true }
```

- `.github/actions/setup/action.yml`: `rustup toolchain install` (the toolchain `rust-toolchain.toml` names), `rustup component add llvm-tools-preview`, `taiki-e/install-action` for the cargo tools and `wasm-bindgen-cli`, `Swatinem/rust-cache`, `pnpm/action-setup` and `actions/setup-node` with the pnpm cache.
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
