---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

Each example of `columns` shows its source, then what it draws.

# Two columns

The first column is what you write first; `:: #end` starts the second.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns
**The harbour at dusk.** Start at the lifeboat station and follow the old quay towards the lighthouse. The lamps come on an hour before the boats come in, one post at a time, from the chapel end.

By the time you reach the fish market the first boats are tying up, and the gulls know it before anyone else does. Stay for the auction if you can: it is over in twenty minutes.

Come back along the harbour wall, which is dry except at the very top of the tide.

:: #end
**The old orchard.** Take the steps behind the chapel up to the lane, and turn left at the stone barn. The orchard is the field after it, open from April to the end of October.

In October the apples are everywhere, and the farm sells them by the bag at the gate. There is a bench at the top of the hill, where the town shows its roofs and the harbour its masts.

The way down is steeper than the way up: take the lane, not the field.
::: /columns
````

# Three columns

`:: #middle` starts a middle one.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns
**Morning**: the harbour. The quay is quiet before eight, and the light comes straight down the channel.

The bakery by the slipway opens at seven; the café next to it an hour later.

:: #middle
**Noon**: the orchard. Walk up through the town and take a picnic: there is nothing to buy on the hill.

The bench at the top is in the sun until three.

:: #end
**Evening**: the canal. The towpath runs flat for four miles, under twelve bridges.

The last bus back leaves the lock at half past eight.
::: /columns
````

# A wide column

`split=wide-start` or `wide-end` makes one column twice the other: a recipe's method beside its ingredients.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-end}
## Ingredients

- 4 apples, sharp ones
- 100 g butter, cold
- 100 g flour
- 100 g brown sugar
- a pinch of cinnamon

:: #end
## Method

Peel, core and slice the apples, and lay them in a buttered dish, a little overlapping. Scatter the cinnamon over them.

Rub the butter into the flour with your fingertips until it looks like breadcrumbs, then stir in the sugar. Don't overwork it: lumps make the best crumble.

Scatter it over the apples without pressing down, and bake at 180 °C for 40 minutes, until the top is golden and the fruit bubbles at the edges. Let it stand for ten minutes before serving.
::: /columns
````

# A wide middle

`split=wide-middle`, with three columns: a story between two notes.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-middle}
*From the harbour master.* The new wall goes in this winter, from November to March.

The slipway stays open throughout.

:: #middle
The quay floods at high water, and has for three hundred years. Twice a month, at the spring tides, the sea comes over the edge and up to the doors of the fish market, and the lamps along the quay stand in it.

This winter a new wall goes in along the seaward side, a metre higher than the old one and built of the same granite. The lamps will be lifted with it, and the market's doors will stay dry.

The work starts at the lighthouse end and moves towards the town, one section a month, so most of the quay stays open while it goes on.

:: #end
*Tide times inside.* Spring tides this month fall on the 4th and the 19th.

High water is about forty minutes later each day.
::: /columns
````

# Right to left

Start and end follow the text's direction: in an Arabic document, `md.lang: ar`, the first column is on the right.

````compmd {render=both}
---
md:
  lang: ar
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-start}
**الميناء عند الغروب.** ابدأ من محطة قوارب النجاة وامشِ على طول الرصيف القديم نحو المنارة. تُضاء المصابيح قبل عودة القوارب بساعة.

عندما تصل إلى سوق السمك تكون القوارب الأولى قد رست، وتعرف النوارس ذلك قبل الجميع.

عُد على طول سور الميناء، فهو جاف إلا في ذروة المد.

:: #end
**البستان القديم.** خذ الدرج خلف الكنيسة الصغيرة إلى الطريق، وانعطف يسارًا عند الحظيرة الحجرية.

التفاح في أكتوبر في كل مكان، وتبيعه المزرعة بالأكياس عند البوابة.
::: /columns
````
