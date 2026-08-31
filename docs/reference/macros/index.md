---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Macros

CGP's procedural macros fall into three jobs: defining a component and writing the providers that
implement it, wiring a concrete type to the providers it uses and checking that wiring holds, and
building the type-level values (strings, lists, paths) that the rest of the system runs on. This
page groups every macro in this section by which of the three it does, in roughly the order most CGP
code reaches for them.

If you are new to CGP, start with the [reference overview](/docs/reference/)'s six essentials rather
than here; this page is the fuller map for once those six are familiar.

## Defining a component, and writing its providers

[`#[cgp_component]`](./cgp_component.md) is the macro every wired capability starts from: it turns one
trait into the consumer trait callers use, the provider trait implementations target, and the marker
that wiring keys against. A provider is then written with [`#[cgp_impl]`](./cgp_impl.md), which keeps
`self`, `Self`, and the consumer trait's own method signatures; [`#[cgp_provider]` and
`#[cgp_new_provider]`](./cgp_provider.md) are the lower-level forms underneath it, mostly met in
generated code rather than written by hand.

When a capability has exactly one implementation and needs no wiring at all,
[`#[cgp_fn]`](./cgp_fn.md) builds it straight from a function, and
[`#[blanket_trait]`](./blanket_trait.md) does the same starting from a trait with default methods and
supertrait dependencies. [`#[async_trait]`](./async_trait.md) is how any of these traits declares an
`async fn` without tripping the lint a bare one produces.

Three macros specialize `#[cgp_component]` for a narrower job. [`#[cgp_type]`](./cgp_type.md) is for a
component that supplies a type a context chooses rather than a value it computes.
[`#[cgp_auto_getter]`](./cgp_auto_getter.md) publishes a context field as a named accessor with a
blanket impl and no wiring, and [`#[cgp_getter]`](./cgp_getter.md) is its wireable counterpart, for the
rarer case where a context needs to choose which field a getter reads.

Three more build a provider from a plain function for CGP's computation family.
[`#[cgp_computer]`](./cgp_computer.md) and [`#[cgp_producer]`](./cgp_producer.md) wire one function to
answer every member of that family at once, synchronous, async, fallible, and infallible alike, and
[`#[cgp_auto_dispatch]`](./cgp_auto_dispatch.md) generates a handler that routes an extensible-data
input to a per-type trait's own implementations.

## Wiring a type, and checking that wiring

[`delegate_components!`](./delegate_components.md) is where a concrete type says which provider
answers each component, in a table that is a compile-time fact rather than a runtime lookup. Because
that wiring is lazy, [`check_components!`](./check_components.md) forces a missing or broken
choice to fail at the table rather than wherever the capability is finally called, and
[`delegate_and_check_components!`](./delegate_and_check_components.md) fuses the two for a newcomer or
a simple table, so the check cannot be forgotten. [`cgp_namespace!`](./cgp_namespace.md) lifts a table
out of any one type and gives it a name other types can join, which is how CGP expresses reusable
presets without a separate construct for them.

## Building type-level values

[`Symbol!`](./symbol.md) turns a string literal into a type-level field-name tag,
[`Product!`](./product.md) and its value-level twin `product!` build a type-level list for a struct's
fields or a handler pipeline's steps, [`Sum!`](./sum.md) builds the dual list for an enum's variants,
and [`Path!`](./path.md) builds the routing list that namespaces and redirected lookups resolve
against. All four are mostly generated for you, by a derive, by the ergonomic macros above, or by the
`open` statement inside `delegate_components!`, and the skill worth having is reading one back out of
a compiler error rather than writing it, since a diagnostic actually prints the raw list each expands
to (`Cons`/`Nil`, `Either`/`Void`, `PathCons`).
