---
sidebar_label: 'IsProviderFor'
---

# `IsProviderFor`

The marker supertrait that makes a provider's missing dependency show up by name.

:::info

### Generated machinery

**You never write `IsProviderFor` yourself.** The provider macros —
[`#[cgp_impl]`](../macros/cgp_impl.md) and [`#[cgp_provider]`](../macros/cgp_provider.md), which also
covers `#[cgp_new_provider]` — derive it from the same `where` clause you already wrote. This page explains what they emit, because this trait is what a missing-dependency error
names, and reading that error is the reason to know it exists.

:::

## What it's for

An implementation in CGP states what it needs from its **context** — the type the capability runs against —
in its own `where` clause: a field, an abstract type, another capability. When one of those requirements is
not met, you want the compiler to say *which*. Left to itself, it will not.

The reason is a specific behaviour of Rust's diagnostics. Asking "does this provider implement the provider
trait?" has two candidate answers in scope — the blanket impl that routes through the wiring table, and the
implementation you wrote — and when more than one impl could apply, Rust reports only that none matched and
withholds the per-candidate reasoning, because it cannot know which one you meant. The real cause, often a
single absent field, never appears.

`IsProviderFor` is a second path to the same question that Rust *will* explain. It is an empty marker trait,
and the macros implement it for a provider under **exactly the same** `where` bounds as the provider trait
itself. Every generated provider trait carries it as a supertrait, so asking whether a provider satisfies
`IsProviderFor` forces the compiler to evaluate those bounds — and because that question has only one
candidate impl, no blanket competing with it, the compiler commits to it and reports precisely which
constraint failed.

**This is invisible plumbing until you read an error.** You never write it and never call it. What it buys is
the difference between "the trait is not implemented" and "the field `name` is missing on `App`", and it is
the mechanism [`check_components!`](../macros/check_components.md) leans on to produce the second.

## Using it

The trait is in the prelude, so `use cgp::prelude::*;` names it. It is empty, with three parameters:

```rust
pub trait IsProviderFor<Component, Context, Params: ?Sized = ()> {}
```

The four types together identify one provider-trait implementation. `Self` is the **provider** whose validity
is asserted. `Component` is the component-name marker, the same key the wiring table uses. `Context` is the
context the provider trait is implemented for. `Params` collects the component's own extra generic
parameters.

### How `Params` is filled

This is the only part with a rule worth memorizing, and it is what a hand-written assertion gets wrong.

- A component with **no** parameters uses the default, `()`.
- A component with **one** parameter passes it directly — `IsProviderFor<AreaCalculatorComponent, App, Rectangle>`.
- A component with **several** groups them into a tuple — `IsProviderFor<FooComponent, App, (I, J)>`.
- A **lifetime** parameter is lifted into [`Life<'a>`](../types/life.md), because the tuple holds types and a
  lifetime cannot sit there directly.

### Asserting it

You do not implement the trait; you occasionally *assert* it, which is a `where` bound on an empty function:

```rust
fn assert_provider()
where
    GreetHello: IsProviderFor<GreeterComponent, App, ()>,
{}
```

That holds exactly when `GreetHello`'s dependencies are satisfied for `App`. In practice you write this
through the `#[check_providers(...)]` form of [`check_components!`](../macros/check_components.md) rather
than by hand — which is the tool for localizing a failure inside a stack of
[higher-order providers](/docs/concepts/higher-order-providers), where checking the context tells you a layer
is broken but not which one.

## Examples

The trait is something you observe. A provider that needs a field gets a matching marker impl automatically:

```rust
use cgp::prelude::*;

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
```

`#[cgp_impl]` emits the `Greeter` impl *and* an `IsProviderFor<GreeterComponent, Context, ()>` impl for
`GreetHello`, both guarded by the same `Self: HasName` bound. Now wire it onto a context that cannot satisfy
that bound:

```rust
#[derive(HasField)]
pub struct App {
    pub first_name: String, // not `name`
}

delegate_components! {
    App { GreeterComponent: GreetHello }
}

check_components! {
    App { GreeterComponent }
}
```

The check forces `App: CanUseComponent<GreeterComponent, ()>`, which routes through
`GreetHello: IsProviderFor<GreeterComponent, App, ()>`, whose `where` clause names the `HasName`
requirement — so the error names the absent field, at the wiring site. Remove the `check_components!` block
and the same mistake surfaces far away, at whatever calls `app.greet()`, as a much worse message.

## When to reach for it, and when not

**Do not reach for it. Recognize it.** There is no case in ordinary CGP code where you write
`IsProviderFor`: the provider macros emit the impl, `#[cgp_component]` emits the supertrait link, and
[`delegate_components!`](../macros/delegate_components.md) emits the forwarding. What you do with it is read
it in an error and, occasionally, assert it through a check.

Two situations do put its name in your hands, and both are about diagnosis rather than construction.

- **Localizing a broken layer in a higher-order provider stack.** Checking the context proves *something*
  in the stack is unsatisfied. `#[check_providers(...)]` asserts `IsProviderFor` on each named provider
  instead, so a dependency missing only from the outer wrapper errors on its line alone while one missing
  from the inner provider errors on both — which pins the layer.
- **Reading a `[CGP-Exxx]` diagnostic.** `cargo cgp check` names this trait in the dependency chain it
  renders, and knowing that a frame means "this provider's requirements, for this context" is what makes
  the chain readable.

If you find yourself wanting to implement it by hand, the thing you actually want is
[`#[cgp_provider]`](../macros/cgp_provider.md) or [`#[cgp_impl]`](../macros/cgp_impl.md) on the impl block —
the trait's own diagnostic says so, which is the one case where a missing marker impl is the reported
problem rather than the hidden one.

## Under the hood

:::note

### Advanced

This section shows where the three generated pieces come from. You do not need it to write CGP, but every
non-trivial wiring error names this trait, and the three-part chain is what those errors are walking.
`cargo cgp expand` prints all three for your own code.

:::

`IsProviderFor` appears in three places, and together they form the chain an error travels along.

**First, the supertrait link.** [`#[cgp_component]`](../macros/cgp_component.md) emits every provider trait
with the marker as a supertrait, so for a component `CanGetFooAt<I, J>` the provider trait reads:

```rust
pub trait FooGetterAt<Context, I, J>:
    IsProviderFor<FooGetterAtComponent, Context, (I, J)>
{ /* … */ }
```

This is what binds a provider's dependency set to the marker: using the provider trait *requires*
establishing `IsProviderFor` first, so probing the marker is equivalent to probing the provider trait.

**Second, the per-provider impl.** [`#[cgp_provider]`](../macros/cgp_provider.md) and
[`#[cgp_impl]`](../macros/cgp_impl.md) emit an empty impl beside the provider trait impl, carrying identical
bounds:

```rust
impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName,
{ /* … */ }

impl<Context> IsProviderFor<GreeterComponent, Context, ()> for GreetHello
where
    Context: HasName,
{}
```

Identical `where` clauses mean the marker is satisfiable exactly when the provider trait is — which is the
whole trick.

**Third, the table forwarding.** [`delegate_components!`](../macros/delegate_components.md) emits, for each
entry, an impl on the table that forwards to the delegated provider's marker:

```rust
impl<Context, Params> IsProviderFor<GreeterComponent, Context, Params> for AppComponents
where
    GreetHello: IsProviderFor<GreeterComponent, Context, Params>,
{}
```

Because this is an explicit impl rather than a blanket, the compiler follows it and surfaces everything
`GreetHello` requires. It is also what carries requirements *across layers*: when a provider bundle delegates
to another bundle, each forwards to the next, so a dependency unmet several tables deep still propagates back
to where the component is checked.

One refinement is worth knowing, because it is what makes `#[check_providers]` able to localize a layer: the
clause the provider macros derive is **augmented rather than copied**. A bound naming the component's own
provider trait gains its marker counterpart, which is the mechanism that carries an inner provider's
requirements outward through a higher-order stack.

## Gotchas

**`Params` grouping is the usual mistake in a hand-written assertion.** One parameter goes in directly, two
or more as a tuple, none as `()`. Getting it wrong reports as an unsatisfied bound on a trait you did not
expect, rather than as a helpful arity error.

**A lifetime parameter becomes [`Life<'a>`](../types/life.md) in the tuple.** It cannot appear as a bare
lifetime in a type position, so it is lifted.

**A missing marker impl usually means a missing attribute.** If the compiler says a provider does not
implement `IsProviderFor`, the likeliest cause is a provider-trait impl written without
[`#[cgp_provider]`](../macros/cgp_provider.md) — the trait's own diagnostic note says exactly this.

**Read it as "because", not as "instead".** `GreetHello: IsProviderFor<…> is not satisfied` does not mean the
marker is the problem; it means the provider trait is not implemented *because* the named dependency is
missing. The useful part of the message is the constraint underneath.

**It is empty, so satisfying it costs nothing.** There is no method, no data, and nothing in the binary. The
only effect is the compile-time obligation its `where` clause imposes.

**A higher-order provider with a lifetime loses the propagation.** The inner-provider bound of such a stack
gets no marker counterpart when the component carries a lifetime, because the rewrite reads the bound's first
generic argument as the context and finds a lifetime there. The stack still compiles and runs; what is lost
is the propagation that lets `#[check_providers]` localize a broken layer.

## Related constructs

- [`#[cgp_component]`](../macros/cgp_component.md) — attaches the marker as a supertrait on every provider
  trait.
- [`#[cgp_impl]`](../macros/cgp_impl.md) and [`#[cgp_provider]`](../macros/cgp_provider.md) — emit the
  per-provider impl beside the provider trait impl.
- [`delegate_components!`](../macros/delegate_components.md) — emits the forwarding impl at each table entry.
- [`DelegateComponent`](./delegate_component.md) — the table those entries live in.
- [`CanUseComponent`](./can_use_component.md) — the context-indexed counterpart, which requires this of the
  delegate.
- [`check_components!`](../macros/check_components.md) — forces both; its `#[check_providers(...)]` form
  asserts this trait directly.
- [Compile errors](../errors.md) — how the resulting diagnostics read, and what the tool makes of them.

The ideas behind it:

- [Check traits](/docs/concepts/check-traits) — why wiring is lazy and how an assertion makes its failures
  readable.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — the `where`-clause requirements this
  trait re-exposes.

## Source

- Trait: [`is_provider.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/is_provider.rs)
- Supertrait link and per-provider impl: [`cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component)
- Table forwarding: [`delegate_component/mapping/eval.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/delegate_component/mapping/eval.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
