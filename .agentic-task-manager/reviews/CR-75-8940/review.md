+++
id = "01a20924-cd84-715c-a79e-72181cad8940"
number = 75
title = "Blog: a new post has no heading of its own"
status = "approved"
tasks = ["CMD-385"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T16:02:16Z
updated = 2026-11-23T16:03:45Z
+++

A new post's title now shows once. A post's page already shows its `title` as its `<h1>`, so a post starts with its first paragraph.

- **`cblo new`** (`npm-packages/blog/src/new.ts`): writes the frontmatter and an empty line, with no `# Title` heading after it.
- **`npm create @compmd/blog`**: its first post (`npm-packages/create-blog/src/create.ts`) no longer starts with `# Hello`.
- **The design** (`docs/blog-tool.md`, decision 3): says so, with the date of the change. It used to say "and a first heading".

## Tests
- `cli.test.ts` checks the new post's text.
- `create.test.ts` builds the new blog and checks that its first post's page has one `<h1>`, the title.
- `cargo make ci` passes.
