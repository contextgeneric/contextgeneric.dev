---
sidebar_label: '#[cgp_auto_dispatch]'
sidebar_position: 15
---

# `#[cgp_auto_dispatch]`

Generate a handler that dispatches over an extensible-data input from a per-type trait.

## Overview

`#[cgp_auto_dispatch]` answers a common shape: **you have a trait with one implementation per type, and you
want it to work on an enum of those types too.**

```rust
#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}
```

Implement `HasArea` for `Circle` and for `Rectangle`, and an `enum Shape` holding those two gets `HasArea`
as well, dispatching to whichever variant it currently holds. No `match` is written anywhere.

Without the macro you would either hand-write that `match` per method, or wire up the
[dispatch combinators](../providers/dispatch/index.md) yourself: a matcher, a per-variant handler, and
the field-extraction machinery between them. The macro does exactly that wiring, so what you write is the
trait, its per-type impls, and a derive on the enum.

**Its value is narrower than it looks, and worth naming precisely.** It fits when the per-variant behaviour
is *exactly* "call the same trait method on the payload". The moment a variant needs different handling, or
the dispatch should be chosen by a **context** (the type the capability runs against) rather than fixed on
the enum, reach for the combinators directly. This is the convenient front end to
[dispatching](../providers/dispatch/index.md), not a replacement for it.

## Usage

Write the attribute above a trait definition. It takes no arguments:

```rust
#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}
```

The trait may have generic parameters and supertraits. Each method may take `self` by value, by shared
reference, or by mutable reference; may take further value or reference arguments; and may be `async`.

Two restrictions are enforced when the macro expands:

- **Every trait item must be a method.** Associated types and constants are rejected.
- **A method may not have non-lifetime generic parameters.** Lifetimes are fine. The reason is in the
  [Common Mistakes](#common-mistakes), and it is a real limitation rather than an oversight.

Each method must have a `self` receiver, since the receiver is the enum value being matched.

The enum also has to be made extensible, with [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) or the
narrower variant derives, so the machinery can take it apart one variant at a time.

## Examples

A per-type trait made to work on an enum of those types:

```rust
use cgp::prelude::*;

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

pub struct Circle { pub radius: f64 }
pub struct Rectangle { pub width: f64, pub height: f64 }

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

impl HasArea for Circle {
    fn area(&self) -> f64 {
        core::f64::consts::PI * self.radius * self.radius
    }
}

impl HasArea for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

`Shape` now implements `HasArea` too:

```rust
let shape = Shape::Rectangle(Rectangle { width: 2.0, height: 2.0 });
assert_eq!(shape.area(), 4.0);
```

**Every variant's payload must implement the trait**, and that requirement is checked: forgetting an impl for
one variant is a compile error where the enum's method is used. That is the same exhaustiveness a hand-written
`match` gives, recovered for a generic matcher. Adding a variant to the enum without an impl for it
breaks the build rather than silently falling through.

Mutating and argument-taking methods dispatch the same way, so one trait can mix them:

```rust
#[cgp_auto_dispatch]
pub trait CanScale {
    fn scale(&mut self, factor: f64);
}

impl CanScale for Circle {
    fn scale(&mut self, factor: f64) { self.radius *= factor; }
}

impl CanScale for Rectangle {
    fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
    }
}

let mut shape = Shape::Circle(Circle { radius: 1.0 });
shape.scale(2.0);
```

## When to use it

**Reach for it when an existing per-type trait should also work on an enum of those types, unchanged.** That
is the case it is built for, and within it there is nothing shorter.

Reach for something else in four situations.

- **The behaviour differs per variant.** The macro generates one handler that calls the same method on every
  payload. When a variant needs something else, use the
  [dispatch combinators](../providers/dispatch/index.md) directly and name a handler per variant.
- **The dispatch should be a wired component.** The generated impl is fixed on the enum, with a unit context
  and a unit code, so a context cannot override how one variant is handled. Wiring
  `MatchWithValueHandlers` into a context's own component gives that control.
- **A method needs to be generic.** Not supported, and not fixable by rearranging: see the
  [Common Mistakes](#common-mistakes).
- **The trait needs an associated type or const.** Also rejected. A trait carrying either is not a dispatch
  trait in this sense; give the enum a component of its own instead.

One framing worth keeping: this macro is about **retrofitting** a trait onto an enum. If you are designing
the operation from scratch and expect contexts to configure it, start with a
[`#[cgp_component]`](./cgp_component.md) and the combinators, and you will not need this.

## Under the hood

The macro keeps the trait unchanged and appends two kinds of item: one per-method computer, and one blanket
impl of the trait for a fresh enum parameter.

For each method it emits a free function turned into a provider by
[`#[cgp_computer]`](./cgp_computer.md), named `Compute` plus the method name in PascalCase:

```rust
#[cgp_computer(ComputeArea)]
fn area<'__a__, __Variants__: HasArea>(__Variants__: &'__a__ __Variants__) -> f64 {
    __Variants__.area()
}
```

The body just calls the trait method on the payload, which makes the per-variant handler "invoke
`HasArea::area` on whatever this variant holds". It is bound by `__Variants__: HasArea` so it applies to every
payload type implementing the trait, and borrows through a fresh `'__a__` lifetime to mirror the `&self`
receiver.

Then the enum-level blanket impl, which wires a matcher over the whole enum:

```rust
impl<__Variants__> HasArea for __Variants__
where
    MatchWithValueHandlersRef<ComputeArea>:
        for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        MatchWithValueHandlersRef::<ComputeArea>::compute(
            &(),
            PhantomData::<()>,
            self,
        )
    }
}
```

Three things are worth reading off that. The matcher is invoked with a **unit context and unit code**
(`&()` and `PhantomData::<()>`), because the per-variant logic depends only on the payload, which is exactly
why a context cannot influence it. The `__Variants__: HasExtractor` bound requires the enum to be
extensible. And **the first `where` bound is where a missing variant impl is reported**: it says the matcher
must be a `Computer` over this enum, which holds only if every variant's payload can be handled.

Which matcher is chosen depends on the method's receiver and argument list, and every choice comes from the
value-handler family so the per-variant computer receives the bare payload:

| Receiver | No extra arguments | With arguments |
|---|---|---|
| `&self` | `MatchWithValueHandlersRef` | `MatchFirstWithValueHandlersRef` |
| `&mut self` | `MatchWithValueHandlersMut` | `MatchFirstWithValueHandlersMut` |
| `self` | `MatchWithValueHandlers` | `MatchFirstWithValueHandlers` |

With arguments, the receiver and the arguments are bundled into the matcher's input as a tuple:

```rust
// where MatchFirstWithValueHandlersRef<ComputeContains>:
//     for<'__a__> Computer<(), (), (&'__a__ __Variants__, (f64, f64)), Output = bool>
fn contains(&self, arg_0: f64, arg_1: f64) -> bool {
    MatchFirstWithValueHandlersRef::<ComputeContains>::compute(
        &(),
        PhantomData::<()>,
        (self, (arg_0, arg_1)),
    )
}
```

An `async` method selects the `AsyncComputer` form of the bound and the call instead of `Computer`, and awaits
the result.

## Common Mistakes

**A trait method cannot be generic**, and the macro explains why in the message itself:

```text
error: Dispatch trait methods cannot contain non-lifetime generic parameters due to the lack of
       quantified constraints in Rust
```

The blanket impl would need a quantified bound ("for every instantiation of the method's type parameter,
every variant's payload satisfies it"), and Rust has no way to write that. A method that must be generic has
to be handled with the [dispatch combinators](../providers/dispatch/index.md) directly. Lifetime
parameters are fine.

**Associated types and consts are rejected.** Every trait item must be a method:

```text
error: Only function items are allowed in a dispatch trait
```

**The enum must derive the extensible-data machinery.** Without
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) or the variant derives, the `HasExtractor` bound fails
and the error names that trait rather than the missing derive.

**A missing variant impl is reported at the use site, against the matcher.** Forgetting `HasArea for Circle`
does not fail where the impl should have been; it fails where `shape.area()` is called, as an unsatisfied
bound on `MatchWithValueHandlersRef<ComputeArea>`. The variant that is missing is somewhere in the chain
rather than in the headline.

**The generated impl is a blanket impl over every type**, not just your enum. This lets it cover any
extensible enum whose payloads implement the trait, and it means the trait cannot also be implemented by
hand for some other type without colliding with it.

## Related constructs

- [Dispatch combinators](../providers/dispatch/index.md) — the matchers this wires, and what to use
  directly for anything it cannot express.
- [`#[cgp_computer]`](./cgp_computer.md) — how each per-variant handler is emitted.
- [`#[derive(CgpData)]`](../derives/derive_cgp_data.md) — what the enum needs to be dispatchable.
- [`Computer`](../components/handler/computer.md) — the component the generated bound is written against.
- [`ExtractField`](../traits/variant/extract_field.md) — the extractor family the matching walks.
- [`#[cgp_component]`](./cgp_component.md) — the better starting point when contexts should configure the
  operation.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data input to a handler per
  variant.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum representation the dispatch
  walks.

## Source

- Entry point: [`entrypoints/cgp_auto_dispatch.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-extra-macro-lib/src/entrypoints/cgp_auto_dispatch.rs)
- The matchers it generates: [`cgp-dispatch/src/providers/matchers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-dispatch/src/providers/matchers/)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
