::: box {display=grid grid-template-areas="'head head' 'side main'"}
::: box {grid-area=head}
Top.
::: /box

::: box {grid-area=mian}
Body.
::: /box
::: /box

::: box {display=grid grid-template-areas=("'a b'")}
::: box {grid-area=anything}
An expression: its areas aren't known, so no area is checked.
::: /box
::: /box
