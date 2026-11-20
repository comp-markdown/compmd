---
md:
  import:
    - from: ./dist/qr-code.js
      names: [qr-code]
  export:
    pack: "@spike/qr-code"
    version: "0.1.0"
    names: [qr-code]
---

# @spike/qr-code

`qr-code` draws a QR code of `value` with [react-qr-code](https://www.npmjs.com/package/react-qr-code) 2.2.0. A reader without it sees the label, which is also the drawing's title.

<!-- The fence below is the build's: it is written again from react-qr-code's types at each build. -->

```compmd component=qr-code signature generated
params:
  value: string
  size: number = 256
  bgColor: color?
  fgColor: color?
  level: L | M | H | Q?
  title: string?
```
