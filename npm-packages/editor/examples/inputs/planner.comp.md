---
title: Plan a stay
hotel: Ryokan Hanaya
rates: {standard: 120, deluxe: 180, suite: 260}
breakfast: 18
md:
  version: 1
  inputs:
    guest: string = "Alex"
    arrival: date = (md.addDays(md.today(), 14))
    nights: number = 3
    guests: number = 2
    room: standard | deluxe | suite = deluxe
    breakfast: boolean = true
---

# {{md.doc.title}} at {{md.doc.hotel}}

Inputs are values the reader changes: declared in the frontmatter with a type and a default, read as `md.input.<name>`, and edited by the input components below, or in the bar above the tabs. Every figure on this page follows them.

::: grid {min=14rem gap=m}
:: text-input[Guest: Alex] {bind=guest}
:: date-input[Arrival: in two weeks] {bind=arrival}
:: number-input[Nights: 3] {bind=nights}
:: number-input[Guests: 2] {bind=guests}
:: select-input[Room: deluxe] {bind=room}
:: toggle-input[Breakfast: yes] {bind=breakfast}
::: /grid

## Your stay

::: box {padding=m border=subtle border-radius=m}
**{{md.input.guest}}**, {{md.input.guests}} guests in a {{md.input.room}} room, from :value[arrival]{of=(md.date(md.input.arrival, "full"))} to :value[departure]{of=(md.date(md.addDays(md.input.arrival, int(md.input.nights)), "full"))}: :value[3 nights]{of=(md.number(md.input.nights, 0) + " " + md.plural(md.input.nights, "night", "nights"))}.

| | Price |
| --- | --- |
| Room | :value[$540.00]{of=(md.currency(md.doc.rates[md.input.room] * md.input.nights, "USD"))} |
| Breakfast | :value[$108.00]{of=(md.input.breakfast ? md.currency(md.doc.breakfast * md.input.guests * md.input.nights, "USD") : "not included")} |
| **Total** | **:value[$648.00]{of=(md.currency(md.doc.rates[md.input.room] * md.input.nights + (md.input.breakfast ? md.doc.breakfast * md.input.guests * md.input.nights : 0.0), "USD"))}** |
::: /box

::: if {when=(md.input.nights >= 7.0)}
> [!TIP]
> A week or more: ask about the long-stay rate.

:: #else
Stay seven nights or more for the long-stay rate.
::: /if

::: if {when=(md.daysBetween(md.today(), md.input.arrival) < 0)}
> [!WARNING]
> The arrival date has passed.
::: /if

## The types

| Input | Type | Component |
| --- | --- | --- |
| `guest` | `string` | `text-input` |
| `arrival` | `date` | `date-input` |
| `nights`, `guests` | `number` | `number-input` |
| `room` | `standard \| deluxe \| suite`, an enum | `select-input` |
| `breakfast` | `boolean` | `toggle-input` |
