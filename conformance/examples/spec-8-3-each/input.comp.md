::: each {in=(md.doc.changes.filter(c, c.type == "breaking")) as=c join=list}
- {{c.text}} ([#{{c.pr}}](https://github.com/acme/rayplay/pull/{{c.pr}}))

:: #before
## Breaking changes
::: /each

::: each {in=(md.doc.stack) as=s join=inline sep=", " last=" and "}
[{{s.name}}]({{s.url}})
::: /each
