---
md:
  import:
    - from: ./dist/rating.js
      names: [rating]
  export:
    pack: "@comp-markdown/rating"
    version: "0.1.0"
    names: [rating]
---

# @comp-markdown/rating

`rating` shows a rating as stars: `value` of `of` (5 unless a use says otherwise), half stars included. It shows a value, a review's score or a recipe's difficulty; it isn't an input.

```md
:: rating[Four of five stars] {value=4}

The cheese was :rating[three and a half of five]{value=3.5 width=5rem} at best.
```

The text in `[…]` is what a reader without the component sees, and the stars' text alternative; without one, they say "Rated 4 of 5". A value is shown to the nearest half star, between 0 and `of`. The stars fit their box, 7.5rem wide unless a use says otherwise (`width`, `height`); they print as drawn, and follow the page's colours.

```compmd component=rating signature {width=7.5rem max-width=100%}
params:
  value: number
  of: number = 5
```
