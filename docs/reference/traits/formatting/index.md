---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The traits that recover a type-level name as runtime data, as a constant string or written into a formatter, and the one that joins two type-level paths.'
---

# Type-level strings and paths

Recovering a field name as runtime data, and joining the paths a lookup follows.

## Overview

[Context-Generic Programming (CGP)](/docs/) encodes field and variant names as *types*, so that a
name can drive trait resolution. A program eventually needs those names as ordinary strings, to
report a missing field or to build a key, and two of these traits are how a name comes back out,
while the third joins the type-level paths a redirected lookup follows. They sit in three different
homes, which is the group's sharpest edge: one is in the prelude and the other two each need a
different import.

- [`StaticString`](static_string.md) decodes a [`Symbol!`](../../macros/symbol.md) into a compile-time
  `&'static str` constant. This is the one to reach for when you need a name as data. Import it from
  `cgp::core::field::traits`.
- [`StaticFormat`](static_format.md) writes a [type-level
  string](/docs/reference/glossary#type-level-string) into a formatter, and backs the `Display` impl
  on `Symbol`, so a `Display` bound is usually all you need. Reach for the trait itself only where a
  value to format is missing. Import it from `cgp::core::base::traits`.
- [`ConcatPath`](concat_path.md) joins two type-level [`Path!`](../../macros/path.md) routes, the
  path-level analogue of `ConcatProduct`. It is in the prelude.

## The ideas behind them

- [Namespaces](/docs/concepts/namespaces): reusable wiring tables and the paths that route into them.
- [Extensible records](/docs/concepts/extensible-records): where field-name types are put to work at
  scale.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
