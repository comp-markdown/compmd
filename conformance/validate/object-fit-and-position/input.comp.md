```js component=photo
export default function mount() {}
```

```compmd component=photo signature
params: {}
```

:: photo[Contained, in the middle] {object-fit=contain object-position=center}

:: photo[Covering, at the end of the block, the start of the line] {object-fit=cover object-position="end start"}

:: photo[Stretched] {object-fit=fill object-position=start}

::: box {object-fit=cover}
A box takes it too, though it means nothing there.
::: /box

:: photo[CSS's none] {object-fit=none}

:: photo[A physical side] {object-position=left}

:: photo[Three positions] {object-position="start center end"}

:: photo[A breakpoint] {md:object-fit=cover}
