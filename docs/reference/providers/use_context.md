---
sidebar_label: 'UseContext'
sidebar_position: 1
---

# `UseContext`

Satisfy a provider trait by routing back through the context's own consumer-trait implementation.

## Overview

`UseContext` turns a context's existing consumer-trait implementation into a provider that other
providers can call. The **context** is the type a capability runs against, and it normally *uses* a
provider through its consumer trait. Sometimes the implementation another provider wants for a
capability is exactly the one the context already supplies that way. `UseContext` is that bridge: it is
a provider whose method bodies call the consumer method on the context, so handing a component
`UseContext` means "use whatever this context already does for this trait."

This makes `UseContext` the exact dual of the consumer-trait blanket implementation that
[`#[cgp_component]`](../macros/cgp_component.md) generates. That blanket impl runs in the
consumer-to-provider direction: a context implements `CanGreet` by delegating to whichever provider
implements `Greeter` for it. `UseContext` runs the other way: it implements the provider trait
`Greeter` by delegating to whatever `CanGreet` implementation the context has. One forwards the
consumer trait to a provider; the other forwards a provider trait back to the consumer trait.

`UseContext` matters most with [higher-order providers](/docs/concepts/higher-order-providers), which
take another provider as a type parameter. A higher-order provider that wraps *one* capability while
delegating to *another* can default its inner-provider parameter to `UseContext`, so the inner step
falls back to whatever the context already wires for that other capability, and the author need not name
an explicit inner provider. Like every CGP provider, `UseContext` carries no runtime value: it is a
unit marker whose `self` position is never read.

## Usage

`UseContext` is in the prelude, so `use cgp::prelude::*;` is enough. It takes no type parameter and
appears in two places. Most often it is a higher-order provider's default inner provider, written in
the provider's struct definition:

```rust
pub struct GreetLoudly<Inner = UseContext>(pub PhantomData<Inner>);
```

It also appears directly in a wiring entry to route a component through the context's own
implementation, which is how the [dispatch combinators](dispatch/index.md) default their
per-variant provider.

**Do not wire a component to `UseContext` when that component's only implementation on the context is
the delegation itself.** Doing so asks the context to implement the consumer trait by delegating to a
provider that implements the provider trait by calling the same consumer trait, a cycle the trait
solver cannot resolve. See [Common Mistakes](#common-mistakes). `UseContext` works when the provider
trait it stands in for resolves to a *different* implementation than the one being wired, which is why
it fits a wrapper that delegates to another capability.

## Examples

The idiomatic use is as the default inner provider of a higher-order provider that builds one capability
on top of another. Here a loud greeter wraps an ordinary greeter, and its inner provider defaults to
`UseContext` so it reuses whatever `CanGreet` the context already wires:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(LoudGreeter)]
pub trait CanGreetLoudly {
    fn greet_loudly(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

pub struct GreetLoudly<Inner = UseContext>(pub PhantomData<Inner>);

#[cgp_impl(GreetLoudly<Inner>)]
#[use_provider(Inner: Greeter)]
impl<Inner> LoudGreeter {
    fn greet_loudly(&self) -> String {
        format!("{}!", Inner::greet(self).to_uppercase())
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
        LoudGreeterComponent: GreetLoudly,
    }
}
```

`App` wires `LoudGreeterComponent` to a bare `GreetLoudly`, whose `Inner` defaults to `UseContext`. When
`app.greet_loudly()` runs, `GreetLoudly` calls `Inner::greet(self)`, which resolves through `UseContext`
to `App`'s own `CanGreet`, wired to `GreetHello`. The result is `"HELLO!"`. Because `GreetLoudly`
implements a *different* component from the one it delegates through, the delegation lands on
`GreetHello` rather than back on `GreetLoudly`, so there is no cycle. Overriding the parameter, as in
`GreetLoudly<GreetHello>`, binds the inner greeter directly and bypasses the context's `CanGreet`
wiring.

`UseContext` acts as a default only when the provider's struct definition gives it as the default
generic parameter, as `GreetLoudly` does. A provider without such a default has no inner provider to
fall back to, and the inner provider must always be named.

## When to reach for it, and when not

**Reach for `UseContext` as the default inner provider of a higher-order provider that delegates to a
different capability than the one it implements**, so the wrapper reuses the context's own wiring for
that other capability when no inner provider is named.

**Do not wire a component to `UseContext` on the context that implements it only through that
delegation.** That is the one shape it cannot serve, because the component's implementation would depend
on itself. When you want a wrapper over the *same* capability, name the inner provider explicitly rather
than defaulting to `UseContext`.

## Under the hood

[`#[cgp_component]`](../macros/cgp_component.md) emits a `UseContext` implementation of the provider
trait for every component it defines, alongside the consumer blanket impl, the provider blanket impl,
the component marker, and the [`RedirectLookup`](redirect_lookup.md) impl. The generated impl bounds the
context on the consumer trait and forwards each method to it. For a component such as

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}
```

the macro generates this `UseContext` impl (shown with the macro's real placeholder identifiers):

```rust
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) {
        __Context__::greet(__context__)
    }
}
```

The provider method takes the context explicitly and calls the context's own `CanGreet::greet`. Each
`UseContext` impl is paired with a matching [`IsProviderFor`](../traits/is_provider_for.md) impl
carrying the same `where` clause, so delegation propagates the dependency and a check reports a missing
consumer-trait implementation precisely. Any supertrait bound on the consumer trait is reproduced in the
`where` clause, so a context must satisfy it before `UseContext` can stand in as a provider.

## Common Mistakes

**Wiring a component to `UseContext` on the context that owns it creates a cycle.** If `App` wires
`GreeterComponent` to `UseContext`, then `App`'s `CanGreet` is implemented by delegating to
`UseContext`, whose `Greeter` impl in turn calls `App`'s `CanGreet`. The trait solver chases this in a
loop and reports it as an overflow or an unsatisfied bound. The same happens with a same-component
wrapper that defaults its inner to `UseContext` and is then wired to that component. `UseContext`
belongs where the provider trait resolves to a different implementation, such as a wrapper over another
capability or a per-type dispatch entry.

## Related constructs

- [`#[cgp_component]`](../macros/cgp_component.md) — generates the `UseContext` impl for every
  component, and the consumer blanket impl `UseContext` is the dual of.
- [Higher-order providers](/docs/concepts/higher-order-providers) — where `UseContext` is the default
  inner provider.
- [`WithProvider`](with_provider.md) — its `WithContext` alias is `WithProvider<UseContext>`, adapting a
  context's own implementation into a component.
- [`RedirectLookup`](redirect_lookup.md) — the other `#[cgp_component]`-generated provider, routing
  through a separate table rather than back to the context.
- [Dispatch combinators](dispatch/index.md) — default their per-variant provider to `UseContext`.
- [`IsProviderFor`](../traits/is_provider_for.md) — carries the dependency `UseContext` propagates.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the two-way split
  `UseContext` bridges.
- [Higher-order providers](/docs/concepts/higher-order-providers) — the pattern that makes it useful.

## Source

- Struct and the `WithContext` alias:
  [`use_context.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_context.rs)
- The generated `UseContext` impl:
  [`cgp_component/evaluated/to_use_context_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_component/evaluated/to_use_context_impl.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
