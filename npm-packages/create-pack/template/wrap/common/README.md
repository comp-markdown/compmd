# __pack__

A [CompMD](https://compmd.dev/) pack of [__from__](https://www.npmjs.com/package/__from__) __version__: its component as `__component__`, one built module that a host loads by URL and runs in its sandbox. Each version of this pack is one build of __from__ at one version, named in `package.json`'s `dependencies`.

## Use it in a document

```md
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[What a reader without it sees] {__use__}
```

The component fills its box, which the use sizes with `width`, `height` or `aspect-ratio`.

## What is whose

- `compmd.wrap.json`: what the pack wraps (`from`, `export`, `kind`), the props that are params (`params`, an allowlist: a prop __from__ adds later is reported, never added), and what you say over the types (`overrides`: a param's `name`, `type`, `default` and `doc`; `input`, a prop that is an input with the callback that reports it; `label`, the prop that takes the fallback's text; `slots`; `box`, the box's defaults).
- `index.comp.md`: the pack's document. The signature fence between its `compmd-pack:generated` comments is written at each build, from __from__'s types and `compmd.wrap.json`; everything else is yours. Remove the comments to make the fence yours too: the build then leaves it.
- `src/__component__.ts`: the glue that mounts the component, written once and yours to change.
- `demo.comp.md`: the test page's document.

## Moving to __from__'s next version

```sh
npm install __from__@<version>
npm run build
```

The build writes the params the new types give, and says what changed: a param added, one gone, one whose type changed. A param that went fails the build while the demo or the overrides still name it. Then change the version in `package.json` and `index.comp.md`'s `export.version`.

## What a strict frame refuses

A host runs the pack's components in a strict frame, with no network: the build warns of what the module has that such a frame refuses (a URL of another origin, a worker, code made from text, WebGL). A component that needs one of them needs its files in the pack, a permissive level with the origins it reaches, or another package.

## Try it

```sh
npm run dev     # the test page, in the strict sandbox
npm test        # the component in happy-dom
npm run build   # the signature, the module, then the checks before publishing
```

## Publish it

1. Change the version in both `package.json` and `index.comp.md`'s `export.version`, the same in each; the build checks they match.
2. `npm login`, then `npm publish --access public`. It builds, checks and tests first.
