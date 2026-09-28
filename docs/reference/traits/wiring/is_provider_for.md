---
title: 'IsProviderFor — name a missing dependency'
description: 'The empty marker every provider trait requires, implemented under the provider''s own where clause so a check can name the dependency a provider is missing.'
sidebar_label: 'IsProviderFor'
sidebar_position: 2
---

# `IsProviderFor`

The marker [supertrait](/docs/reference/glossary#supertrait) that makes a provider's missing
dependency show up by name.

:::info

### Generated machinery

**You are not expected to implement `IsProviderFor`.** The provider macros,
[`#[cgp_impl]`](../../macros/cgp_impl.md) and [`#[cgp_provider]`](../../macros/cgp_provider.md)
(which also covers `#[cgp_new_provider]`), emit its impl from the `where` clause you already wrote,
and [`delegate_components!`](../../macros/delegate_components.md) forwards it through each table
entry. The one place you name it is an assertion, usually through the `#[check_providers(...)]` form
of [`check_components!`](../../macros/check_components.md). This page explains what the macros emit,
because this trait is what a missing-dependency error names.

:::

## Overview

A provider states what it needs from its [**context**](/docs/reference/glossary#context), the type
the implementation runs against, in its own `where` clause: a field, an
[abstract type](/docs/reference/glossary#abstract-type), another trait. When one of those
requirements is not met, you want the compiler to say which, and left to itself it will not. Asking
whether a provider implements its provider trait has two candidate impls in scope, the blanket impl
that routes through the wiring table and the one you wrote, and when more than one impl could apply,
rustc reports only that none matched.

`IsProviderFor` is a second path to the same question, with one candidate. It is an empty marker
trait, and the provider macros implement it for a provider under exactly the provider trait impl's
`where` bounds. With no competing impl, the compiler commits to that one and reports which bound
failed. So the trait buys the difference between "the trait is not implemented" and "`Person` has no
`name` field", and it is what [`check_components!`](../../macros/check_components.md) relies on to
report the second.

## Definition

`IsProviderFor` is an empty trait with three type parameters:

```rust
pub trait IsProviderFor<Component, Context, Params: ?Sized = ()> {}
```

The four types together identify one provider trait impl:

- **`Self`** is the provider whose validity is asserted.
- **`Component`** is the component's marker, the same key the wiring table uses.
- **`Context`** is the context the provider trait is implemented for.
- **`Params`** holds the component's own generic parameters, and defaults to `()` for a component
  with none; [Usage](#how-params-is-filled) gives the rule.

[`#[cgp_component]`](../../macros/cgp_component.md) makes it a supertrait of every provider trait,
so using a provider trait requires `IsProviderFor` first. Probing the marker therefore probes the
provider trait's whole dependency set.

## Usage

The trait is in the prelude, so `use cgp::prelude::*;` names it.

### How `Params` is filled

The component's generic parameters go into `Params` by one rule:

- A component with **no** parameters uses the default, `()`.
- A component with **one** parameter passes it directly, as in
  `IsProviderFor<AreaCalculatorComponent, App, Rectangle>`.
- A component with **several** groups them into a tuple, as in
  `IsProviderFor<FooComponent, App, (I, J)>`.
- A **lifetime** parameter is lifted into [`Life<'a>`](../../types/life.md), since the tuple holds
  types.

The same rule fills the `Params` of [`CanUseComponent`](./can_use_component.md) and the parameter
list of a [`check_components!`](../../macros/check_components.md) entry.

### Asserting it

You assert the trait rather than implement it, as a `where` bound on an empty function:

```rust
pub fn assert_provider()
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, App, Rectangle>,
{
}
```

The bound holds exactly when `RectangleArea`'s dependencies are met for `App`, whether or not `App`
wires it. The `#[check_providers(...)]` form of `check_components!` writes such assertions for you,
which is how a failure inside a stack of
[higher-order providers](/docs/concepts/higher-order-providers) is pinned to one layer.

## Examples

A provider with a dependency, a context that meets it, and the marker asserted both through the
context and on the provider itself:

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

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

check_components! {
    #[check_trait(CheckGreetHello)]
    #[check_providers(GreetHello)]
    Person {
        GreeterComponent,
    }
}

pub fn demo() {
    let person = Person {
        name: "Ada".to_owned(),
    };

    assert_eq!(person.greet(), "Hello, Ada!");
}
```

The first check asserts it through `Person`'s wiring, and the second on `GreetHello` directly, as
`GreetHello: IsProviderFor<GreeterComponent, Person, ()>`; the second needs its own trait name,
since two checks in one module would otherwise both declare `__CheckPerson`. Both hold because
`Person` has a `name` field. `Person` is a
[value context](/docs/reference/glossary#value-context): the greeting reads its own field.

## When to use it

**Recognize it; do not write it.** The provider macros emit the impl, `#[cgp_component]` the
supertrait link, and `delegate_components!` the forwarding, so ordinary CGP code never implements
it. Two situations put its name in your hands, and both are about diagnosis:

- **Localizing a broken layer in a [higher-order
  provider](/docs/reference/glossary#higher-order-provider) stack.** Checking the context proves
  that something in the stack is unsatisfied; `#[check_providers(...)]` asserts `IsProviderFor` on
  each named provider instead, so a dependency missing only from the outer wrapper errors on its
  line alone, while one missing from the inner provider errors on both.
- **Reading a diagnostic.** A compiler note or a [`cargo cgp check`](/docs/cargo-cgp/check) chain
  that names `IsProviderFor<C, Ctx, P>` for a provider means "this provider's requirements, for this
  context", and the frame below it names the requirement.

If you want to implement it by hand, what you want is
[`#[cgp_provider]`](../../macros/cgp_provider.md) or [`#[cgp_impl]`](../../macros/cgp_impl.md) on
the impl block.

## Under the hood

`IsProviderFor` appears in three places in an expansion, and together they form the chain an error
travels along. `cargo cgp expand` on the example shows the second and third.

**The supertrait link.** [`#[cgp_component]`](../../macros/cgp_component.md) gives every provider
trait the marker as a supertrait. For the `AreaCalculator` component of [Usage](#usage), whose
consumer trait is `CanCalculateArea<Shape>`, the provider trait begins:

```rust
pub trait AreaCalculator<
    __Context__,
    Shape,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Shape)> {
```

`(Shape)` is the one parameter in parentheses, not a tuple, which is the Params rule at work.

**The per-provider impl.** [`#[cgp_impl]`](../../macros/cgp_impl.md) emits, beside the provider
trait impl, an empty marker impl under the same bound:

```rust
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", __context__.name()))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasName,
{}
```

Identical `where` clauses make the marker satisfiable exactly when the provider trait is. The clause
is augmented rather than copied in one case: a bound naming a component's provider trait, as a
higher-order provider's inner-provider bound does, gains its marker counterpart, which carries an
inner provider's requirements outward through the stack.

**The table forwarding.** [`delegate_components!`](../../macros/delegate_components.md) emits, for
each entry, an impl on the table that forwards to the chosen provider's marker:

```rust
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
```

Because this is an impl on a specific table rather than a blanket, the compiler follows it and
surfaces everything `GreetHello` requires. When a context delegates to a provider bundle that
delegates further, each table forwards to the next, so a requirement unmet several tables deep still
reaches the check.

## Common Mistakes

**Read the error as "because", not "instead".** Give the example's `Person` a differently named
field, and its check fails:

```rust
#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}
```

The notes name the marker as the step between the check and the real cause, down to the attribute
that introduced the bound:

```text
error[E0277]: the trait bound `Person: CanUseComponent<GreeterComponent>` is not satisfied
...
note: required for `Person` to implement `HasName`
...
note: required for `GreetHello` to implement `IsProviderFor<GreeterComponent, Person>`
...
   |        ------- unsatisfied trait bound introduced here
```

The marker is not the problem; the missing `name` field, which the error's help line names as an
unimplemented `HasField`, is.

**One parameter is not a one-element tuple.** Writing the Usage assertion with `(Rectangle,)`,
as `RectangleArea: IsProviderFor<AreaCalculatorComponent, App, (Rectangle,)>`, asserts a different
bound, which rustc reports with the mismatch:

```text
error[E0277]: the trait bound `RectangleArea: cgp::prelude::IsProviderFor<AreaCalculatorComponent, App, (Rectangle,)>` is not satisfied
...
   = help: for that trait implementation, expected `Rectangle`, found `(Rectangle,)`
```

**A missing marker impl means a missing attribute.** A provider trait impl written without
[`#[cgp_provider]`](../../macros/cgp_provider.md) gets no marker impl, so it fails its own
supertrait:

```rust
pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "Hello!".to_owned()
    }
}
```

```text
error[E0277]: the trait bound `GreetHello: IsProviderFor<GreeterComponent, Context>` is not satisfied
```

**A higher-order provider over a component with a lifetime loses the propagation.** The rewrite
reads the inner-provider bound's first generic argument as the context and finds a lifetime there,
so the bound gets no marker counterpart. The stack still compiles and runs; what is lost is the
propagation that lets `#[check_providers]` localize a broken layer.

## Related constructs

- [`#[cgp_component]`](../../macros/cgp_component.md): makes the marker a supertrait of every
  provider trait.
- [`#[cgp_impl]`](../../macros/cgp_impl.md) and [`#[cgp_provider]`](../../macros/cgp_provider.md):
  emit the per-provider impl.
- [`delegate_components!`](../../macros/delegate_components.md): emits the forwarding impl at each
  table entry.
- [`CanUseComponent`](./can_use_component.md): the context-side counterpart, which requires this of
  the delegate.
- [`check_components!`](../../macros/check_components.md): asserts it, directly with
  `#[check_providers(...)]`.
- [Compile errors](../../errors.md): how the resulting diagnostics read.

The ideas behind it:

- [Check traits](/docs/concepts/check-traits): why wiring is lazy and how an assertion makes its
  failures readable.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the `where`-clause requirements
  this trait re-exposes.

## Source

- [`traits/is_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/is_provider.rs)
- The supertrait link and the per-provider impl, in
  [`cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component),
  and the table forwarding, in
  [`delegate_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
