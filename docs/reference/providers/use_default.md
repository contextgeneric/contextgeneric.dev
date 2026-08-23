---
sidebar_label: 'UseDefault'
sidebar_position: 5
---

# `UseDefault`

Select a component's implementation from the consumer trait's own default method bodies.

## Overview

`UseDefault` names the "use the trait's own defaults" choice in wiring. A consumer trait may define
default bodies for its methods, the same way any Rust trait can. When every method of a component has a
usable default, no behavior is left for a provider to supply, and the provider only needs to exist so
the component can be wired. `UseDefault` is that empty provider: wiring a component to `UseDefault`
selects an implementation whose method bodies come from the trait's defaults rather than from a
dedicated provider.

This keeps a default-only component consistent with the rest of wiring. Without it, a component whose
methods are all defaulted would still need some provider type and some
[`delegate_components!`](../macros/delegate_components.md) entry to take part in the table. `UseDefault`
is the shared name for that role, so a **context** (the type the capability runs against) that wants
the defaults wires the component to `UseDefault` and writes no method bodies of its own.

`UseDefault` is a bare marker that CGP defines but does not implement for any trait. Unlike
[`UseContext`](use_context.md) or the getter providers, no macro generates a provider implementation
for it. The author writes the implementation, usually with [`#[cgp_impl]`](../macros/cgp_impl.md) and
an empty body so the trait's defaults take effect. That distinguishes `UseDefault` from the
providers that carry generated behavior: it is purely a conventional name for an author-supplied,
default-bodied implementation. Like every CGP provider, it carries no runtime value.

## Usage

`UseDefault` is not in the prelude. Import it from `cgp::core::component`:

```rust
use cgp::core::component::UseDefault;
```

Giving `UseDefault` a behavior takes two steps. First, write a provider implementation against it with
an empty body, so the trait's defaults fill in:

```rust
#[cgp_impl(UseDefault)]
impl Greeter {}
```

Then wire the component to it, exactly like any other provider:

```rust
delegate_components! {
    App {
        GreeterComponent: UseDefault,
    }
}
```

Because `UseDefault` has no generated impls, the author controls precisely which components it serves.
A type with no provider impl written for a component is simply not a provider for it; there is no
automatic fallback, and the empty-body `#[cgp_impl]` is the explicit opt-in.

## Examples

The use of `UseDefault` is to wire several default-bodied components to one shared marker, then delegate
them all to it in the context's table. A getter and a greeter both carry default bodies:

```rust
use cgp::prelude::*;
use cgp::core::component::UseDefault;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str {
        "John"
    }
}

#[cgp_component(Greeter)]
pub trait CanGreet: HasName {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

#[cgp_impl(UseDefault)]
impl NameGetter {}

#[cgp_impl(UseDefault)]
#[uses(HasName)]
impl Greeter {}

pub struct App;

delegate_components! {
    App {
        [
            NameGetterComponent,
            GreeterComponent,
        ]:
            UseDefault,
    }
}
```

The first `#[cgp_impl]` makes `UseDefault` a `NameGetter` provider whose `name` is the trait default
`"John"`; the second makes it a `Greeter` provider whose `greet` is the trait default that formats
around `self.name()`. The `Greeter` impl restates its consumer-side dependency with
[`#[uses(HasName)]`](../attributes/uses.md), because the default body of `greet` calls `name`. `App`
then delegates both components to `UseDefault` in one array entry, so `App.greet()` produces
`"Hello, John!"` entirely from the two default bodies, with no method implemented on `App` or on a
dedicated provider.

## When to reach for it, and when not

**Reach for `UseDefault` when a component's methods all have usable defaults and a context wants them
unchanged.** It gives a default-only component a name to wire, so it fits the delegation table like any
other component.

Reach for a named provider written with [`#[cgp_impl]`](../macros/cgp_impl.md) whenever a component has
real behavior to supply, which is the usual case. `UseDefault` is only for the narrow case where the
trait's own defaults are the whole implementation.

## Under the hood

`UseDefault` has no generated implementations. It is a bare unit struct:

```rust
pub struct UseDefault;
```

Meaning is given to it by the provider impl an author writes. An empty
[`#[cgp_impl(UseDefault)]`](../macros/cgp_impl.md) block emits a provider-trait implementation with no
method bodies, so each method falls back to the default defined on the consumer trait, plus the
matching [`IsProviderFor`](../traits/is_provider_for.md) implementation carrying the block's `where`
clause. That is the same pair any `#[cgp_impl]` produces; the only thing special about `UseDefault` is
the empty body and the shared, conventional name.

## Related constructs

- [`#[cgp_impl]`](../macros/cgp_impl.md) — writes the empty-body provider impl that gives `UseDefault`
  its behavior.
- [`delegate_components!`](../macros/delegate_components.md) — wires it, and
  [`check_components!`](../macros/check_components.md) verifies its dependencies.
- [`UseContext`](use_context.md) and [`UseFields`](use_fields.md) — the providers that carry
  macro-generated behavior rather than relying on author-written empty bodies.
- [`IsProviderFor`](../traits/is_provider_for.md) — tracks the dependencies of a `UseDefault` impl.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the split that lets a
  default-bodied consumer trait stand in for a provider's behavior.

## Source

- Struct:
  [`use_default.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_default.rs)
  — the file contains only the bare struct, with no generated impls.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
