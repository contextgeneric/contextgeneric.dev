---
title: '#[cgp_auto_dispatch] — dispatch over variants'
sidebar_label: '#[cgp_auto_dispatch]'
sidebar_position: 15
description: 'Generate a handler that dispatches over an extensible-data input from a per-type trait, routing each variant to its own implementation.'
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
the dispatch should be chosen by a **context** (the type the method runs on) rather than fixed on
the enum, use the combinators directly. This is the convenient front end to
[dispatching](../providers/dispatch/index.md), not a replacement for it.

## Usage

Write the attribute above a trait definition. It takes no arguments, and an argument is rejected
with `` `#[cgp_auto_dispatch]` takes no arguments ``:

```rust
#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}
```

The trait may have generic parameters, lifetime parameters among them, and
[supertraits](/docs/reference/glossary#supertrait), which the enum then has to implement as well.
Each method may take `self` by value, by shared reference, or by mutable reference; may take further
value or reference arguments; may name its lifetimes or leave them elided; may have a default body;
and may be `async`.

The macro enforces four restrictions when it expands:

- **Every trait item must be a method.** Associated types and constants are rejected.
- **Every method must have a `self` receiver**, since the receiver is the enum value being matched.
- **The receiver must be `self`, `&self`, or `&mut self`.** A typed receiver such as
  `self: Box<Self>` is rejected, because the generated code passes the receiver to a matcher that
  takes the enum or a borrow of it.
- **A method may not have non-lifetime generic parameters.** Lifetimes are fine. The reason is in the
  [Common Mistakes](#common-mistakes), and it is a real limitation rather than an oversight.

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

**Use it when an existing per-type trait should also work on an enum of those types, unchanged.** That
is the case it is built for, and within it there is nothing shorter.

Use something else in these situations:

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

For each method it writes a private free function under a reserved name, `__compute_` plus the
method name plus `__`, and turns it into a provider named `Compute` plus the method name in
PascalCase exactly as [`#[cgp_computer]`](./cgp_computer.md) would. It runs the function through
`#[cgp_computer]`'s own code rather than emitting the attribute, so the expansion holds the
provider's final code; the block below shows the function as `#[cgp_computer]` input, which is the
readable form:

```rust
#[cgp_computer(ComputeArea)]
fn __compute_area__<'__a__, __Variants__: HasArea>(__Variants__: &'__a__ __Variants__) -> f64 {
    __Variants__.area()
}
```

The reserved name does not collide with the module's own items, so a function called `area` beside
the trait compiles. Both generated names drop a raw-identifier prefix, so a method named `r#type`
yields `__compute_type__` and `ComputeType`.

The body calls the trait method on the payload, which makes the per-variant handler "invoke
`HasArea::area` on whatever this variant holds". It is bound by `__Variants__: HasArea` so it applies to every
payload type implementing the trait, and borrows through the receiver's lifetime, a fresh `'__a__`
when the receiver elides it as `&self` does.

**The macro names every elided lifetime in the signature the way Rust's elision rules read it**,
because it copies the signature into this function and into the bound below, where an unnamed
lifetime is either rejected or means something else. Each elided lifetime in an argument, at any
depth (`&str`, `Option<&str>`, `Foo<'_>`), gets its own fresh name, `'__a1__`, `'__a2__`, and so on,
just as elided inputs are distinct lifetimes. An elided lifetime in the return type takes the
receiver's, or, for a by-value `self`, the one lifetime the arguments use. A `fn(&T)` type or an
`Fn(&T)` bound keeps the lifetimes it binds itself. So `fn label(&self, suffix: &str) -> &str`
generates:

```rust
#[cgp_computer(ComputeLabel)]
fn __compute_label__<'__a__, '__a1__, __Variants__: CanLabel>(
    __Variants__: &'__a__ __Variants__,
    (arg_0): (&'__a1__ str),
) -> &'__a__ str {
    __Variants__.label(arg_0)
}
```

and the borrow the method returns outlives a shorter-lived `suffix`, as the trait's own signature
promises.

Then the enum-level blanket impl, which wires a matcher over the whole enum:

```rust
impl<__Variants__> HasArea for __Variants__
where
    MatchWithValueHandlersRef<ComputeArea>:
        for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        <MatchWithValueHandlersRef<ComputeArea> as Computer<_, _, _>>::compute(
            &(),
            PhantomData::<()>,
            self,
        )
    }
}
```

A few things are worth reading off that. The matcher is invoked with a **unit context and unit
code** (`&()` and `PhantomData::<()>`), because the per-variant logic depends only on the payload,
which is exactly why a context cannot influence it. The call names the provider trait, with its
arguments inferred, so it stays unambiguous in a module that also imports `CanCompute`. The
`__Variants__: HasExtractor` bound requires the enum to be extensible, and when the trait has a
supertrait, a further `__Variants__: Supertrait` bound requires the enum to provide it, for instance
by dispatching the supertrait as well. And **the first `where` bound
is where a missing variant impl is reported**: it says the matcher must be a `Computer` over this
enum, which holds only if every variant's payload can be handled.

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
    <MatchFirstWithValueHandlersRef<ComputeContains> as Computer<_, _, _>>::compute(
        &(),
        PhantomData::<()>,
        (self, (arg_0, arg_1)),
    )
}
```

The bound is quantified, in a single `for<…>`, over the method's own lifetime parameters and the
lifetimes the macro named; the trait's lifetime parameters belong to the impl and are not quantified.
An `async` method selects the `AsyncComputer` form of the bound and the call instead of `Computer`,
and awaits the result. Every name in the expansion is written as a full path, so it compiles in a
module that does not import `cgp::prelude::*`.

## Common Mistakes

**A trait method cannot be generic**, and the macro explains why in the message itself:

```text
error: Dispatch trait methods cannot contain non-lifetime generic parameters due to the lack of quantified constraints in Rust
```

The blanket impl would need a quantified bound ("for every instantiation of the method's type parameter,
every variant's payload satisfies it"), and Rust has no way to write that. A method that must be generic has
to be handled with the [dispatch combinators](../providers/dispatch/index.md) directly. Lifetime
parameters are fine.

**Associated types and consts are rejected.** Every trait item must be a method:

```text
error: Only function items are allowed in a dispatch trait
```

**A method without a receiver is rejected**, with *Dispatcher method must have a self argument*,
and **a typed receiver** such as `self: Box<Self>` with
*Dispatcher method receiver must be `self`, `&self`, or `&mut self`*.

**An attribute argument is rejected**, with
`` `#[cgp_auto_dispatch]` takes no arguments ``. The macro has nothing to configure.

**A lifetime hidden in a type's path is not named.** A type that carries a lifetime without writing
it, such as `Cow<str>` for `Cow<'_, str>`, gives the macro nothing to rename, since whether a path
hides a lifetime depends on the type's definition, which a macro cannot see. Such an argument fails
to compile with ``error[E0726]: implicit elided lifetime not allowed here``, and such a return type
with ``error[E0106]: missing lifetime specifier``. Write the lifetime out as `Cow<'_, str>`, which
the macro then names like any other elided lifetime.

**Two dispatch traits in one module cannot share a method name.** The macro names the helper and
the provider after the method alone, so a second trait with an `area` method emits a second
`__compute_area__` and a second `ComputeArea`, and the module fails with
``error[E0428]: the name `__compute_area__` is defined multiple times``, the same error for
`ComputeArea`, and `E0119` conflicts between the two providers' impls, followed by `E0277` errors on
the second trait's generated code. Declare such traits in separate modules.

**The enum must derive the extensible-data machinery.** Without
[`#[derive(CgpData)]`](../derives/derive_cgp_data.md) or the variant derives, the call fails with
the error below, and a note names `HasExtractor` rather than the missing derive:

```text
error[E0599]: the method `area` exists for reference `&Plain`, but its trait bounds were not satisfied
```

**A missing variant impl is reported at the use site, against the matcher.** Forgetting
`HasArea for Rectangle` does not fail where the impl should have been. It fails where
`shape.area()` is called, with the same `E0599` headline, and the notes list unsatisfied bounds on
`MatchWithValueHandlersRef<ComputeArea>` without naming the variant that lacks an impl.

**The generated impl is a blanket impl over every type that implements `HasExtractor`.** This lets
it cover any extensible enum whose payloads implement the trait. A hand-written impl for a type
outside that set, such as the payload structs themselves, coexists with it, but one for any
extensible enum collides, including the enum the trait was written for:

```text
error[E0119]: conflicting implementations of trait `HasArea` for type `Shape`
```

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

- Entry point: [`cgp-macro-extra-lib/src/cgp_auto_dispatch.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-extra-lib/src/cgp_auto_dispatch.rs)
- The parsing and code generation, including the lifetime naming: [`cgp-macro-extra-core/src/types/cgp_auto_dispatch/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-extra-core/src/types/cgp_auto_dispatch/)
- The matchers it generates: [`cgp-dispatch/src/providers/matchers/`](https://github.com/contextgeneric/cgp/tree/main/crates/extra/cgp-dispatch/src/providers/matchers/)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
