# 20. The configuration's reference

Every key of `compmd.config.json` and every option of `@compmd/vite`'s plugin: its type, its default, an example, and what is checked. Then what the plugin checks when the app builds, and the errors it gives. [Chapter 9](09-packs-from-npm.md) teaches the configuration; this is the list to look things up in.

## Where it is

The configuration is one of these, never both:

- **`compmd.config.json`**, beside the app's `package.json`;
- **the `compmd` key of `package.json`**, as an object. A string there is a pack's document (a pack's own `package.json` names it so), never a configuration;
- **the plugin's `config` option**, which takes the place of both, for a tool that serves a page of its own.

None is an empty configuration: every installed pack is available, strict, and nothing more. The schema, `@compmd/vite/compmd.config.schema.json`, gives an editor completion and checks as you type:

```json
{ "$schema": "./node_modules/@compmd/vite/compmd.config.schema.json" }
```

`readConfig` from `@compmd/vite/config` checks it as the plugin does, without Vite, for any host ([chapter 18](18-other-hosts.md)). It finds every problem at once, and throws a `ConfigError` whose `problems` lists them.

## The keys

### `packs`

An object: by pack name, `@scope/pack`, or by folder path, `./components` or `../shared/ui`, each with its options (`{}` for none). Default: `{}`.

```json
{ "packs": { "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com"] }, "./components": { "trust": "trusted", "ambient": true } } }
```

List a pack only to give it more than the strict sandbox, to make it ambient, or to check its version: every installed pack is available without it. A name must be scoped; a folder is relative to the app's root (Vite's `root`), and must be a folder holding a pack, a `package.json` with a `name` and a document. A folder pack's name is its `package.json`'s `name`, which the policy uses, and which its document's `export.pack` must equal (F022). Under `"imports": "listed"`, these are the packs a document may import. Becomes `policy.packs`, and `ambient`.

### `packs.*.trust`

`"strict"`, `"permissive"` or `"trusted"`. Default: `"strict"`.

```json
{ "packs": { "@acme/reviews": { "trust": "permissive", "host": true } } }
```

- **`strict`**: a sandbox with no network, which sees only its params, slots and box.
- **`permissive`**: a frame of its own, with the network to `network`'s origins, and `host` if `host` is `true`.
- **`trusted`**: in the app's page, built with the app: for a folder pack only. A pack name with `trusted` is an error, since a registry pack is never the app's own code.

A document can never raise a pack's trust ([chapter 8](08-components-and-trust.md)).

### `packs.*.network`

A list of origins a permissive pack's code may reach, each `https://host[:port]` (`http:` only for `localhost`, `127.0.0.1` and `[::1]`). Default: none.

```json
{ "packs": { "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com", "https://tiles.example.com:8443"] } } }
```

An origin alone: no path, query or credentials, no wildcard. Only a permissive pack takes it.

### `packs.*.host`

`true` or `false`: whether a permissive pack's components get `host`, the calls the app gives them (`createSandbox`'s `host`). Default: `false`. Only a permissive pack takes it.

```json
{ "packs": { "@acme/reviews": { "trust": "permissive", "host": true } } }
```

### `packs.*.ambient`

`true` or `false`: whether every document gets the pack's names without importing it. Default: `false`.

```json
{ "packs": { "./components": { "trust": "trusted", "ambient": true } } }
```

An ambient pack's names have the lowest priority, so a document's own definitions and imports win. A document may still import it by name, to say what it depends on. Becomes the request's `ambient`.

### `packs.*.version`

A semver range the installed version must satisfy. Default: none, any version.

```json
{ "packs": { "@kit/charts": { "version": "^2.0.0" } } }
```

Checked when the app builds: a pack at a version outside it stops the build. It says nothing about which version a document gets; the document's import does.

### `imports`

`"any"` or `"listed"`. Default: `"any"`.

```json
{ "imports": "listed" }
```

`any`: a document may import any pack the app has, installed or from its registries. `listed`: only the packs in `packs` and the ambient ones; any other import is F026, saying the host allows only the packs its configuration lists. Becomes `policy.imports`.

### `deny`

A list of `{ "pack", "version"?, "reason" }`: packs the app won't serve, all versions or those in the semver range `version`, with the reason readers see. Default: `[]`.

```json
{ "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }] }
```

`pack` is a scoped name; `reason` isn't empty. A denied import is F026, with the reason. A pack from a registry that is denied is never downloaded. `deny` is the app's own policy, not what keeps readers safe: the sandbox does that for every pack. Becomes `policy.deny`.

### `maxPackSize`

A size: a number of bytes, or a string, a number and `B`, `KB`, `MB` or `GB` (`"5 MB"`, `"2.5 MB"`; a kilobyte is 1,000 bytes). Default: no limit for installed packs; 10 MB for a pack from a registry.

```json
{ "maxPackSize": "5 MB" }
```

- **An installed pack** whose files (its `package.json`, its documents and what they import) are larger is left out: its imports are F026, saying why, and the plugin warns when it builds. Becomes `policy.refused`.
- **A pack from a registry** is checked by the unpacked size its registry's metadata states, before it is downloaded; one larger, or whose metadata states no size, isn't fetched, and is F026. Becomes `registry.maxPackSize`.

### `libraries`

A list of npm packages the app installs, which the frames offer component code by name, besides React, React DOM and Svelte: a name, `"d3"`, or `{ "name", "imports" }` for a package whose components import more names than its own. Default: `[]`.

```json
{ "libraries": ["d3", { "name": "d3-scale", "imports": ["d3-scale", "d3-scale-chromatic"] }] }
```

Each one is built once into `compmd/libs/`, and the frames' import map gives components that copy. A library that isn't installed stops the build. Their names and versions go in the request's `policy.libraries`, so the processor reports a pack whose `peerDependencies` the app can't meet (W121). A document whose components import an offered library works only in hosts that offer it.

### `registries`

An object: by scope, `"@kit"`, with `"*"` for every other scope, each the URL of an npm registry, named in full. Default: none, so a document's import of a pack the app doesn't install is F026.

```json
{ "registries": { "@kit": "https://npm.kit.example/", "*": "https://registry.npmjs.org/" } }
```

Each URL is `https:` (`http:` only for `localhost`, `127.0.0.1` and `[::1]`), without credentials, query or fragment: a private registry's token belongs to the app's proxy, in its `NpmRegistry`'s `headers`. A `/` is added at its end when it has none. Neither a document nor a pack can name a registry. Becomes `virtual:compmd`'s `registry`, which `resolvePacks` takes ([chapter 9](09-packs-from-npm.md)).

### `fetch`

`"host"` or `"direct"`: who reaches the registries. Default: `"host"`.

```json
{ "registries": { "*": "https://registry.npmjs.org/" }, "fetch": "direct" }
```

- **`host`**: the page reaches the registries only through the app's proxy, at `compmd/registry/`, so they learn nothing about readers. The dev server and `vite preview` serve it; a deployed app serves it from its server or a worker ([chapter 18](18-other-hosts.md)).
- **`direct`**: the reader's browser fetches from the registries named, without credentials, which then learn the reader's address and the names and versions of the packs a document imports. For an app with no server; tell your readers.

## The plugin's options

`compmd(options)`, in `vite.config.ts`. Both are optional.

### `folder`

The folder of the app, under Vite's `base`, where the frame documents, their libraries, the packs' built modules and the registry proxy are served. Default: `"compmd/"`.

```ts
compmd({ folder: "sandbox/" });
```

Leading slashes are dropped and a trailing one added: `"/sandbox"` is `sandbox/`. `virtual:compmd`'s `sandbox.base` and `registry` follow it.

### `config`

The configuration itself, as the parsed JSON, in place of `compmd.config.json` and `package.json`'s `compmd` key: for a tool that serves a page of its own, as `@compmd/pack`'s test page does. Default: none, and the plugin reads the files.

```ts
compmd({ config: { packs: { "./": {} } } });
```

It is checked as the file is, and its problems are listed under *the plugin's `config`*; folder packs in it are relative to Vite's root.

## What the plugin checks

At the start of every build and of the dev server, and in dev again when `compmd.config.json`, `package.json` or a file of a pack changes:

1. **The configuration**, with every problem at once (below).
2. **The files.** A configuration in both `compmd.config.json` and `package.json` is an error (*keep one*), and so is a file that isn't JSON.
3. **The packs.** Each listed pack is installed (*`@kit/charts` is listed in the configuration, but it isn't installed: `npm install @kit/charts`*), at the version its `version` asks (*`@kit/charts` is 1.4.0, which isn't `^2.0.0` as the configuration asks*); each listed folder is a folder of the app, holding a pack; each library is installed; and each file a pack's document imports is in the pack, and doesn't leave it.
4. **The listed packs' documents**, through the processor, in one document that imports all of each: an error stops the build (*the listed packs have errors*, each as `file:line:column: code message`), a warning is a Vite warning. F026 is a warning there, since it is how a left-out pack shows. A pack left out by `maxPackSize` is a warning too.

This configuration has a problem in nearly every line:

```json snippet=bad-config/compmd.config.json
{
  "packs": {
    "@acme/maps": { "trust": "trusted", "network": ["https://tiles.example.com/v2"] },
    "charts": {}
  },
  "imports": "some",
  "maxPackSize": "lots"
}
```

The plugin, and `readConfig`, give them all:

```text
compmd.config.json:
- packs["@acme/maps"].trust: only a folder in the app may be trusted, never a registry pack; a registry pack that needs more is `permissive`
- packs["@acme/maps"].network: only a permissive pack reaches the network
- packs["@acme/maps"].network: `https://tiles.example.com/v2` must be an origin alone, without a path
- packs["charts"]: a pack is a scoped name (`@scope/pack`) or a folder path (`./components`)
- imports: `any` or `listed`
- maxPackSize: bytes, or a size such as `"5 MB"`
```

A key the configuration doesn't have, or an option a pack doesn't take, is a problem too, whose message names the keys it takes.

In a build, a problem stops it. On the dev server, a change that brings one is logged, `[compmd] …`, and the app keeps what it had until the problem is fixed.

The plugin finds installed packs among the app's `dependencies` and `devDependencies`: each scoped one whose `package.json` has a `compmd` field or whose folder has `index.comp.md`. Each one's integrity comes from the nearest lock file above the app, `pnpm-lock.yaml` or `package-lock.json`; without one, a document that pins a pack's integrity gets F035.

Next: [the diagnostics](21-the-diagnostics.md).
