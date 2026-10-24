::: grid {gap=m md:grid-template-columns="repeat(auto-fill, minmax(12rem, 1fr))" lg:gap=2.5rem}
::: + grid-template-areas="'head head' 'side main'"
::: box {grid-area=side padding-inline="s l" background=#f5f5f5 border="1px solid" border-radius=8px}
Side.
::: /box
::: /grid

::: row {gap=bigger justify-content=space-around align-items=middle}
Row.
::: /row
