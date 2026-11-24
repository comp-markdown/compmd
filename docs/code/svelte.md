# Svelte and the TypeScript packages

The repository's own rules for its Svelte and TypeScript code: the website, the Developer Tools, the playground and the packages.

## Basics

- **Svelte 5 with runes** and TypeScript in strict mode. No legacy `$:` statements and no `export let`; props use `$props()`.
- ESLint, Prettier and `svelte-check` pass with no warnings.
- **One lint and one set of compiler options** for every package and pack: its `eslint.config.js` is `lintConfig(import.meta.dirname, [its own ignores])` from the root's `eslint.shared.js`, and its `tsconfig.json` extends the root's `tsconfig.base.json`, adding only its `lib`, `types` and `include`. The guides keep their own, since their examples are written as an author writes them.

## Toolchain

- **pnpm, installed with its [standalone installer](https://pnpm.io/installation).** Not through corepack, which Node.js stops shipping from Node 25 on, and not through a Node version manager.
- **Every `package.json` with a lockfile pins pnpm and Node.js** in `devEngines`, with `"onFail": "download"`, so scripts run on the pinned Node.js, locally and in CI, the way `rust-toolchain.toml` pins Rust.
- A bare `node` typed in a shell may still be another Node.js. Run tools through `pnpm` (`pnpm exec`, `pnpm <script>`).

## Components and styling

- **Small components with typed props.** One component per file, in `PascalCase.svelte`, grouped by area.
- **Scoped `<style>` per component**, except in a package that mounts into a page someone else styles: the Developer Tools keep one stylesheet, `devtools.css`, adopted into the shadow root they mount in, so the page's styles can't reach in and theirs can't leak out. Either way, colors, spacing, radii and type are CSS custom properties. No CSS framework and no inline colors.
- Shared state lives in `.svelte.ts` modules with `$state`, not in global stores.

## Accessibility

- Real `<button>`, `<a>`, `<input>` and `<label>` elements. No click handlers on `div` or `span`.
- Every icon-only button has an `aria-label`.
- Keyboard: everything reachable by Tab, and Esc closes overlays.
- Text contrast of at least 4.5:1.

## Browser storage

- What a page keeps in `localStorage` is a convenience: every read and write is wrapped in `try/catch`, and the page works without it.

## Tests

- **Vitest**, with tests in `foo.test.ts` next to `foo.ts`.
- **Playwright** for what runs in a page, by the [Playwright rules](playwright.md).
