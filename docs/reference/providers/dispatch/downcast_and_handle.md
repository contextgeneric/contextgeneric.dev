---
title: 'DowncastAndHandle — match a group of variants'
description: 'The matcher adapter that narrows an enum to a smaller enum of some of its variants and hands that whole group to one inner provider.'
sidebar_label: 'DowncastAndHandle'
sidebar_position: 7
---

# `DowncastAndHandle`

Match a group of variants in one arm, by narrowing the input to a smaller enum.

## Overview

`DowncastAndHandle<Inner, Provider>` is the matcher arm for several variants that one provider
handles together. Instead of extracting one variant, it tries to narrow the input to `Inner`, a
smaller enum whose variants are a subset of the input's. When the value is one of those variants,
it hands the whole `Inner` value to `Provider` and returns `Ok`; otherwise it returns `Err` of the
remainder, with every variant of `Inner` ruled out, on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against.
`Provider` defaults to [`UseContext`](../use_context.md). Like every CGP provider, it carries no
runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the smaller enum and an optional inner provider, and
sits in a matcher's list beside single-variant arms:

```rust
use cgp::extra::dispatch::{
    DowncastAndHandle, ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers,
};

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CountCircleCorners>>,
                DowncastAndHandle<Quadrilateral, CountQuadrilateralCorners>,
            ]>,
    }
}
```

`Inner` needs [`HasFields`](../../traits/shape/has_fields.md) and
[`FromVariant`](../../traits/variant/from_variant.md), which `#[derive(CgpData)]` gives it, and each
of its variants must be a variant of the input with the same name and payload type. It implements
`Computer` and `AsyncComputer`.

## Examples

Counting the corners of a shape, with the two quadrilaterals handled as one group:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{
    DowncastAndHandle, ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers,
};
use cgp::extra::handler::CanCompute;

pub struct Circle {
    pub radius: f64,
}

pub struct Square {
    pub side: f64,
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

#[derive(CgpData)]
pub enum Shape {
    Circle(Circle),
    Square(Square),
    Rectangle(Rectangle),
}

#[derive(CgpData)]
pub enum Quadrilateral {
    Square(Square),
    Rectangle(Rectangle),
}

#[cgp_computer]
pub fn count_circle_corners(_circle: Circle) -> u32 {
    0
}

#[cgp_computer]
pub fn count_quadrilateral_corners(_shape: Quadrilateral) -> u32 {
    4
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent:
            MatchWithHandlers<Product![
                ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<CountCircleCorners>>,
                DowncastAndHandle<Quadrilateral, CountQuadrilateralCorners>,
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

    assert_eq!(App.compute(code, Shape::Circle(Circle { radius: 1.0 })), 0);
    assert_eq!(App.compute(code, Shape::Square(Square { side: 2.0 })), 4);
    let rectangle = Shape::Rectangle(Rectangle {
        width: 3.0,
        height: 4.0,
    });
    assert_eq!(App.compute(code, rectangle), 4);
}
```

The downcast arm rules out both `Square` and `Rectangle` on a miss, so after the two arms nothing is
left and the list is complete. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `DowncastAndHandle` when several variants share one provider that wants them as an
enum of their own**, such as a sub-matcher or a function over a narrower type. For a single variant
use [`ExtractFieldAndHandle`](extract_field_and_handle.md). Both go in a list for
[`MatchWithHandlers`](match_with_handlers.md).

## Under the hood

`DowncastAndHandle` carries the smaller enum and the provider in `PhantomData`; the struct's first
parameter is named `Input`, while the impl names it `Inner`. The `Computer` impl narrows the input
with [`CanDowncastFields<Inner>`](../../traits/casting/can_downcast_fields.md), returning early with
the remainder through `?`:

```rust
pub struct DowncastAndHandle<Input, Provider = UseContext>(pub PhantomData<(Input, Provider)>);

#[cgp_provider]
impl<Context, Code, Input, Provider, Inner, Output, Remainder> Computer<Context, Code, Input>
    for DowncastAndHandle<Inner, Provider>
where
    Input: CanDowncastFields<Inner, Remainder = Remainder>,
    Provider: Computer<Context, Code, Inner, Output = Output>,
{
    type Output = Result<Output, Remainder>;

    fn compute(
        context: &Context,
        tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Output, Remainder> {
        let inner = input.downcast_fields(PhantomData::<Inner>)?;
        let output = Provider::compute(context, tag, inner);
        Ok(output)
    }
}
```

`CanDowncastFields` works on the extractor, not the enum, which is why the arm fits in a matcher's
loop. The `AsyncComputer` impl matches on the downcast and awaits `Provider::compute_async` on a
hit.

## Related constructs

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the single-variant arm.
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher that runs a list mixing both.
- [`CanDowncastFields`](../../traits/casting/can_downcast_fields.md) — the cast it uses to narrow
  the input.
- [`UseContext`](../use_context.md) — the default inner provider.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing a group of variants to one handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern and its casts.

## Source

- [`providers/field_matchers/extract_handle.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/extract_handle.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
