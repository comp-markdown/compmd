---
md:
  import:
    - from: "@example/charts@1"
      names: [bar]
    - from: "@example/old@1"
      names: [gauge]
  inputs:
    rating: number = 3
---

# Reading notes

How good was it? :value[3]{of=(string(md.input.rating))} of 5.

:: stars[3 of 5] {value=rating}

Pages read each day:

:: bar[Pages read each day: 30, 52, 18, 41] {values=[30, 52, 18, 41]}

:: gauge[An old gauge: 40] {value=40}
