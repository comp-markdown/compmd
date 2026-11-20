---
md:
  import:
    - from: "@spike/bar-chart@0"
      names: [bar-chart]
sales:
  - { month: Jan, total: 12 }
  - { month: Feb, total: 18 }
  - { month: Mar, total: 9 }
  - { month: Apr, total: 22 }
---

# layerchart's BarChart, wrapped

Each use's box has a magenta border. A chart has no aspect ratio of its own, so it fills its box both ways.

**No size:** the column's width, the chart's default height (18rem).

:: bar-chart[Sales by month: January 12, February 18, March 9, April 22] {data=(md.doc.sales) x=month y=total border="2px solid #e5007d"}

**Width only,** `width=20rem`: the default height.

:: bar-chart[Sales by month, narrow] {data=(md.doc.sales) x=month y=total width=20rem border="2px solid #e5007d"}

**Height only,** `height=8rem`, sideways with thinner bars:

:: bar-chart[Sales by month, sideways] {data=(md.doc.sales) x=total y=month orientation=horizontal bandPadding=0.6 height=8rem border="2px solid #e5007d"}

**Fill a box,** `width=24rem height=10rem`:

:: bar-chart[Sales by month, in a box] {data=(md.doc.sales) x=month y=total width=24rem height=10rem border="2px solid #e5007d"}

**A box by its aspect ratio,** `aspect-ratio=3/1`:

:: bar-chart[Sales by month, three to one] {data=(md.doc.sales) x=month y=total aspect-ratio=3/1 border="2px solid #e5007d"}
