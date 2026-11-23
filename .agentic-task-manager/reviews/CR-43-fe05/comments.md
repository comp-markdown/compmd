## spec/compmd-spec-v1.md:548 · round 1 · t1
>   level: L | M | H | Q?                 # an optional enum

- [x] **Hakan Serce** · 2026-11-21T14:03:09Z · c1
  what is this out of nowhere, where is this explained in spec?
  - **claude:79612601** · 2026-11-21T14:03:40Z · c2
    Fair point: as it stands, the example line shows something the text around it never says.

    It comes from the grammar just above it: `type-decl = type [ "?" ] …`, and `enum` is one of the `type`s. So `L | M | H | Q?` already parses as the enum `L | M | H | Q`, made optional. The processor reads it so (`validate/optional-enum`), and the wrap writes such params, because react-qr-code's `level` is one. That was the design's gap G6: the grammar allowed it, but nothing showed it.

    What's missing is the sentence. I'd add one to the rules under the table: "**An enum may be optional too:** `level: L | M | H | Q?` takes `L`, `M`, `H` or `Q`, or no value; the `?` applies to the whole enum, not to its last member." Or I can drop the example line and leave only the conformance case. Which would you prefer?
    - **Hakan Serce** · 2026-11-21T14:06:30Z · c3
      add the sentence
      - **claude:79612601** · 2026-11-21T14:07:02Z · c4
        Added under the table, as the rule after the one on `?` (§7.3), and the changelog entry says so: 4507703.
