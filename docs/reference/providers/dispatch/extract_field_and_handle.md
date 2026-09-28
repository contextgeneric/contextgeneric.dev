---
title: 'ExtractFieldAndHandle — one matcher arm'
description: 'The matcher adapter that tries one variant, hands its payload to an inner provider on a match, and passes the remainder on when the variant does not match.'
sidebar_label: 'ExtractFieldAndHandle'
sidebar_position: 5
---

# `ExtractFieldAndHandle`

Try one variant of an enum, handing its payload to an inner provider on a match and the remainder
on to the next arm otherwise.

## Overview

`ExtractFieldAndHandle<Tag, Provider>` is one arm of a matcher's list. It takes an extractor, a view
of an enum that can rule variants out one at a time, and tries to extract the variant named `Tag`.
On a match it wraps the payload in a [`Field<Tag, Value>`](../../types/field.md) and returns `Ok` of
what `Provider` computes from it; on a miss it returns `Err` of the remainder, the extractor with
that variant ruled out, which the next arm takes as its input. It runs on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against, and
`Provider` defaults to [`UseContext`](../use_context.md). Like every CGP provider, it carries no
runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the variant's tag, a
[`Symbol!`](../../macros/symbol.md) naming it, and an optional inner provider. It is written as an
element of a matcher's list, usually around [`HandleFieldValue`](handle_field_value.md) so the inner
provider receives the bare payload rather than the `Field`:

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

The first-argument form `ExtractFirstFieldAndHandle<Tag, Provider>` is the same arm for an
`(Input, Args)` input, as [`MatchFirstWithHandlers`](match_first_with_handlers.md) runs it. It hands
`Provider` a `(Field<Tag, Value>, Args)` on a match and returns `Err((remainder, args))` on a miss,
so the arguments travel on with the remainder. Both forms implement `Computer` and `AsyncComputer`.

## Examples

Two arms called by hand, to show the `Result` a matcher's loop passes between them:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue};

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

pub type CircleArm = ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>;
pub type RectangleArm =
    ExtractFieldAndHandle<Symbol!("Rectangle"), HandleFieldValue<ComputeArea>>;

pub struct App;

pub fn demo() {
    let code = PhantomData::<()>;

    // A hit: the circle arm extracts its variant and returns `Ok` of the area.
    let circle = Shape::Circle(Circle { radius: 1.0 });
    let hit = CircleArm::compute(&App, code, circle.to_extractor());
    assert_eq!(hit.ok(), Some(core::f64::consts::PI));

    // A miss: the circle arm hands back the remainder, which the rectangle arm takes.
    let rectangle = Shape::Rectangle(Rectangle {
        width: 3.0,
        height: 4.0,
    });
    match CircleArm::compute(&App, code, rectangle.to_extractor()) {
        Ok(_) => panic!("a rectangle is not a circle"),
        Err(remainder) => {
            let area = RectangleArm::compute(&App, code, remainder);
            assert_eq!(area.ok(), Some(12.0));
        }
    }
}
```

The arms are called through the `Computer` provider trait with `App` as the context, which needs no
wiring because `ComputeArea` reads nothing from it. `to_extractor` comes from the
[`HasExtractor`](../../traits/variant/has_extractor.md) impl that `#[derive(CgpData)]` gives
`Shape`. A matcher runs exactly this chain, and stops at the first `Ok`.

## When to use it

**Reach for `ExtractFieldAndHandle` when you spell out a matcher's list**, one arm per variant, for
[`MatchWithHandlers`](match_with_handlers.md). When every payload goes to one provider,
[`MatchWithValueHandlers`](match_with_value_handlers.md) and
[`MatchWithFieldHandlers`](match_with_field_handlers.md) build the list of these arms from the enum.
To handle several variants in one arm, use [`DowncastAndHandle`](downcast_and_handle.md).

## Under the hood

`ExtractFieldAndHandle<Tag, Provider>` carries the tag and provider in `PhantomData`. Its `Computer`
impl calls [`ExtractField<Tag>`](../../traits/variant/extract_field.md) on the input, returns early
with the remainder on a miss through `?`, and converts the payload into a `Field` for `Provider`:

```rust
pub struct ExtractFieldAndHandle<Tag, Provider = UseContext>(pub PhantomData<(Tag, Provider)>);

#[cgp_provider]
impl<Context, Code, Input, Tag, Value, Provider, Output, Remainder> Computer<Context, Code, Input>
    for ExtractFieldAndHandle<Tag, Provider>
where
    Input: ExtractField<Tag, Value = Value, Remainder = Remainder>,
    Provider: Computer<Context, Code, Field<Tag, Value>, Output = Output>,
{
    type Output = Result<Output, Remainder>;

    fn compute(
        context: &Context,
        tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Output, Remainder> {
        let value = input.extract_field(PhantomData::<Tag>)?;
        let output = Provider::compute(context, tag, value.into());
        Ok(output)
    }
}
```

The `AsyncComputer` impl matches on the extraction and awaits `Provider::compute_async` on a hit.
`ExtractFirstFieldAndHandle` has the same two impls over `(Input, Args)`, with
`Output = Result<Output, (Remainder, Args)>`.

## Related constructs

- [`HandleFieldValue`](handle_field_value.md) — strips the `Field` this arm delivers, so the inner
  provider receives the bare payload.
- [`DowncastAndHandle`](downcast_and_handle.md) — the arm that handles a group of variants at once.
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher that runs a list of these arms.
- [`ExtractField`](../../traits/variant/extract_field.md) — the trait it drives.
- [`Field`](../../types/field.md) — the tagged payload it forwards.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing one variant to a handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern and its extractors.

## Source

- [`providers/field_matchers/extract_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_matchers/extract_field.rs),
  with the first-argument form in `extract_first_field.rs`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
