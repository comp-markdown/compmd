---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

# rating

Each example shows its source, then what it draws.

**A value of five:**

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

:: rating[Four of five stars] {value=4}
````

**Half stars,** to the nearest half:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

:: rating[Three and a half of five] {value=3.5}
````

**Out of ten,** `of=10`, in a wider box:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

:: rating[Seven of ten] {value=7 of=10 width=15rem}
````

**In a sentence,** as an inline component, sized to the text:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

The cheese was :rating[two of five]{value=2 width=5rem} at best, but the bread :rating[five of five]{value=5 width=5rem}.
````
