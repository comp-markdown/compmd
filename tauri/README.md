# tauri-plugin-compmd

Serves CompMD's sandbox frames to a [Tauri](https://tauri.app) app from a scheme of its own, so the app can show CompMD documents with their components running in `@compmd/sandbox`. The design, and why each rule is there, is [docs/tauri-plugin.md](../docs/tauri-plugin.md).

## Use

1. **Build the frames** with the app, before `tauri build` and `tauri dev`, into a folder the app bundles as a resource:

   ```js
   import { buildHashedFrames } from "@compmd/sandbox/build";
   await buildHashedFrames({ out: "src-tauri/compmd", source: "compmd://localhost/frames/" });
   ```

   ```json
   "bundle": { "resources": { "compmd/": "compmd/" } }
   ```

2. **Add the plugin**, and grant its `base` to the windows that render documents:

   ```rust
   tauri::Builder::default().plugin(tauri_plugin_compmd::Builder::new().build())
   ```

   ```json
   "permissions": ["compmd:default"]
   ```

3. **Give the page's policy what the frames need**, in `tauri.conf.json`'s `app.security.csp`:

   ```
   frame-src compmd: http://compmd.localhost;
   script-src 'self' compmd://localhost/frames/ http://compmd.localhost/frames/;
   connect-src 'self' ipc: http://ipc.localhost compmd://localhost/frames/ http://compmd.localhost/frames/;
   frame-ancestors 'none'
   ```

4. **Ask for `base`** in the page, and render:

   ```js
   const base = await invoke("plugin:compmd|base");
   const view = createSandbox({ base, page: "host" }).open(container, { components: processed.hostComponents });
   view.render(processed.expanded);
   ```

The scheme (`compmd`) and the resource folder (`compmd`) are the defaults; `Builder::scheme` and `Builder::frames` change them. Never list the scheme in a capability: no frame may reach a command.

## Status

In development, released with CompMD's packages at the same version. Tested on Linux (WebKitGTK) in CI by the example app, `examples/tauri/`, and by hand on macOS; iOS and Android aren't tested yet.
