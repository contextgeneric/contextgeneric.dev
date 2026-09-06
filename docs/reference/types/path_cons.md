---
sidebar_label: 'PathCons'
sidebar_position: 10
---

# `PathCons`

One segment of a type-level path: the list that names a route through nested delegation tables, so a
lookup can reach an entry several layers deep.

## Overview

`PathCons<Head, Tail>` expresses a *route* through nested delegation tables as a single type. A bare
component name picks one entry out of a context's table, where the **context** is the type the capability
runs against and the table records which provider it uses for each component. Sometimes the entry a
lookup wants lives behind one or more layers of indirection: inside a namespace, behind a namespace it
inherits from, under a prefix. A path names such a route, as a list of segments read left to right, each
one narrowing the lookup a step further. `PathCons` is the cell of that list, and [`Nil`](nil.md)
terminates it, so `PathCons<A, PathCons<B, Nil>>` is the two-step path "first `A`, then `B`".

A path differs from the [`Cons`](cons.md) product list, even though both are right-nested and
`Nil`-terminated, and the difference is the unsized bound. `PathCons` declares its head and tail
`?Sized`, so a segment may be a trait object or another unsized type and the whole path can be
manipulated without its parts having a known size. A product list keys a struct's fields and its
elements are always sized values; a path keys a lookup and its segments are pure type-level markers that
never need to be `Sized`.

The segments are the markers CGP uses elsewhere: a lowercase dotted name becomes a
[`Symbol`](chars.md) type-level string, and a capitalized name becomes that named type, usually a
component key such as `FooProviderComponent` or a namespace marker. A path is therefore an interleaving
of symbols and component names, the form `@a.B.c`, assembled into a `PathCons` chain. You write paths
through the [`Path!`](../macros/path.md) macro, so this page is the runtime list and that macro's page
carries the `@`-segment syntax.

## Definition

`PathCons` is a zero-sized struct holding two phantom markers, one for the head segment and one for the
rest:

```rust
pub struct PathCons<Head: ?Sized, Tail: ?Sized>(pub PhantomData<Head>, pub PhantomData<Tail>);
```

`Head` is the first segment and `Tail` is the remainder, expected to be either another `PathCons` or
[`Nil`](nil.md) at the end. Both bounds are `?Sized`, so any type may occupy a segment. The struct carries
no runtime data: like the other type-level building blocks it exists through
[`PhantomData`](phantom_data.md) so a route can be named and matched in trait resolution.

## Behavior

`PathCons` supports path concatenation through the
[`ConcatPath`](../traits/formatting/concat_path.md) trait, which appends one path onto the end of
another at the type level. The trait recurses down the list: `PathCons<Head, Tail>` concatenates with
`Other` by keeping `Head` and concatenating `Tail` with `Other`, while [`Nil`](nil.md) concatenates with
`Other` by becoming `Other`. The result is ordinary list append, computed as an associated-type
projection.

Beyond concatenation, a path is consumed by [`RedirectLookup`](../providers/redirect_lookup.md), the
provider that resolves a delegation by walking a context's table along a path. When a namespace or a
prefixed component reroutes a lookup, it produces a `RedirectLookup<Components, Path>` whose `Path` is a
`PathCons` chain, and `RedirectLookup` follows the chain segment by segment until it lands on a concrete
provider. The path never names a provider itself; it only describes where to look, so the same path can
resolve to different providers depending on the table it is walked against.

## Examples

Paths are produced by the [`Path!`](../macros/path.md) macro and most often appear inside the wirings
that [`cgp_namespace!`](../macros/cgp_namespace.md) emits. A namespace entry that redirects one
component key to a path desugars into a `RedirectLookup` over a `PathCons` chain:

```rust
use cgp::prelude::*;

cgp_namespace! {
    new MyNamespace {
        FooProviderComponent =>
            @MyFooComponent,
    }
}

// the emitted entry, in readable form:
// impl<__Table__> MyNamespace<__Table__> for FooProviderComponent {
//     type Delegate = RedirectLookup<__Table__, PathCons<MyFooComponent, Nil>>;
// }
```

A path with both a lowercase symbol segment and a capitalized component segment interleaves the two
marker kinds. Registering a component into a namespace under a prefix produces a two-segment path:

```rust
// @MyBarComponent.BarProviderComponent  expands to
// PathCons<MyBarComponent, PathCons<BarProviderComponent, Nil>>
```

The lookup steps first through `MyBarComponent` and then through `BarProviderComponent` before
resolving. A single-segment path is `PathCons<Segment, Nil>`, and the empty path is [`Nil`](nil.md) alone.

## When to use it

**You read `PathCons` in an expansion; you write [`Path!`](../macros/path.md), or wire a namespace.**

- **Decode a `PathCons` chain by reading the segments left to right.** Each is one step of the route, a
  [`Symbol`](chars.md) for a lowercase name or a component or namespace type for a capitalized one.
- **Use [`Path!`](../macros/path.md) to write a path** when you need one directly, in its `@a.B.c`
  form.
- **Use a namespace rather than a raw path** for the common case: joining a
  [`cgp_namespace!`](../macros/cgp_namespace.md) or the `open` statement of
  [`delegate_components!`](../macros/delegate_components.md) produces the paths for you.

## Common Mistakes

**A path names a route, not a provider.** A `PathCons` chain describes where to look; which provider it
resolves to depends on the table [`RedirectLookup`](../providers/redirect_lookup.md) walks it against.
The same path can land on different providers in different contexts.

**A path list is `?Sized`, unlike the [`Cons`](cons.md) product list.** Its segments are markers rather
than values, so a segment need not have a known size. Expecting a path to behave like a product list of
values is the usual source of confusion.

**Both a path and a product end in [`Nil`](nil.md).** Seeing `Nil` at the end of a `PathCons` chain is
the same terminator doing the same job it does for a record, not a sign the two lists have been mixed.

## Related constructs

- [`Nil`](nil.md): the end marker that terminates a `PathCons` chain.
- [`Cons`](cons.md): the product list this one parallels, whose elements are sized values rather than
  `?Sized` markers.
- [`Chars`](chars.md): the [`Symbol`](chars.md) segments a path is built from.
- [`Path!`](../macros/path.md): the macro that folds `@`-segments onto this list.
- [`ConcatPath`](../traits/formatting/concat_path.md): appends one path onto another.
- [`RedirectLookup`](../providers/redirect_lookup.md): walks a path against a table at resolution
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
- The constructing macro is [`Path!`](../macros/path.md); `RedirectLookup`, which consumes a path, is
  in
  [`redirect_lookup.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/redirect_lookup.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
