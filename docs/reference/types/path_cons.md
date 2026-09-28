---
title: 'PathCons — a type-level lookup path'
sidebar_label: 'PathCons'
sidebar_position: 10
description: 'The cell of a type-level path, the list of segments that names a route through nested delegation tables so a lookup can reach an entry behind a prefix.'
---

# `PathCons`

One segment of a type-level path: the list that names a route through nested delegation tables, so a
lookup can reach an entry several layers deep.

## Overview

`PathCons<Head, Tail>` expresses a *route* through nested delegation tables as a single type. A bare
component name picks one entry out of a context's table. Here the **context** is the type the
implementation runs against, and its table records which provider it uses for each component.
Sometimes the entry a lookup wants sits behind a layer of indirection: inside a namespace, behind a
namespace it inherits from, or under a prefix. A path names such a route as a list of segments read
left to right, and each segment narrows the lookup a step further. `PathCons` is the cell of that
list, and [`Nil`](nil.md) terminates it, so `PathCons<A, PathCons<B, Nil>>` is the path that steps
through `A` and then `B`.

A path differs from the [`Cons`](cons.md) product list in its unsized bound, even though both are
right-nested and end in `Nil`. `PathCons` declares its head and tail `?Sized`, so a segment may be
an unsized type such as `str`, and code can manipulate the whole path without its parts having a
known size. A product list keys a struct's fields, and its elements are always sized values. A path
keys a lookup, and its segments are pure type-level markers that never need to be `Sized`.

The segments are the markers CGP uses elsewhere. A lowercase dotted name becomes a
[`Symbol`](chars.md) [type-level string](/docs/reference/glossary#type-level-string), and a capitalized name becomes that named type, usually a
component key such as `FooProviderComponent` or a namespace marker. So a path interleaves symbols and
component names, in the form `@a.B.c`, assembled into a `PathCons` chain. You write paths through the
[`Path!`](../macros/path.md) macro. This page covers the list type, and that macro's page covers the
`@`-segment syntax.

## Definition

`PathCons` is a zero-sized struct holding a phantom marker for the head segment and another for the rest:

```rust
pub struct PathCons<Head: ?Sized, Tail: ?Sized>(pub PhantomData<Head>, pub PhantomData<Tail>);
```

`Head` is the first segment, and `Tail` is the remainder, which is another `PathCons` or
[`Nil`](nil.md) at the end. Both bounds are `?Sized`, so any type may occupy a segment. The struct
holds nothing at run time. Like the other type-level building blocks, it exists through
[`PhantomData`](phantom_data.md) so that trait resolution can name and match a route. It is in the
prelude, so `use cgp::prelude::*;` is enough.

## Behavior

`PathCons` supports path concatenation through the
[`ConcatPath`](../traits/formatting/concat_path.md) trait, which appends one path onto the end of
another at the type level. The trait recurses down the list: `PathCons<Head, Tail>` concatenates with
`Other` by keeping `Head` and concatenating `Tail` with `Other`, while [`Nil`](nil.md) concatenates with
`Other` by becoming `Other`. The result is ordinary list append, computed as an associated-type
projection.

[`RedirectLookup`](../providers/redirect_lookup.md) consumes a path. When a namespace entry, an
`open` statement, or a `=>` redirect reroutes a lookup, it produces a
`RedirectLookup<Components, Path>` whose `Path` is a `PathCons` chain. Its impl, which
[`#[cgp_component]`](../macros/cgp_component.md) generates for each component, extends the path with
the component's type parameters through `ConcatPath`, then looks the whole extended path up in
`Components` as one key, through `DelegateComponent`. The path never names a provider itself. It
only says which key to look up, so the same path can resolve to different providers in different
tables, and the entry it reaches may redirect again.

## Examples

A path is a type, so the checks here are type equalities. The
[`cgp_namespace!`](../macros/cgp_namespace.md) entry at the top reroutes one component to a path:

```rust
use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanFoo {
    fn foo(&self);
}

#[cgp_component(MyFoo)]
pub trait CanMyFoo {
    fn my_foo(&self);
}

cgp_namespace! {
    new MyNamespace {
        FooProviderComponent =>
            @MyFooComponent,
    }
}

pub fn demo() {
    // A lowercase segment is a `Symbol`, and a capitalized one names its type.
    let _: PhantomData<PathCons<Symbol!("app"), PathCons<MyFooComponent, Nil>>> =
        PhantomData::<Path!(@app.MyFooComponent)>;

    // A one-segment path.
    let _: PhantomData<PathCons<MyFooComponent, Nil>> = PhantomData::<Path!(@MyFooComponent)>;

    // `ConcatPath` appends one path to another.
    let _: PhantomData<Path!(@a.b.c)> =
        PhantomData::<<Path!(@a.b) as ConcatPath<Path!(@c)>>::Output>;
}
```

The namespace entry expands into a `RedirectLookup` over the one-segment path. `cargo cgp expand`
prints the path in its `Path!` form:

```rust
impl<__Table__> MyNamespace<__Table__> for FooProviderComponent {
    type Delegate = RedirectLookup<__Table__, Path!(@MyFooComponent)>;
}
```

which is `RedirectLookup<__Table__, PathCons<MyFooComponent, Nil>>`. A single-segment path is
`PathCons<Segment, Nil>`, and the empty path is [`Nil`](nil.md) alone.

## When to use it

**You read `PathCons` in an expansion. You write [`Path!`](../macros/path.md), or you wire a namespace.**

- **Decode a `PathCons` chain by reading the segments left to right.** Each segment is one step of the
  route: a [`Symbol`](chars.md) for a lowercase name, or a component or namespace type for a capitalized
  one.
- **Use [`Path!`](../macros/path.md) to write a path** when you need one directly, in its `@a.B.c`
  form.
- **Use a namespace rather than a raw path** for the common case: joining a
  [`cgp_namespace!`](../macros/cgp_namespace.md) or the `open` statement of
  [`delegate_components!`](../macros/delegate_components.md) produces the paths for you.

## Common Mistakes

**A path names a route, not a provider.** A `PathCons` chain describes which key to look up. The
provider it resolves to depends on the table that
[`RedirectLookup`](../providers/redirect_lookup.md) looks it up in, so the same path can resolve to
different providers in different contexts.

**A path list is `?Sized`, unlike the [`Cons`](cons.md) product list.** Its segments are markers rather
than values, so a segment need not have a known size. Expecting a path to behave like a product list of
values is the usual source of confusion.

**Both a path and a product end in [`Nil`](nil.md).** A `Nil` at the end of a `PathCons` chain is the
same terminator doing the same job it does for a record. It does not mean the two lists have been mixed.

**A table's own path keys end in a parameter, not in `Nil`.** A path key written in
[`delegate_components!`](../macros/delegate_components.md) matches every longer path beneath it, so
its last `PathCons` holds a generic parameter where a `Path!` holds `Nil`. The expansion of such a
table shows that parameter rather than `Nil`.

## Related constructs

- [`Nil`](nil.md): the end marker that terminates a `PathCons` chain.
- [`Cons`](cons.md): the product list this one parallels, whose elements are sized values rather than
  `?Sized` markers.
- [`Chars`](chars.md): the [`Symbol`](chars.md) segments a path is built from.
- [`Path!`](../macros/path.md): the macro that folds `@`-segments onto this list.
- [`ConcatPath`](../traits/formatting/concat_path.md): appends one path onto another.
- [`RedirectLookup`](../providers/redirect_lookup.md): looks a path up in a table at resolution
  time.
- [`cgp_namespace!`](../macros/cgp_namespace.md): emits these paths to reroute and register entries.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces): where a path routes a component lookup through a reusable
  table.

## Source

- The type:
  [`path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/path.rs),
  with [`Nil`](nil.md) in
  [`nil.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/types/nil.rs)
- The `ConcatPath` trait and impls:
  [`concat_path.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-base-types/src/traits/concat_path.rs)
- The constructing macro is [`Path!`](../macros/path.md). `RedirectLookup`, which consumes a path, is
  in
  [`redirect_lookup.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/redirect_lookup.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
