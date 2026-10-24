---
md:
  data:
    - from: ./sales.csv
      as: sales
      schema:
        product: {from: "Product", type: string}
        price: {from: "Unit price", type: number}
        units: {from: "units-sold", type: number?}
    - from: ./plain.tsv
      as: plain
    - from: ./semicolons.csv
      as: euro
      delimiter: ";"
---

::: each {in=(md.data.sales) as=s join=list}
- {{s.product}}: {{s.units}} at {{s.price}}, total :value[0]{of=(md.number(s.price * (s.units == null ? 0.0 : s.units), 2))}
::: /each

::: each {in=(md.data.plain) as=p join=inline}
Plain values are text: {{p.name}} {{p.qty}}
::: /each

::: each {in=(md.data.euro) as=e join=inline sep="; "}
{{e.Produkt}} = {{e.Preis}}
::: /each
