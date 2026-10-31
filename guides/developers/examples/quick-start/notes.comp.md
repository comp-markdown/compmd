---
tasks: [Write the outline, Draft the first part, Ask for a review, Publish]
md:
  inputs:
    done: number = 1
---

# The guide, step by step

::: box {padding=m border=subtle border-radius=m}
**:value[1 of 4 done]{of=(string(md.input.done) + " of " + string(size(md.doc.tasks)) + " done")}**

:: number-input[Done: 1] {bind=done}
::: /box

::: each {in=(md.doc.tasks) as=task join=list}
1. {{task}}
::: /each
