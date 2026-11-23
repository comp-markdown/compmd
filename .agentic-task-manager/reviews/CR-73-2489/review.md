+++
id = "01a20909-c1fd-7641-88ca-423a86d62489"
number = 73
title = "Blog pages follow a save: cblo dev and the website dev server"
status = "approved"
tasks = ["CMD-383"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T15:32:44Z
updated = 2026-11-23T15:59:33Z
+++

The blog's open pages follow a save, under `cblo dev` and the website's dev server, as the four decisions settled on 2026-11-23 say. The decisions are now in `docs/blog-tool.md` (decision 3) and `website/docs/blog.md` (decision 2).

## `@compmd/blog`
- **`blog-change.ts`**: `changeOf(sources, file)` says what a saved file changes:
  - `page`: a post or a post's file, `blog.config.json`, `theme.css`, `layout.html`;
  - `client`: `compmd.config.json`, `package.json`, a lock file, a folder pack;
  - nothing: other files, an editor's swap and backup files, `node_modules`.

  `blogSources(dir)` finds the folder packs the blog's `compmd.config.json` names.
- **`client-cache.ts`**: `ClientCache` replaces `BuildOptions.reuseClient` and the module-level map. The dev server owns it, and passes it as `BuildOptions.clients`. A build that fails isn't kept.
- **`blog-watch.ts`**: `watchBlog` watches:
  - the blog's top level, not recursively, so `node_modules` isn't watched (on Linux, every folder would cost an inotify watch);
  - `posts/` and each folder pack, recursively.

  The saves that come together within 50 ms settle into one change.
- **`reload-events.ts`**: the server-sent events at `/_cblo/reload`, and `withReload`, which adds the few lines of script to the HTML pages `cblo dev` serves. `cblo build` output is unchanged.
- **`serve.ts`**:
  - `cblo dev` drops the cached script on a `client` change and tells every open page to reload.
  - Its error page is HTML now, with the script, so it follows the fix.
  - Its server (`DevServer`) stops the watch and the streams on `close()`, so it can close.

## The website
- **How the pages are told**: `blogInDev` listens on Vite's watcher, using `changeOf` and the files the site shares with its posts.
- **One change from decision 2**: it doesn't send Vite's `full-reload`. That reloaded every open page of the site, and the dev test caught it, mid-check on a guide page. Instead it sends a custom event, `compmd:blog-changed`, on Vite's channel. Only the blog's pages listen for it, through `src/blog-reload.ts`, a module the dev server adds to their template. The doc says so.

## Tests
- Unit tests for each new module, and in `cli.test.ts`, the stream telling the page and the error page.
- `test/reload.test.ts`: a post open in Chrome and WebKit follows a save under `cblo dev`.
- `website/build/dev.test.ts`: the open `/blog/` list shows a post added while it is open, in Chrome and WebKit (the post is removed afterwards).
- `cargo make ci` and `cargo make site` pass.

## Worth checking
- `BuildOptions.reuseClient` is gone and `clients` replaces it: a change to the package's API (0.9.0, before 1.0). No release note has been written yet.
- The folder packs are read once, when `cblo dev` starts: a folder pack that a later `compmd.config.json` adds is used (the script is built again) but isn't watched until the server restarts.
