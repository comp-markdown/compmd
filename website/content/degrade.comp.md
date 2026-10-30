---
title: Release notes
fixed: 14
---

# {{md.doc.title}}

::: box {padding=m background=subtle border-radius=m}
:value[14 bugs fixed]{of=(string(md.doc.fixed) + " bugs fixed")} since the last release.
::: /box

::: if {when=(md.doc.fixed > 10)}
A big one: read it before you upgrade.

:: #else
A small one.
::: /if
