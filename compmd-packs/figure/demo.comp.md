---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

Each example of `figure` shows its source, then what it draws.

# An image and its caption

The caption is the image's alt too.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

:: figure[Boats at the quay, the lamps just lit.] {src="images/harbour.svg"}
````

# A narrower figure

`width=20rem`, in a post's text.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

From the quay the path climbs to the orchard.

:: figure[Rows of apple trees on the slope above the town.] {src="images/orchard.svg" width=20rem}

At the top there is a bench.
````

# Its box changed by the use

As any box's: a border and padding around the image and its caption.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

:: figure[The harbour, framed.] {src="images/harbour.svg" border=subtle padding=m width=24rem}
````
