---
title: 'MatchFirstWithHandlers — match with arguments'
description: 'The matcher for an (Input, Args) input: it matches the enum in the first element and passes the extra arguments to every per-variant handler.'
sidebar_label: 'MatchFirstWithHandlers'
sidebar_position: 2
---

# `MatchFirstWithHandlers`

Match an enum that arrives together with extra arguments, passing the arguments to every handler.

## Overview

`MatchFirstWithHandlers<Handlers>` is the matcher for an input of the form `(Input, Args)`: the enum
being matched, first, and extra arguments that every per-variant handler needs, such as a scale
factor or a target buffer. It works like [`MatchWithHandlers`](match_with_handlers.md) on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, except
that `Args` travels with the extractor through the loop, so on a miss the next handler still
receives the arguments. Like every CGP provider, it carries no runtime value; the handler list rides
in `PhantomData`.

## Usage

Import it from `cgp::extra::dispatch`. It takes one type parameter, a
[`Product!`](../../macros/product.md) list of per-variant adapters built from the first-argument
forms [`ExtractFirstFieldAndHandle`](extract_field_and_handle.md) and
[`HandleFirstFieldValue`](handle_field_value.md), which carry `Args` alongside the payload:

```rust
use cgp::extra::dispatch::{
    ExtractFirstFieldAndHandle, HandleFirstFieldValue, MatchFirstWithHandlers,
};

delegate_components! {
    App {
        ComputerComponent:
            MatchFirstWithHandlers<Product![
                ExtractFirstFieldAndHandle<Symbol!("Circle"), HandleFirstFieldValue<ScaledArea>>,
                ExtractFirstFieldAndHandle<Symbol!("Rectangle"), HandleFirstFieldValue<ScaledArea>>,
            ]>,
    }
}
```

Each payload handler then computes over `(Circle, Args)` or `(Rectangle, Args)`. Two borrowed forms
take the same list: `MatchFirstWithHandlersRef<Handlers>` over `(&Input, Args)` and
`MatchFirstWithHandlersMut<Handlers>` over `(&mut Input, Args)`. All three implement `Computer` and
`AsyncComputer`, and nothing else.

## Examples

A `Shape` enum whose area is scaled by a factor passed with it:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{
    ExtractFirstFieldAndHandle, HandleFirstFieldValue, MatchFirstWithHandlers,
};
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
impl<Context, Code> Computer<Context, Code, (Circle, f64)> for ScaledArea {
    type Output = f64;

    fn compute(
        _context: &Context,
        _code: PhantomData<Code>,
        (circle, scale): (Circle, f64),
    ) -> f64 {
        core::f64::consts::PI * circle.radius * circle.radius * scale * scale
    }
}

#[cgp_provider]
impl<Context, Code> Computer<Context, Code, (Rectangle, f64)> for ScaledArea {
    type Output = f64;

    fn compute(
        _context: &Context,
        _code: PhantomData<Code>,
        (rectangle, scale): (Rectangle, f64),
    ) -> f64 {
        rectangle.width * rectangle.height * scale * scale
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent:
            MatchFirstWithHandlers<Product![
                ExtractFirstFieldAndHandle<Symbol!("Circle"), HandleFirstFieldValue<ScaledArea>>,
                ExtractFirstFieldAndHandle<Symbol!("Rectangle"), HandleFirstFieldValue<ScaledArea>>,
            ]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), (Shape, f64)),
    }
}

pub fn demo() {
    let code = PhantomData::<()>;
    let rectangle = Shape::Rectangle(Rectangle {
        width: 3.0,
        height: 4.0,
    });

    assert_eq!(App.compute(code, (rectangle, 2.0)), 48.0); // 12 * 2 * 2
}
```

The `Circle` arm misses and hands the extractor and the `2.0` on together, so `ScaledArea` receives
the `Rectangle` payload beside its scale factor. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `MatchFirstWithHandlers` when every per-variant handler needs the same extra
arguments.** When the handlers need only the payload, [`MatchWithHandlers`](match_with_handlers.md)
is simpler. When every payload goes to one provider, the convenience aliases build the list from the
enum: `MatchFirstWithValueHandlers<ScaledArea>` answers the example above on its own, and
`MatchFirstWithFieldHandlers` keeps each payload tagged. They come in the same three input modes as
their plain counterparts, [`MatchWithValueHandlers`](match_with_value_handlers.md) and
[`MatchWithFieldHandlers`](match_with_field_handlers.md). The value forms are in the prelude; the
field forms are imported from `cgp::extra::dispatch`.

## Under the hood

`MatchFirstWithHandlers<Handlers>` carries the list in `PhantomData`. Its `Computer` impl runs the
shared matcher loop over `(Input::Extractor, Args)`, so each miss returns `(Remainder, Args)` and
the arguments stay with the extractor, and it discharges the leftover remainder once every variant
is ruled out:

```rust
pub struct MatchFirstWithHandlers<Handlers>(pub PhantomData<Handlers>);

#[cgp_provider]
impl<Context, Code, Input, Args, Output, Remainder, Handlers> Computer<Context, Code, (Input, Args)>
    for MatchFirstWithHandlers<Handlers>
where
    Input: HasExtractor,
    DispatchMatchers<Handlers>: Computer<
            Context,
            Code,
            (Input::Extractor, Args),
            Output = Result<Output, (Remainder, Args)>,
        >,
    Remainder: FinalizeExtract,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, (input, args): (Input, Args)) -> Output {
        let res = DispatchMatchers::compute(context, code, (input.to_extractor(), args));
        match res {
            Ok(output) => output,
            Err((remainder, _)) => remainder.finalize_extract(),
        }
    }
}
```

The loop is the `DispatchMatchers` alias described on
[`MatchWithHandlers`](match_with_handlers.md#under-the-hood). The `AsyncComputer` impl has the same
bounds over `AsyncComputer`. The convenience aliases are `UseInputDelegate` tables keyed on
`(Input, Args)`, and on `(&'a Input, Args)` and `(&'a mut Input, Args)` for the borrowed forms, each
mapping the input to this matcher over a list built from the enum's variants.

## Common Mistakes

**The mistakes of the plain matcher apply unchanged.** A list that misses a variant fails the
`FinalizeExtract` bound, and the matcher cannot fill a `TryComputer` or `Handler` slot without a
promotion; [`MatchWithHandlers`](match_with_handlers.md#common-mistakes) shows both.

## Related constructs

- [`MatchWithHandlers`](match_with_handlers.md) — the plain matcher, for handlers that need only the
  payload.
- [`ExtractFieldAndHandle`](extract_field_and_handle.md),
  [`HandleFieldValue`](handle_field_value.md) — document the first-argument adapters its list is
  built from.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) — the convenience matcher whose
  `MatchFirstWith…` forms build this list automatically.
- [`HasExtractor`](../../traits/variant/has_extractor.md),
  [`FinalizeExtract`](../../traits/variant/finalize_extract.md) — the traits it stands on.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing a value, with extra arguments, to per-variant
  handlers.

## Source

- [`providers/with_handlers/match_first_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/match_first_with_handlers.rs),
  with the borrowed forms in `match_first_with_handlers_ref.rs` and
  `match_first_with_handlers_mut.rs`, and the convenience aliases in
  [`providers/matchers/match_first_with_field_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/matchers/match_first_with_field_handlers.rs).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
