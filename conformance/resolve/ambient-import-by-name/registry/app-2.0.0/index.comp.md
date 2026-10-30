---
md:
  export:
    pack: "@acme/app"
    version: "2.0.0"
    names: "*"
---

A package published under the host's pack name, with a `stars` of its own.

```compmd component=stars
params:
  count: number
---
{{count}} stars
```
