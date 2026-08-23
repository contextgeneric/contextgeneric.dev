---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Providers

A provider is a zero-sized type wired to a component to say how that component is implemented for a
context. Providers carry no runtime value; they are type-level markers named in a
[`delegate_components!`](../macros/delegate_components.md) table and resolved at compile time. This page
groups the providers CGP ships by the job they do, in roughly the order most CGP code reaches for them.

If you are new to CGP, start with the [reference overview](/docs/reference/) rather than here; this page
is the fuller map for once the essentials are familiar.

## Wiring an everyday component

[`UseContext`](./use_context.md) implements a provider trait by routing back through the context's own
consumer-trait implementation, which is what a higher-order provider falls back to and the default
per-variant provider a dispatch combinator routes each matched case through.
[`UseType`](./use_type.md) supplies a concrete type for an abstract-type component, so a context binds
its error type or runtime with one wiring line.
[`UseField`](./use_field.md) implements a getter by reading a context field named by a tag, and
[`UseFields`](./use_fields.md) does the same when every getter method reads a same-named field.
[`UseDefault`](./use_default.md) selects a component's own default method bodies when there is no
behaviour left for a provider to supply.

[`WithProvider`](./with_provider.md) is the adapter behind the `With…` aliases: it turns a foundational
provider such as a field getter or a type provider into a provider for a named component, and the
aliases `WithField`, `WithType`, and `WithContext` are how it usually appears in wiring.

## Dispatching per type, and organizing wiring

[`UseDelegate`](./use_delegate.md) chooses a provider by the type of a generic parameter through a
lookup table, the legacy form of the per-type dispatch the `open` statement now expresses.
[`RedirectLookup`](./redirect_lookup.md) routes a component's lookup along a type-level path in a
separate table, the mechanism every namespace and `open` statement rides. Both are read in wiring far
more often than written.

## The specialized getters and type providers

[`UseFieldRef`](./use_field_ref.md) is the foundational getter that borrows a field through `AsRef`, for
a getter that returns a view of a field the context stores as a different type.
[`ChainGetters`](./chain_getters.md) composes a list of getters to reach a field several hops inside a
nested context.
[`UseDelegatedType`](./use_delegated_type.md) resolves an abstract type through a table rather than
fixing it to one concrete type. Each is reached through [`WithProvider`](./with_provider.md) when wired
to a component.

## The computation-family combinators

The handler family has its own providers, which build, route, and lift computations rather than wire a
single capability. The [error providers](./error/index.md) raise and wrap a context's abstract
error, the [handler combinators](./handler/index.md) compose and promote handlers across the
family's synchronous, async, and fallible shapes, the [dispatch combinators](./dispatch/index.md)
route an extensible-data value to per-field or per-variant handlers, and the
[monad providers](./monad/index.md) chain handlers that short-circuit through a monad. Each group is a
subsection here with one page per provider and its own overview.
