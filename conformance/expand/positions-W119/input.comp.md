::: box
Its height comes from this paragraph only.

::: box {inset-block-start=1rem}
Positioned in a box without a height.
::: /box
::: /box

::: box {md:height=10rem}
::: box {md:inset-inline-start=50%}
Positioned from md up, where the box has a height: no warning.
::: /box
::: /box

::: box {md:height=10rem}
::: box {inset-inline-start=50%}
Positioned below md too, where the box has no height.
::: /box
::: /box

::: box {aspect-ratio=16/9}
::: box {inset-inline-end=0 inset-block-end=0}
Sized by its aspect ratio: no warning.
::: /box
::: /box

::: box {inset-inline-start=1rem}
Positioned at the top level.
::: /box
