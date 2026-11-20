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

The month's sales:

:: bar-chart[Sales by month: January 12, February 18, March 9, April 22] {data=(md.doc.sales) x=month y=total}

Sideways, with thinner bars:

:: bar-chart[Sales by month, sideways] {data=(md.doc.sales) x=total y=month orientation=horizontal bandPadding=0.6 height=12rem}
