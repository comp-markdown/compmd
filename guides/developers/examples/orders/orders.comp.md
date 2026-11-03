---
md:
  inputs:
    status: open | shipped = open
  data:
    - from: host:orders?status={{md.input.status}}
      as: orders
      schema: {id: string, customer: string, total: number}
      fallback: ./sample-orders.json
    - from: ./delivery.csv
      as: delivery
      schema: {zone: string, days: number}
---

# Orders

:: select-input[Status: open] {bind=status}

::: each {in=(md.data.orders) as=o join=list}
- **{{o.id}}**, {{o.customer}}: {{o.total}} EUR

:: #empty
No orders.
::: /each

:value[2 orders]{of=(string(size(md.data.orders)) + " orders")} in all.

## Delivery

::: each {in=(md.data.delivery) as=d join=list}
- {{d.zone}}: {{d.days}} days
::: /each
