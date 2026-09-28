---
title: 'ConcatPath — join two type-level paths'
sidebar_label: 'ConcatPath'
sidebar_position: 3
description: 'Join two type-level paths, as the generated lookup behind namespaces and open does to append a component''s type parameters to its route.'
---

# `ConcatPath`

Joining two type-level paths.

:::info

### Generated machinery

**You are not expected to call `ConcatPath` directly.**
[`#[cgp_component]`](../../macros/cgp_component.md) uses it in the
[`RedirectLookup`](../../providers/redirect_lookup.md) impl it generates for every component, to
append the component's type parameters to the lookup path that namespaces and the `open` statement
route along. This page explains the operation, so that a composed path in an expansion or an error
message is legible. The one case for naming it is generic code that composes paths rather than
writing one out.

:::

## Overview

A [`Path!`](../../macros/path.md) is a type-level list of segments: the route a namespaced or
`open`-dispatched component lookup is redirected along. Composing two such routes means splicing one
list onto the end of another, and `ConcatPath` is that operation. It is the path-level analogue of
[`ConcatProduct`](../type-level/concat_product.md), with the same two-impl recursion over a
different list.

## Definition

`ConcatPath` carries a single associated type and nothing else:

```rust
pub trait ConcatPath<Other: ?Sized> {
    type Output: ?Sized;
}
```

`Self` is the first path and `Other` the second, and `Output` is the first path's segments followed
by the second's. **Both sides may be unsized**, since path types are markers rather than types
anything is instantiated at. The trait has neither a method nor a value: `ConcatPath` is a pure
type-level computation resolved during trait resolution, so it names the combined path type and
nothing runs.

## Usage

**`ConcatPath` is in the prelude**: `use cgp::prelude::*;` names it, which makes it the one member
of the type-level recovery group that comes with the prelude. Its two neighbours each need a
different import: [`StaticString`](./static_string.md) comes from `cgp::core::field::traits`, and
[`StaticFormat`](./static_format.md), defined in the same crate as this trait, from
`cgp::core::base::traits`.

## Examples

Two paths joined at the type level, and a generic signature naming a composed route:

```rust
use cgp::prelude::*;

pub type Outer = Path!(@a.b);
pub type Inner = Path!(@c.d);

pub type Joined = <Outer as ConcatPath<Inner>>::Output;

// Compiles only if the joined path is `@a.b.c.d`.
pub fn assert_joined(path: PhantomData<Joined>) -> PhantomData<Path!(@a.b.c.d)> {
    path
}

// A generic signature naming the composed route.
pub fn descend<Outer: ?Sized, Inner: ?Sized>(
) -> PhantomData<<Outer as ConcatPath<Inner>>::Output>
where
    Outer: ConcatPath<Inner>,
{
    PhantomData
}
```

`assert_joined` compiles only because the joined path is `@a.b.c.d`. Note the leading `@`:
[`Path!`](../../macros/path.md) requires it, and `Path!(a.b)` does not parse. The `?Sized` bounds on
`descend` are needed because a path is an unsized marker.

## When to use it

**Reach for it when composing paths in generic code**, which is rare outside the generated lookup
impls. If you are writing a path literally, [`Path!`](../../macros/path.md) already gives you the
whole thing and there is nothing to concatenate.

- **Use [`ChainGetters`](../../providers/chain_getters.md)** to reach a field on a nested context.
  It chains getters through a [`Product!`](../../macros/product.md) list, not a path, so this trait
  is not involved.
- **Use [`ConcatProduct`](../type-level/concat_product.md)** for field lists rather than paths. The two recursions
  are the same shape over different lists and are not interchangeable.
- **Use [`StaticString`](./static_string.md)** if what you want is the segments as *text*. This produces
  a type; decoding it means decoding each segment's symbol.

## Under the hood

Two impls, one per list node. Each node keeps its head segment and rebuilds the tail; the
terminator becomes the other path outright:

```rust
impl<Head: ?Sized, Tail: ?Sized, Other: ?Sized> ConcatPath<Other> for PathCons<Head, Tail>
where
    Tail: ConcatPath<Other>,
{
    type Output = PathCons<Head, <Tail as ConcatPath<Other>>::Output>;
}

impl<Other: ?Sized> ConcatPath<Other> for Nil {
    type Output = Other;
}
```

Note the `?Sized` on every parameter, including the associated type: that lets both operands be the
unsized markers a path is built from, and it is the one way this recursion differs from
[`ConcatProduct`](../type-level/concat_product.md)'s otherwise identical shape. So the result is the
first path's segments followed by the second's, in order, with the cost of resolution proportional
to the first path's length.

Its one consumer shows the job. For a component `Show<T>`, `#[cgp_component]` generates a
[`RedirectLookup`](../../providers/redirect_lookup.md) provider impl that appends `T` to the path it
was given before reading the table; `cargo cgp expand` shows the bounds:

```rust
impl<__Context__, T, __Components__, __Path__> ShowImpl<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: ShowImpl<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::show(__context__, value)
    }
}
```

That is how `@test.ShowImplComponent` becomes `@test.ShowImplComponent.u64` for a `u64` value, the
key a namespaced or `open` entry is written at.

## Common Mistakes

**[`Path!`](../../macros/path.md) requires a leading `@`.** `Path!(@a.b)` parses, and
`pub type Outer = Path!(a.b);` fails with:

```text
error: expected `@`
```

This is the most common way an otherwise-correct `ConcatPath` example fails to build.

**It produces a type, not a joined string.** This is path composition for trait resolution. If you want
the segments as text, decode the symbols with [`StaticString`](./static_string.md).

**Both sides may be unsized**, which is why the trait carries `?Sized` on the parameter and the output,
a detail worth noticing if you write a bound over it and get an unexpected `Sized` error.

**It is in the prelude while [`StaticString`](./static_string.md) is not.** The asymmetry between two
neighbouring traits catches people importing one and assuming the other came with it.

**Order is part of the type.** `@a.b` then `@c.d` is a different path from the reverse, and nothing
normalizes them.

## Related constructs

- [`Path!`](../../macros/path.md): the sugar for the paths this joins.
- [`PathCons`](../../types/path_cons.md): the list underneath.
- [`ConcatProduct`](../type-level/concat_product.md): the product-level analogue.
- [`StaticString`](./static_string.md): recovering a segment's name as text.
- [`StaticFormat`](./static_format.md): the lazy formatting counterpart.
- [`RedirectLookup`](../../providers/redirect_lookup.md): the provider that follows a path.
- [`#[cgp_component]`](../../macros/cgp_component.md): generates the one impl that uses it.
- [`cgp_namespace!`](../../macros/cgp_namespace.md): where paths key a namespace's entries.
- [`ChainGetters`](../../providers/chain_getters.md): nested accessors, which chain getters rather
  than paths.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces): reusable wiring tables and the paths that route into them.

## Source

- [`concat_path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/concat_path.rs):
  `ConcatPath`

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
