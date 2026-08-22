---
sidebar_label: 'ConcatPath'
---

# `ConcatPath`

Joining two type-level paths.

:::info

### Generated machinery

**You are not expected to call `ConcatPath` directly.**
[`cgp_namespace!`](../macros/cgp_namespace.md) and [`RedirectLookup`](../providers/redirect_lookup.md)
use it to extend a route one segment at a time, and [`ChainGetters`](../providers/chain_getters.md) to
descend into a nested context. This page explains the operation, so that a composed path in an
expansion or an error message is legible. The one case for naming it is generic code that composes paths rather than writing one out.

:::

## Overview

A [`Path!`](../macros/path.md) is a type-level list of segments — the route a namespaced component
lookup is redirected along, or the chain of field names a nested getter descends. Composing two such
routes means splicing one list onto the end of another, and `ConcatPath` is that operation:

```rust
pub trait ConcatPath<Other: ?Sized> {
    type Output: ?Sized;
}
```

There is no method and no value. It is a pure type-level computation resolved during trait resolution,
so it names the combined path type and nothing runs.

It is the path-level analogue of [`ConcatProduct`](./concat_product.md), with the same two-impl
recursion over a different spine.

## Usage

**`ConcatPath` is in the prelude** — `use cgp::prelude::*;` names it, which makes it the one member of
the type-level recovery group that needs no import. Its two neighbours each need a different one:
[`StaticString`](./static_string.md) comes from `cgp::core::field::traits`, and
[`StaticFormat`](./static_format.md) — defined in the same crate as this trait — from
`cgp::core::base::traits`.

`Self` is the first path and `Other` the second, and the result is the first's segments followed by the
second's. **Both sides may be unsized**, since path types are markers rather than types anything is
instantiated at.

## Examples

Two paths composed at the type level, which is the operation behind chaining nested accessors:

```rust
use cgp::prelude::*;

type Outer = Path!(@a.b);
type Inner = Path!(@c.d);

type Joined = <Outer as ConcatPath<Inner>>::Output;   // the path @a.b.c.d
```

Note the leading `@` — [`Path!`](../macros/path.md) requires it, and `Path!(a.b)` does not parse.

In generic code the projection usually appears in a bound rather than a type alias, naming the route a
composed getter will take:

```rust
fn descend<Outer, Inner>() -> <Outer as ConcatPath<Inner>>::Output
where
    Outer: ConcatPath<Inner>,
{
    todo!()
}
```

## When to reach for it, and when not

**Reach for it when composing paths in generic code**, which is nested-accessor territory. If you are
writing a path literally, [`Path!`](../macros/path.md) already gives you the whole thing and there is
nothing to concatenate.

- **Use [`ChainGetters`](../providers/chain_getters.md)** rather than composing paths by hand when the
  goal is reaching a field on a nested context. That provider is the construct this operation serves.
- **Use [`ConcatProduct`](./concat_product.md)** for field lists rather than paths. The two recursions
  are the same shape over different spines and are not interchangeable.
- **Use [`StaticString`](./static_string.md)** if what you want is the segments as *text*. This produces
  a type; decoding it means decoding each segment's symbol.

## Under the hood

:::note

### Advanced

This section shows the recursion, which is [`ConcatProduct`](./concat_product.md#under-the-hood)'s over
the path spine.

:::

Two impls, one per spine node. Each node keeps its head segment and rebuilds the tail; the terminator
becomes the other path outright:

```rust
impl<Head: ?Sized, Tail: ?Sized, Other: ?Sized> ConcatPath<Other> for PathCons<Head, Tail>
where
    Tail: ConcatPath<Other>,
{
    type Output = PathCons<Head, <Tail as ConcatPath<Other>>::Output>;
}

impl<Other: ?Sized> ConcatPath<Other> for Nil {
    type Output = Other;   // the second path is substituted whole
}
```

Note the `?Sized` on every parameter, including the associated type — that is what lets both operands
be the unsized markers a path is built from, and it is the one way this recursion differs from
[`ConcatProduct`](./concat_product.md)'s otherwise identical shape.

So the result is the first path's segments followed by the second's, in order, with the cost of
resolution proportional to the first path's length.

It never touches a value; it only names the combined path type, which a getter or a
[`RedirectLookup`](../providers/redirect_lookup.md) then uses to descend.

## Gotchas

**[`Path!`](../macros/path.md) requires a leading `@`.** `Path!(a.b)` does not parse; `Path!(@a.b)` does.
This is the most common way an otherwise-correct `ConcatPath` example fails to build.

**It produces a type, not a joined string.** This is path composition for trait resolution. If you want
the segments as text, decode the symbols with [`StaticString`](./static_string.md).

**Both sides may be unsized**, which is why the trait carries `?Sized` on the parameter and the output —
a detail worth noticing if you write a bound over it and get an unexpected `Sized` error.

**It is in the prelude while [`StaticString`](./static_string.md) is not.** The asymmetry between two
neighbouring traits catches people importing one and assuming the other came with it.

**Order is part of the type.** `@a.b` then `@c.d` is a different path from the reverse, and nothing
normalizes them.

## Related constructs

- [`Path!`](../macros/path.md) — the sugar for the paths this joins.
- [`PathCons`](../types/type_level_spines.md) — the spine underneath.
- [`ConcatProduct`](./concat_product.md) — the product-level analogue.
- [`StaticString`](./static_string.md) — recovering a segment's name as text.
- [`StaticFormat`](./static_format.md) — the lazy formatting counterpart.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the provider that follows a path.
- [`ChainGetters`](../providers/chain_getters.md) — nested accessors, the setting path composition
  serves.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — where paths key a namespace's entries.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable wiring tables and the paths that route into them.

## Source

- [`concat_path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/concat_path.rs)
  — `ConcatPath`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
