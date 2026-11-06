---
md:
  import:
    - from: ./spark.js
      names: [spark]
---

| City | This week | Today |
| --- | --- | --- |
| Lisbon | :spark[16–22 °C]{values=[17, 19, 22, 21, 18, 16, 20]} | 20 °C |
| Oslo | :spark[3–9 °C]{values=[3, 5, 4, 7, 9, 6, 6]} | 6 °C |
| Cairo | :spark[24–31 °C]{values=[24, 27, 31, 30, 28, 29, 29]} | 29 °C |

```compmd component=spark signature
params:
  values: array
```
