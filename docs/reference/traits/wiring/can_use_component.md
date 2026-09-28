---
title: 'CanUseComponent — the wiring check'
description: 'The check a check_components! entry asserts: the context wires the component, and the provider it chose has every dependency met for that context.'
sidebar_label: 'CanUseComponent'
sidebar_position: 3
---

# `CanUseComponent`

The context-side check that a context both wires a component and satisfies its provider.

:::info

### Generated machinery

**You are not expected to name `CanUseComponent` directly.**
[`check_components!`](../../macros/check_components.md) and
[`delegate_and_check_components!`](../../macros/delegate_and_check_components.md) generate the
assertion that uses it. What you write is the check; this page explains what a check asserts, so
that its failure is legible. A hand-written bound on it is legal, and occasionally useful when
probing one case.

:::

## Overview

CGP's wiring is lazy: writing a wiring line does not check that the chosen provider's own
requirements are met. That check happens where the trait is first used, usually far from the wiring,
and the error there is poor, because asking whether a
[**context**](/docs/reference/glossary#context), the type the implementation runs against,
implements the consumer trait makes the compiler report only the outermost unmet bound. The root
cause, often one absent field, does not appear.

`CanUseComponent` asks the same question along a path the compiler explains. It holds when two
things hold together: the context **wires** the component, with a
[`DelegateComponent`](./delegate_component.md) entry, and the provider it chose **is valid** for
that context and those parameters, through [`IsProviderFor`](./is_provider_for.md). Because
`IsProviderFor` carries the provider's real `where` bounds, requiring it makes the compiler evaluate
and report them, at the check.

## Definition

`CanUseComponent` is an empty trait whose meaning lives in one blanket impl:

```rust
pub trait CanUseComponent<Component, Params: ?Sized = ()> {}

impl<Context, Component, Params: ?Sized> CanUseComponent<Component, Params> for Context
where
    Context: DelegateComponent<Component>,
    Context::Delegate: IsProviderFor<Component, Context, Params>,
{
}
```

`Self` is the context being checked. `Component` is the component's marker, the key the wiring table
uses. `Params` holds the component's generic parameters, filled by the rule
[`IsProviderFor`](./is_provider_for.md#how-params-is-filled) uses: `()` for none, one directly,
several as a tuple. The two bounds are the two halves of "can use": an entry, and a valid provider
behind it.

## Usage

The trait is in the prelude. A [`check_components!`](../../macros/check_components.md) entry asserts
it, carrying the component's parameters after a colon:

```rust
check_components! {
    App {
        GreeterComponent,
        AreaCalculatorComponent: Rectangle,
    }
}
```

The same bounds written by hand hold exactly when that check passes:

```rust
pub fn assert_wiring()
where
    App:
        CanUseComponent<GreeterComponent> + CanUseComponent<AreaCalculatorComponent, Rectangle>,
{
}
```

## Examples

Two components checked on one context, one without parameters and one with a shape parameter:

```rust
use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator<Rectangle> {
    fn area(&self, rectangle: &Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetHello,
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    App {
        GreeterComponent,
        AreaCalculatorComponent: Rectangle,
    }
}

pub fn assert_wiring()
where
    App:
        CanUseComponent<GreeterComponent> + CanUseComponent<AreaCalculatorComponent, Rectangle>,
{
}

pub fn demo() {
    assert_wiring();

    let app = App {
        name: "Ada".to_owned(),
    };
    let rectangle = Rectangle {
        width: 3.0,
        height: 4.0,
    };

    assert_eq!(app.greet(), "Hello, Ada!");
    assert_eq!(app.area(&rectangle), 12.0);
}
```

The `Greeter` check holds because `App` has the `name` field `GreetHello` reads, and the
`AreaCalculator` check holds at `Rectangle`, the parameter it names. A passing check produces
nothing; the build succeeding is the result. Rename the field, and the `Greeter` check fails with
the chain shown on [`IsProviderFor`](./is_provider_for.md#common-mistakes). `App` is an
[environmental context](/docs/reference/glossary#environmental-context) that also holds the name its
greeting reads.

## When to use it

**Reach for [`check_components!`](../../macros/check_components.md), which reaches for this trait.**
What is not optional is that every context's wiring gets checked somehow, since lazy wiring lets an
unchecked context compile and fail later. Which macro does it is a matter of scale:

- **[`delegate_and_check_components!`](../../macros/delegate_and_check_components.md)** wires and
  checks in one block, which suits basic wiring and a starter context. It checks each entry keyed on
  a component name, including `->` entries, and wires path keys, `=>` redirects, and statements
  without checking them.
- **A separate [`check_components!`](../../macros/check_components.md)** gives control over what is
  checked: parameters for a generic component, opened or namespaced wiring, and
  `#[check_providers(...)]` per provider layer.
- **`#[check_providers(...)]`** switches the assertion from this trait to
  [`IsProviderFor`](./is_provider_for.md) on named providers, which is what localizes a failure in a
  stack of [higher-order providers](/docs/concepts/higher-order-providers).

**Do not assert it on a provider bundle.** A `delegate_components!` block with a leading `new`
declares a provider that contexts delegate to, not a context, so asserting `CanUseComponent` on it
asks a question it never answers in use. It passes vacuously when the bundled providers need nothing
from their context, and fails blaming the bundle when one needs something the real context would
supply. Check the bundle through a real context that delegates to it, or assert `IsProviderFor` on
it directly.

Not every unmet bound is a component. Ordinary and blanket traits that a provider requires surface
as themselves, and no check macro verifies them.

## Under the hood

[`check_components!`](../../macros/check_components.md) emits a private check trait whose supertrait
is this bound, then one empty impl of it per entry. `cargo cgp expand` on the example shows:

```rust
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
impl __CheckApp<AreaCalculatorComponent, Rectangle> for App {}
```

Each impl compiles only if its supertrait holds, so the table is an assertion, and it adds nothing
to the binary. The two bounds of the blanket impl then fail differently, which tells you what to
fix:

- **The entry bound failing means the wiring is absent.** The check's help line names the
  unimplemented `DelegateComponent<Component>` for the context, as
  [`DelegateComponent`](./delegate_component.md#common-mistakes) shows, and the fix is a wiring
  line.
- **The provider bound failing means the wiring is incomplete.** The component is wired, but the
  chosen provider's requirements are not met, and the notes carry the provider's unsatisfied bound
  up through `IsProviderFor`, as [`IsProviderFor`](./is_provider_for.md#common-mistakes) shows. The
  fix is to supply the dependency.

The trait is the mirror image of `IsProviderFor`: the same question, indexed on the context
(`Self = Context`) rather than on the provider (`Self = Provider`).

## Common Mistakes

**It is not the consumer trait.** `App: CanUseComponent<GreeterComponent>` holding is a different
statement from `App: CanGreet`, though in practice one follows the other. The check exists because
the two questions produce different diagnostics.

**A green check on a provider bundle proves nothing.** It can pass vacuously, as [When to use
it](#when-to-use-it) explains, which is why `delegate_and_check_components!` is the wrong macro for
a bundle.

**A passing check does not test behavior.** It proves the wiring resolves and every dependency
exists, not that the provider computes the right thing. The trait asks the delegate's
`IsProviderFor` impl rather than the context's own, so a component used at an unsized type parameter
passes it and still fails at the call; see
[`delegate_components!`](../../macros/delegate_components.md#common-mistakes).

## Related constructs

- [`check_components!`](../../macros/check_components.md): the macro that asserts this trait; what
  you write.
- [`delegate_and_check_components!`](../../macros/delegate_and_check_components.md): wires and
  checks at once, for basic wiring.
- [`DelegateComponent`](./delegate_component.md): the first bound, the entry.
- [`IsProviderFor`](./is_provider_for.md): the second bound, and the provider-side counterpart.
- [Compile errors](../../errors.md): the shapes these checks produce, and how to read them.

The ideas behind it:

- [Check traits](/docs/concepts/check-traits): why wiring is lazy, and what a compile-time
  assertion buys.
- [Higher-order providers](/docs/concepts/higher-order-providers): the case where a context-side
  check is not enough.

## Source

- [`traits/can_use_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/can_use_component.rs)
- The checks that assert it, in
  [`check_components/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/check_components).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
