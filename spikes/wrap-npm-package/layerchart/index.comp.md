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

<!-- The fence below is the build's: it is written again from layerchart's types at each build. -->

```compmd component=bar-chart signature generated
params:
  data: array?
  x: string?
  y: string?
  orientation: horizontal | vertical = vertical
  bandPadding: number = 0.4
```
