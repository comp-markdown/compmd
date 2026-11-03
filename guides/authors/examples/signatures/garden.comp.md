---
md:
  version: 1
  import:
    - from: ./parts/garden-kit.comp.md
      names: [plot-card, notice]
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: notice {title="The water goes off on November 15" tone=urgent}

Empty your cans and hoses before then.

::: /notice

## The plots

::: grid {min=12rem gap=m}

:: plot-card[Plot 1, large: Maya Okafor, growing garlic] {number=1 gardener="Maya Okafor" crop=garlic size=large}

:: plot-card[Plot 2, small: Tomás Reyes, growing something new] {number=2 gardener="Tomás Reyes"}

:: plot-card[Plot 3, small: free for next year] {number=3}

::: /grid
