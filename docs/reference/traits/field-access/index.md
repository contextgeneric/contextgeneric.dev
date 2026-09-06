---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Field access

Reading one field of a context by its type-level name, from code that cannot name the context.

## Overview

A CGP implementation most often reads a value out of its **context**, the type a capability runs against,
without naming that type. It reads the field by keying on the field's *name as a type*, so any context
with a matching field satisfies the bound. These traits are the foundation
the ergonomic surface stands on: an [`#[implicit]`](../../attributes/implicit.md) argument, a
[`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md) method, and a
[`UseField`](../../providers/use_field.md) wiring entry all generate a bound on the traits here.

The traits divide on two axes: read versus write, and consumer-side versus provider-side.

The **consumer traits** are what an implementation bounds against on its own context.
[`HasField`](has_field.md) reads a field, and [`HasFieldMut`](has_field_mut.md) adds mutable access.

The **provider-side mirrors** are what a getter component is wired through, so that a context chooses by
wiring which field answers a getter. [`FieldGetter`](field_getter.md) is the read side and
[`MutFieldGetter`](mut_field_getter.md) the write side.

The **lifetime-safe forms** let a getter reach into a nested value without forcing its type to be
`'static`. [`MapField`](map_field.md) is the consumer side and [`FieldMapper`](field_mapper.md) the
provider side, and [`ChainGetters`](../../providers/chain_getters.md) composes them to descend into a
nested context.

## The ideas behind them

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): why a field requirement belongs on
  the implementation rather than the interface.
- [Implicit arguments](/docs/concepts/implicit-arguments): the ergonomic surface built on these traits.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
