---
md:
  inputs:
    guests: number = 4
---

# Dinner for :value[4]{of=(string(md.input.guests))}

:: number-input[Guests: 4] {bind=guests}

::: box {padding=m border=subtle border-radius=m}
**To buy:** :value[600 g]{of=(string(md.input.guests * 150) + " g")} of rice, and a lemon.
::: /box

Edit this text, and the page follows it as you type.
