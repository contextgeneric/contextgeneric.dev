---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Attributes

An attribute here is a modifier, not a macro of its own. Each one is an option a host macro reads,
[`#[cgp_impl]`](../macros/cgp_impl.md), [`#[cgp_fn]`](../macros/cgp_fn.md), or
[`#[cgp_component]`](../macros/cgp_component.md), to refine the trait or implementation that macro
generates. This page groups every attribute in this section by the job it does, in roughly the order
most CGP code reaches for them.

If you are new to CGP, start with the [reference overview](/docs/reference/) rather than here; this page
is the fuller map for once the essentials are familiar.

## Reading a value from the context

[`#[implicit]`](./implicit.md) is the one almost every provider uses, and usually the first piece of CGP
anyone writes. It marks a function argument that the macro reads from a same-named field on the context
rather than from the caller, so a provider that needs a value reads like an ordinary function that takes
it.

## Importing a dependency

These attributes declare what an implementation depends on, each reading like an import. Most
dependencies belong on the implementation, out of sight of callers, and two of the attributes put one on
the generated trait instead, for when every caller must rely on it.

[`#[uses]`](./uses.md) imports a consumer trait, or an ordinary Rust trait, that the context must
satisfy, which is the common case. [`#[use_type]`](./use_type.md) imports an abstract type another
component supplies, such as an error type, and lets the signature name it as a bare word instead of a
qualified path. [`#[use_provider]`](./use_provider.md) imports a provider trait a named inner provider
must satisfy, the dependency a higher-order provider declares.

[`#[extend]`](./extend.md) and [`#[extend_where]`](./extend_where.md) put the requirement on the generated
trait rather than on the implementation, so every caller inherits it: `#[extend]` adds a supertrait, and
`#[extend_where]` adds a `where` predicate a supertrait cannot express.

## Registering a namespace default

[`#[default_impl]`](./default_impl.md) lets a provider register itself as a namespace's default for a
key, at the point where the provider is defined rather than in the namespace's own body. Reach for it
once per-type defaults accumulate.

## Legacy

[`#[derive_delegate]`](./derive_delegate.md) generates the older `UseDelegate` dispatcher for a component
generic over a type parameter. The `open` statement of
[`delegate_components!`](../macros/delegate_components.md) replaced it for new code; the page is here
because you will meet the form in existing wiring and in CGP's own error and handler components.
