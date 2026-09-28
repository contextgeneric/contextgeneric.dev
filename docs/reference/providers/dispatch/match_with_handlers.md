---
title: 'MatchWithHandlers — match an enum, arm by arm'
description: 'The matcher that runs a spelled-out list of per-variant handlers over an enum, stopping at the first match and needing no wildcard arm.'
sidebar_label: 'MatchWithHandlers'
sidebar_position: 1
---

# `MatchWithHandlers`

Match an enum against a spelled-out list of per-variant handlers, with no wildcard arm.

## Overview

`MatchWithHandlers<Handlers>` is the matcher for an enum whose variants each have their own handler.
It turns the input into its extractor, a view of the enum that can rule variants out one at a time,
and runs a list of handlers over it on a [**context**](/docs/reference/glossary#context), the type
the implementation runs against. The first handler whose variant matches produces the output. When
the list has a handler for every variant, the last miss leaves nothing to match, so the matcher
returns the bare output with no fallback arm; a list that misses a variant does not compile. Like
every CGP provider, it carries no runtime value; the handler list rides in `PhantomData`.

## Usage

Import it from `cgp::extra::dispatch`. It takes one type parameter, a
[`Product!`](../../macros/product.md) list of per-variant adapters. The list is normally one
[`ExtractFieldAndHandle`](extract_field_and_handle.md) per variant, each wrapping its payload
handler in [`HandleFieldValue`](handle_field_value.md) so the handler receives the bare payload:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
            ]>,
    }
}
```

Two borrowed forms take the same list and match without moving the value:
`MatchWithHandlersRef<Handlers>` over `&Input` and `MatchWithHandlersMut<Handlers>` over
`&mut Input`. Their adapters extract a borrowed payload, so each payload handler computes over
`&Circle` or `&mut Circle`, and the check names the borrowed input:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlersRef};

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlersRef<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeAreaRef>>,
                ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeAreaRef>>,
            ]>,
    }
}

check_components! {
    App {
        ComputerComponent: <'a> ((), &'a Shape),
    }
}
```

All three implement `Computer` and `AsyncComputer`, and nothing else.

## Examples

A `Shape` enum dispatched to one area provider with an impl per payload type:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
use cgp::extra::handler::CanCompute;

pub struct Circle {
    pub radius: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, circle: Circle) -> f64 {
        core::f64::consts::PI * circle.radius * circle.radius
    }
}

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
    type Output = f64;

    fn compute(_context: &Context, _code: PhantomData<Code>, rectangle: Rectangle) -> f64 {
        rectangle.width * rectangle.height
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>,
                ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>,
            ]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), Shape),
    }
}

pub fn demo() {
    let code = PhantomData::<()>;

    let rectangle = Shape::Rectangle(Rectangle {
        width: 3.0,
        height: 4.0,
    });
    assert_eq!(App.compute(code, rectangle), 12.0);

    let circle = App.compute(code, Shape::Circle(Circle { radius: 1.0 }));
    assert!((circle - core::f64::consts::PI).abs() < 1e-9);
}
```

A rectangle misses the `Circle` arm, and the extractor, now with `Circle` ruled out, reaches the
`Rectangle` arm. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: it acts on the
`Shape` input.

## When to use it

**Reach for `MatchWithHandlers` when each variant needs a handler you name**, or when the variants
map to handlers in a way the enum's field list cannot generate. When every payload goes to the same
provider, [`MatchWithValueHandlers`](match_with_value_handlers.md) builds this list from the enum
and is shorter. When the handlers also take extra arguments, use
[`MatchFirstWithHandlers`](match_first_with_handlers.md). And when the per-variant handlers return
`Result`, [`TryPromote`](../handler/try_promote.md) turns the matcher's `Result` output into a
`TryComputer`.

## Under the hood

`MatchWithHandlers<Handlers>` carries the list in `PhantomData`, and its `Computer` impl turns the
input into its [extractor](../../traits/variant/has_extractor.md), runs the list over it, and
discharges the leftover with
[`finalize_extract_result`](../../traits/variant/finalize_extract_result.md):

```rust
pub struct MatchWithHandlers<Handlers>(pub PhantomData<Handlers>);

#[cgp_provider]
impl<Context, Code, Input, Output, Remainder, Handlers> Computer<Context, Code, Input>
    for MatchWithHandlers<Handlers>
where
    Input: HasExtractor,
    DispatchMatchers<Handlers>:
        Computer<Context, Code, Input::Extractor, Output = Result<Output, Remainder>>,
    Remainder: FinalizeExtract,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, input: Input) -> Output {
        DispatchMatchers::compute(context, code, input.to_extractor()).finalize_extract_result()
    }
}
```

`DispatchMatchers` is the loop every matcher shares, an alias for
[`PipeMonadic`](../monad/pipe_monadic.md) under the `OkMonadic` monad:

```rust
pub type DispatchMatchers<Providers> = PipeMonadic<OkMonadic, Providers>;
```

Each adapter returns `Ok(output)` when its variant matches and `Err(remainder)` when it does not,
handing back the extractor with that variant ruled out. Under `OkMonadic` the loop passes each `Err`
to the next adapter and stops at the first `Ok`. After the last adapter, `Remainder` has every
variant ruled out, and only then does it implement
[`FinalizeExtract`](../../traits/variant/finalize_extract.md), which turns the impossible `Err` into
any type. The `AsyncComputer` impl has the same bounds over `AsyncComputer`, and the borrowed forms
call `extractor_ref()` or `extractor_mut()` in place of `to_extractor()`.

## Common Mistakes

**A list that misses a variant does not compile.** Leaving out the `Rectangle` arm leaves the
extractor with a variant still present after the last adapter, so the leftover has no
`FinalizeExtract` impl, and the check reports it by the partial enum's markers: `IsVoid` for the
ruled-out `Circle`, `IsPresent` for the unmatched `Rectangle`:

```text
error[E0277]: the trait bound `__PartialShape<IsVoid, IsPresent>: FinalizeExtract` is not satisfied
```

**A matcher cannot fill a fallible slot directly.** It implements only `Computer` and
`AsyncComputer`, so wiring it to `HandlerComponent` fails, and rustc lists the two impls it has:

```text
help: `MatchWithHandlers<Handlers>` implements trait `IsProviderFor<Component, Context, Params>`
...
   | |_______________________________^ `IsProviderFor<cgp::prelude::ComputerComponent, Context, (Code, Input)>`
...
   | |_______________________________^ `IsProviderFor<AsyncComputerComponent, Context, (Code, Input)>`
   = note: required for `App` to implement `CanUseComponent<cgp::prelude::HandlerComponent, ((), Shape)>`
```

Lift it with the handler promotions instead, as `TryComputerComponent: Promote<AreaMatcher>` and
`HandlerComponent: PromoteAsync<Promote<AreaMatcher>>`, where `AreaMatcher` names the matcher type.

## Related constructs

- [`MatchWithValueHandlers`](match_with_value_handlers.md) — builds the handler list from the enum's
  own variants.
- [`MatchFirstWithHandlers`](match_first_with_handlers.md) — the same matcher for an input that
  carries extra arguments.
- [`ExtractFieldAndHandle`](extract_field_and_handle.md),
  [`HandleFieldValue`](handle_field_value.md), [`DowncastAndHandle`](downcast_and_handle.md) — the
  adapters a list is built from.
- [`PipeMonadic`](../monad/pipe_monadic.md) — the pipeline the matcher loop is built on.
- [`HasExtractor`](../../traits/variant/has_extractor.md),
  [`FinalizeExtract`](../../traits/variant/finalize_extract.md) — the traits it stands on.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing an extensible-data value to per-variant
  handlers.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern the matchers serve.

## Source

- [`providers/with_handlers/match_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/match_with_handlers.rs),
  with the borrowed forms in `match_with_handlers_ref.rs` and `match_with_handlers_mut.rs`, and the
  loop alias in
  [`providers/dispatchers/dispatch_matchers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/dispatchers/dispatch_matchers.rs).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
