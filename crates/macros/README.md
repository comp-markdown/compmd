# compmd-macros

The [`compmd`](https://crates.io/crates/compmd) crate's own macros, for its use only. In development.

`#[library]` writes a family of CompMD's expression functions (spec §8.2): the `md.` functions, or those the profile takes from CEL. Each function is one variant of an enum, with everything about it beside it:

```rust
#[library(prefix = "md.", call = Call)]
pub(crate) enum MdFunction {
    /// An amount written in a currency, by `md.lang`.
    ///
    /// ```cel
    /// let price = 12
    /// md.currency(price * 10, "USD") // "$120.00"
    /// ```
    #[overload(fn(n: Number, code: String) -> String = currency)]
    Currency,
}
```

- **Its name:** the variant in lower camel case: `GetFullYear` is `getFullYear`.
- **Its description:** the first paragraph of its doc, which rustdoc shows too.
- **Its examples:** the doc's ` ```cel ` block. Each line `expression // result` is an example. `let name = value` lines before it give the variables it reads; a value is JSON, or the name of a constant that holds it.
- **Its overloads:** one `#[overload(…)]` for each way to call it. Each param has a name and a type, and `self` comes first for a method. `, size = path` is added when the result can be far larger than the arguments.
- **`#[counts]`:** when its `int` arguments are positions or counts.

A function without a description, an example or an overload doesn't compile, and each example is a test of its own (`md_function_examples::currency_1`). The catalog `compmd::functions::all()` reads it all, so the documentation an editor shows is the documentation that is tested.

## Licence

MIT.
