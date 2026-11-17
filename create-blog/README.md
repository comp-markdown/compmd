# @compmd/create-blog

Writes a new blog in [CompMD](https://compmd.dev/), built with
[`@compmd/blog`](https://github.com/comp-markdown/compmd/tree/main/blog): its configuration, a first post, and the
scripts to write, see and build it.

```sh
npm create @compmd/blog my-notes
cd my-notes
npm install
npm run dev
```

Options, after `--`: `--title "My notes"` (the folder's name by default), `--author "Ada Lovelace"` (git's
`user.name` by default), `--url https://example.com/notes/` (where it is published; set it later in
`blog.config.json` otherwise). It asks nothing, so it scripts the same way it runs. The new blog pins `@compmd/blog` at
this package's version, as the two are released together.

## Licence

MIT.
