---
md:
  version: 1
  inputs:
    name: string = ""
    size: small | large = small
    plots: number = 1
    tools: boolean = false
    start: date = (md.doc.workday)
    month: november | december = november
  data:
    - from: ./jobs-{{md.input.month}}.yaml
      as: jobs
title: Elm Street Garden
issue: November 2026
workday: "2026-11-07"
fees: {small: 25, large: 40, tools: 5}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Take a plot for next year

::: grid {min=14rem gap=m}
:: text-input[Your name: none yet] {bind=name}
:: select-input[Size: small] {bind=size}
:: number-input[Plots: 1] {bind=plots}
:: toggle-input[Share the tool shed: no] {bind=tools}
:: date-input[First work day: November 7, 2026] {bind=start}
::: /grid

::: box {padding=m border=subtle border-radius=m}

::: if {when=(md.input.name != "")}

Thank you, **{{md.input.name}}**.

::: /if

:value[1 small plot]{of=(md.number(md.input.plots, 0) + " " + md.input.size + " " + md.plural(md.input.plots, "plot", "plots"))} for the year: :value[$25.00]{of=(md.currency(md.doc.fees[md.input.size] * md.input.plots + (md.input.tools ? md.doc.fees.tools : 0.0), "USD"))}, :value[without the tool shed]{of=(md.input.tools ? "with the tool shed" : "without the tool shed")}. Your first work day is :value[Saturday, November 7, 2026]{of=(md.date(md.input.start, "full"))}.

::: /box

## Jobs by month

:: select-input[Month: november] {bind=month}

::: each {in=(md.data.jobs) as=job join=list}

- {{job}}

::: /each
