---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=notice {padding=m border-radius=m background=subtle}
params:
  title: string
slots: [default]
---
# {{title}}

{{md.slot}}
```

```compmd component=work-day {padding=m border=subtle border-radius=m}
params:
  date: date
  hours: string
slots: [default, jobs, bring]
---
# Work day: :value[Saturday, November 7, 2026]{of=(md.date(date, "full"))}

{{hours}}.

{{md.slot}}

## On the list

{{md.slot.jobs}}

::: box {padding=s border-radius=s background=subtle}

{{md.slot.bring}}

::: /box
```

::: notice {title="The water goes off on November 15"}

Empty your cans and hoses before then. The taps come back on in March.

::: /notice

::: work-day {date=2026-11-07 hours="From nine to noon"}

The last work day of the year, and the garlic goes in.

:: #bring

**Bring** gloves and a trowel; we bring the tea.

:: #jobs

- Turn the compost
- Mend the gate by plot 4
- Plant the garlic

::: /work-day
