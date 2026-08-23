---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Type-level strings and paths

Recovering a field name or a routing path as runtime data.

## Overview

CGP encodes field and variant names as *types*, so that a name can drive trait resolution. A program
eventually needs those names as ordinary strings — to report a missing field, to build a key, to render
a path — and these traits are how a name comes back out. The three sit in three different homes, which
is the group's sharpest edge: one is in the prelude and the other two each need a different import.

- [`StaticString`](static_string.md) decodes a [`Symbol!`](../../macros/symbol.md) into a compile-time
  `&'static str` constant. This is the one to reach for when you need a name as data. Import it from
  `cgp::core::field::traits`.
- [`StaticFormat`](static_format.md) writes a type-level string into a formatter, and backs the
  `Display` impl on `Symbol`, so a `Display` bound is usually all you need. Reach for the trait itself
  only where there is no value to format. Import it from `cgp::core::base::traits`.
- [`ConcatPath`](concat_path.md) joins two type-level [`Path!`](../../macros/path.md) routes, the
  path-level analogue of `ConcatProduct`. It is in the prelude.

## The ideas behind them

- [Namespaces](/docs/concepts/namespaces) — reusable wiring tables and the paths that route into them.
- [Extensible records](/docs/concepts/extensible-records) — where field-name types are put to work at
  scale.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
