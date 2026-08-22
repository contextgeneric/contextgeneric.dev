---
sidebar_label: 'CanUseComponent'
---

# `CanUseComponent`

The context-side check that a context both wires a component and satisfies its provider.

:::info

### Generated machinery

**You are not expected to name `CanUseComponent` directly.**
[`check_components!`](../macros/check_components.md) and
[`delegate_and_check_components!`](../macros/delegate_and_check_components.md) generate the assertion
that uses it. What you write is the check; this page explains what a check actually asserts, so that its
failure is legible.

:::

## Overview

CGP's wiring is lazy: writing a wiring line does not check that the chosen implementation's own requirements
are met. That check happens the first time the capability is actually called, which is usually somewhere far
from the wiring and produces a bad error when it fails.

`CanUseComponent` is the bound that forces the check early, and it exists because the obvious way of asking
does not work. Asking "does this **context** — the type the capability runs against — implement the consumer
trait?" makes the compiler report the outermost unmet bound, usually a bare "the provider does not implement
the provider trait", and hide the reasoning behind it. The root cause, often one absent field, never appears.

`CanUseComponent` reframes the same question along a path the compiler will explain. It holds when two things
hold together:

- the context **delegates** the component — it has a [`DelegateComponent`](./delegate_component.md) entry;
- the delegated provider **is a valid provider** for that exact context and parameters — it satisfies
  [`IsProviderFor`](./is_provider_for.md).

Because `IsProviderFor` carries the provider's real `where` bounds, requiring it forces the compiler to
evaluate and report them. The error that was hidden becomes visible, at the wiring site.

**You never name this trait.** It is what [`check_components!`](../macros/check_components.md) asserts, and the
reason that macro produces a legible error instead of a cascade.

## Using it

The trait is in the prelude. It is empty, and its entire meaning lives in a single blanket impl:

```rust
pub trait CanUseComponent<Component, Params: ?Sized = ()> {}

impl<Context, Component, Params: ?Sized> CanUseComponent<Component, Params> for Context
where
    Context: DelegateComponent<Component>,
    Context::Delegate: IsProviderFor<Component, Context, Params>,
{
}
```

`Self` is the context being checked, `Component` is the component-name marker, and `Params` collects the
component's extra generic parameters — filled by the same rule
[`IsProviderFor`](./is_provider_for.md) uses: one parameter directly, several as a tuple, `()` when there are
none.

### Asserting it

You assert it through a macro. A [`check_components!`](../macros/check_components.md) table reduces to one
impl of a private check trait per entry, whose supertrait is this bound, so the impl compiles only if the
bound holds:

```rust
check_components! {
    Person {
        GreeterComponent,
    }
}
```

Writing the bound by hand is legal and occasionally useful when probing one case:

```rust
fn assert_wiring()
where
    Person: CanUseComponent<GreeterComponent, ()>,
{}
```

For a component with parameters the check table carries them, which is the same `Params` slot:
`GreeterComponent: Rectangle` for one, `(Rectangle, f64)` for several.

## Examples

The trait's value is what happens when the check fails. Here a provider needs a `name` field and the context
has a differently-named one:

```rust
use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String, // not `name`
}

delegate_components! {
    Person { GreeterComponent: GreetHello }
}

check_components! {
    Person { GreeterComponent }
}
```

The check forces `Person: CanUseComponent<GreeterComponent, ()>`. Resolving it needs
`Person: DelegateComponent<GreeterComponent>`, which the wiring supplies, and
`GreetHello: IsProviderFor<GreeterComponent, Person, ()>`, which fails because `GreetHello` needs `HasName`
and `Person` has no `name` field. The compiler reports the absent field **here**, at the check, rather than at
some later `person.greet()`.

Rename the field to `name` and the block compiles and produces nothing — a passing check is a successful
build, with nothing added to the binary.

## When to reach for it, and when not

**Reach for [`check_components!`](../macros/check_components.md); it is what reaches for this trait.** The one
non-negotiable is that a context's wiring gets checked *somehow* — lazy wiring means an unchecked context can
compile and fail later. Which macro does it is a matter of scale.

- **[`delegate_and_check_components!`](../macros/delegate_and_check_components.md)** fuses wiring and checking,
  which suits basic wiring and getting started. It derives a check only for plain `Component: Provider`
  entries.
- **Separate [`check_components!`](../macros/check_components.md)** is what larger wiring wants, because it
  gives control over *what* is checked: concrete parameters for a generic component, checks over opened or
  namespaced wiring, and `#[check_providers(...)]` per provider layer.
- **`#[check_providers(...)]`** switches the assertion from this trait to
  [`IsProviderFor`](./is_provider_for.md) on named providers. Reach for it when a
  [higher-order provider](/docs/concepts/higher-order-providers) stack fails and you need to know *which
  layer*, which a context-side check cannot tell you.

One case makes a context-side check actively wrong rather than merely unnecessary: **a provider bundle.** A
[`delegate_components!`](../macros/delegate_components.md) block with a leading `new` declares a provider that
other contexts delegate to, not a context. Asserting `CanUseComponent` on it asks whether the bundle can use
each component *as a context* — a role it never plays — so the answer is uninformative either way. It passes
vacuously when the bundled providers need nothing from their context, and fails blaming the bundle when any of
them needs something the real context would have supplied. Wire a bundle with plain
`delegate_components!` and let a real context's check verify it, or assert `IsProviderFor` on it directly.

Finally, not every unmet bound is a component. Some are ordinary or blanket traits, and no check macro can
verify those — they surface as themselves.

## Under the hood

:::note

### Advanced

This section shows how the two bounds distinguish the two ways wiring goes wrong. You do not need it to write
a check, but it is what tells you whether to add a wiring line or supply a dependency.

:::

The blanket impl *is* the behaviour, and its two bounds fail differently — which is the practical payoff.

**The first bound failing means the wiring is absent.** `Context: DelegateComponent<Component>` is unmet when
the context never wired the component, and the [`DelegateComponent`](./delegate_component.md) diagnostic
reports a missing table entry. The fix is a wiring line.

**The second bound failing means the wiring is incomplete.** `Context::Delegate: IsProviderFor<…>` is unmet
when the component *is* wired but the chosen provider's own requirements are not satisfied, and the error is
that provider's unsatisfied `where` clause carried up through the marker. The fix is to supply the dependency.

Reading which bound failed is therefore the first thing to do with the error. The trait is the mirror image of
[`IsProviderFor`](./is_provider_for.md): the same readability-restoring question, indexed on the context
(`Self = Context`) rather than on the provider (`Self = Provider`). One starts from "the context delegates and
its delegate is valid"; the other from "this specific provider is valid".

[`check_components!`](../macros/check_components.md) consumes it by emitting a private check trait whose
supertrait is this bound, then one empty impl of that trait per checked entry. Each impl compiles only if its
supertrait holds, so the table is an assertion. Because it produces no values, a successful build *is* the
passing test.

## Gotchas

**It is not the consumer trait.** `Person: CanUseComponent<GreeterComponent, ()>` holding is not the same
statement as `Person: CanGreet`, even though in practice one follows the other. The check exists precisely
because the two questions produce different diagnostics.

**Do not assert it on a provider bundle.** The question is meaningless there, and worse, it can pass
vacuously — so a green check on a bundle proves nothing. This is why
[`delegate_and_check_components!`](../macros/delegate_and_check_components.md) is wrong for a bundle.

**`Params` follows the tuple rule.** One parameter directly, several as a tuple, `()` for none. A hand-written
assertion that gets this wrong reports as an unsatisfied bound rather than an arity error.

**A passing check is not a passing test of behaviour.** It proves the wiring resolves and the dependencies
exist. It says nothing about whether the provider does the right thing.

**Checking costs compile time and nothing else.** The trait is empty and the impls are empty; nothing reaches
the binary. The cost is trait resolution at build time, which is the trade the check exists to make.

## Related constructs

- [`check_components!`](../macros/check_components.md) — the macro that asserts this trait; what you write.
- [`delegate_and_check_components!`](../macros/delegate_and_check_components.md) — wires and checks at once,
  for basic wiring.
- [`DelegateComponent`](./delegate_component.md) — the first of the two bounds: the context must delegate.
- [`IsProviderFor`](./is_provider_for.md) — the second bound, and the provider-indexed counterpart of this
  trait.
- [`#[cgp_component]`](../macros/cgp_component.md) — defines the components being checked.
- [Compile errors](../errors.md) — the shapes these checks produce, and how to read them.

The ideas behind it:

- [Check traits](/docs/concepts/check-traits) — why wiring is lazy, and what a compile-time assertion buys.
- [Higher-order providers](/docs/concepts/higher-order-providers) — the case where a context-side check is not
  enough.

## Source

- Trait and its blanket impl: [`can_use_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/can_use_component.rs)
- The checks that assert it: [`check_components/table.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/check_components/table.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
