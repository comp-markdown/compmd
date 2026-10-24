---
changes:
  - {text: "New renderer", pr: 412}
  - {text: "Config format 2", pr: 398}
stack:
  - {name: Rust}
  - {name: wgpu}
  - {name: WebAssembly}
none: []
---

::: each {in=(md.doc.changes) as=c join=list}
- {{c.text}} (#{{c.pr}}), {{md.loop.number}} of {{md.loop.count}}

:: #before
## Changes
::: /each

::: each {in=(md.doc.stack) as=s join=inline sep=", " last=" and "}
{{s.name}}
::: /each

::: each {in=(md.doc.none) as=x}
{{x}}
:: #empty
Nothing yet.
::: /each

::: each {in=(md.doc.changes) as=c}
Paragraph for {{c.text}}.
::: /each
