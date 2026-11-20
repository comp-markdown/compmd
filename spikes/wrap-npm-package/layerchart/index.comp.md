---
md:
  import:
    - from: ./dist/bar-chart.js
      names: [bar-chart]
  export:
    pack: "@spike/bar-chart"
    version: "0.1.0"
    names: [bar-chart]
---

# @spike/bar-chart

`bar-chart` draws `data`'s rows as bars with [layerchart](https://www.npmjs.com/package/layerchart) 2.11.0's `BarChart`: `x` names the field the bars are placed by, `y` the field their length is. The chart is as tall as its box: `height=12rem` on the use, 18rem without. A reader without it sees the label.

<!-- compmd-pack:generated from layerchart 2.11.0's types and compmd.wrap.json; edits between these comments are overwritten at each build -->
```compmd component=bar-chart signature
params:
  data: array?
  x: string?
  y: string?
  orientation: horizontal | vertical = vertical
  bandPadding: number = 0.4
```
<!-- /compmd-pack:generated -->
