---
title: 'HandleFieldValue — strip a variant''s tag'
description: 'The adapter that unwraps the tagged Field a matcher arm delivers and passes the bare payload to an inner provider, so an ordinary computer can handle it.'
sidebar_label: 'HandleFieldValue'
sidebar_position: 6
---

# `HandleFieldValue`

Strip the variant tag from a matched payload and pass the bare value to an inner provider.

## Overview

`HandleFieldValue<Provider>` sits between a matcher arm and the provider that does the work. An arm
such as [`ExtractFieldAndHandle`](extract_field_and_handle.md) delivers its payload as a
[`Field<Tag, Value>`](../../types/field.md), with the variant name attached as a type. Most payload
handlers are ordinary computers over the payload type and know nothing of that tag, so
`HandleFieldValue` unwraps the `Field` and passes the bare `Value` to `Provider`, on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against.
`Provider` defaults to [`UseContext`](../use_context.md). Like every CGP provider, it carries no
runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes one optional type parameter, the inner provider, and
is written inside an arm:

```rust
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue};

type CircleArm = ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>;
```

The first-argument form `HandleFirstFieldValue<Provider>` does the same for the
`(Field<Tag, Value>, Args)` input an [`ExtractFirstFieldAndHandle`](extract_field_and_handle.md) arm
delivers, forwarding `(Value, Args)`. Both implement `Computer` and `AsyncComputer`.

## Examples

`ComputeArea` is a plain computer over `Circle` and `Rectangle`, reached from each arm through
`HandleFieldValue`:

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
    let circle = App.compute(PhantomData::<()>, Shape::Circle(Circle { radius: 2.0 }));
    assert!((circle - 4.0 * core::f64::consts::PI).abs() < 1e-9);
}
```

Without `HandleFieldValue`, `ComputeArea` would need impls over `Field<Symbol!("Circle"), Circle>`
and `Field<Symbol!("Rectangle"), Rectangle>` instead. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `HandleFieldValue` inside a spelled-out arm whose provider wants the bare payload.** You
rarely write it with the convenience matchers, because
[`MatchWithValueHandlers`](match_with_value_handlers.md) adds it to every arm; that is its one
difference from [`MatchWithFieldHandlers`](match_with_field_handlers.md). Leave it out when the
provider needs the variant's name from the tag.

## Under the hood

`HandleFieldValue<Provider>` carries the inner provider in `PhantomData`, and its `Computer` impl
reads the `Field`'s `value` and forwards it:

```rust
pub struct HandleFieldValue<Provider = UseContext>(pub PhantomData<Provider>);

#[cgp_provider]
impl<Context, Code, Tag, Input, Output, Provider> Computer<Context, Code, Field<Tag, Input>>
    for HandleFieldValue<Provider>
where
    Provider: Computer<Context, Code, Input, Output = Output>,
{
    type Output = Output;

    fn compute(
        context: &Context,
        tag: PhantomData<Code>,
        input: Field<Tag, Input>,
    ) -> Self::Output {
        Provider::compute(context, tag, input.value)
    }
}
```

The `AsyncComputer` impl forwards the value to `Provider::compute_async`, and
`HandleFirstFieldValue` has the same pair over `(Field<Tag, Input>, Args)`, forwarding
`(input.value, args)`.

## Related constructs

- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the arm that delivers the `Field` this
  unwraps.
- [`MatchWithValueHandlers`](match_with_value_handlers.md) — adds this to every arm;
  [`MatchWithFieldHandlers`](match_with_field_handlers.md) does not.
- [`Field`](../../types/field.md) — the tagged payload it unwraps.
- [`UseContext`](../use_context.md) — the default inner provider.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — preparing a matched payload for its handler.

## Source

- [`providers/field_matchers/field_value.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/field_value.rs),
  with the first-argument form in `first_field_value.rs`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
