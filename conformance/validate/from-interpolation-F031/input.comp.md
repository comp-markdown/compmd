---
city: Berlin
md:
  inputs:
    region: all | eu = all
    picked: string = ""
  data:
    - from: https://api.example.com/sales?region={{md.input.region}}&city={{md.doc.city}}
      as: sales
    - from: https://{{md.doc.city}}.example.com/data.json
      as: hosted
    - from: ./data-{{md.data.sales}}.json
      as: chained
    - from: https://api.example.com/x?p={{md.input.picked}}
      as: leaked
---

```compmd component=chart signature
params:
  selected: input<string>
```

```vue component=chart
<template><svg /></template>
```

:: chart[Sales] {selected=picked}
