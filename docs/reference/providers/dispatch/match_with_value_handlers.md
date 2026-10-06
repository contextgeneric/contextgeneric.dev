---
title: 'MatchWithValueHandlers — match every variant'
description: 'The convenience matcher that builds the per-variant handler list from an enum''s own variants and passes each payload to one provider as a bare value.'
sidebar_label: 'MatchWithValueHandlers'
sidebar_position: 3
---

# `MatchWithValueHandlers`

Match an enum by building the per-variant handler list from its own variants, passing each payload
as a bare value.

## Overview

`MatchWithValueHandlers<Provider>` is the matcher for the common case, where every variant's
payload goes to one provider. Rather than a spelled-out list, it builds one adapter per variant from
the input enum's own field list, so adding a variant needs no new arm, and it hands
each payload to `Provider` as a bare value on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. That
makes it the form for payload handlers that are ordinary computers over the payload type, such as
ones from [`#[cgp_computer]`](../../macros/cgp_computer.md). `Provider` defaults to
[`UseContext`](../use_context.md), so each payload goes back through the context's own wiring. Like
every CGP provider, it carries no runtime value.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. It takes one optional type parameter, the
per-variant provider. With the default `UseContext`, it is wired by input type with the [`open`
statement](../../macros/delegate_components.md#choosing-a-provider-per-type-the-open-statement),
beside the entries that answer each payload type:

```rust
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.<Code> Code.[Circle, Rectangle]: ComputeArea,
        @ComputerComponent.<Code> Code.Shape: MatchWithValueHandlers,
    }
}
```

Each key has one segment per type parameter of `CanCompute<Code, Input>`, and the generic first
segment matches any code, so the second selects by the input type. A named provider needs no
per-type entries: `ComputerComponent: MatchWithValueHandlers<ComputeArea>` sends every payload
straight to `ComputeArea`.

The borrowed form `MatchWithValueHandlersRef<Provider>` matches without moving the value and answers
two components. It serves `ComputerComponent` over `&Input` with a provider over borrowed payloads,
and `ComputerRefComponent` over `Input` with a
[`ComputerRef`](../../components/handler/computer_ref.md) provider over the payloads:

```rust
delegate_components! {
    App {
        ComputerComponent: MatchWithValueHandlersRef<ComputeAreaOfRef>,
        ComputerRefComponent: MatchWithValueHandlersRef<ComputeAreaByRef>,
    }
}

check_components! {
    App {
        ComputerComponent: <'a> ((), &'a Shape),
        ComputerRefComponent: ((), Shape),
    }
}
```

`MatchWithValueHandlersMut<Provider>` serves `ComputerComponent` the same way over `&mut Input`,
with a provider over mutable payloads that can change them in place. Its `…Ref` entries cannot
resolve, because `PromoteRef` lends only a shared borrow. The multi-argument forms
`MatchFirstWithValueHandlers`, `MatchFirstWithValueHandlersRef`, and
`MatchFirstWithValueHandlersMut` are on
[`MatchFirstWithHandlers`](match_first_with_handlers.md#when-to-use-it).

## Examples

A context that computes each shape directly, and a `Shape` enum by matching it:

```rust
use cgp::prelude::*;
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
        open ComputerComponent;

        @ComputerComponent.<Code> Code.[Circle, Rectangle]: ComputeArea,
        @ComputerComponent.<Code> Code.Shape: MatchWithValueHandlers,
    }
}

check_components! {
    App {
        ComputerComponent: [((), Circle), ((), Rectangle), ((), Shape)],
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

For a `Shape`, the matcher extracts the payload and asks `App` again, through `UseContext`, for the
payload's type, which reaches `ComputeArea`. `App` is an
[environmental context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: it acts on the
input. Existing code writes the same table as a
[`UseInputDelegate`](../handler/use_input_delegate.md) table.

## When to use it

**Reach for `MatchWithValueHandlers` when every variant's payload goes to the same provider as a
bare value.** This is the usual case, and shorter than spelling out the list with
[`MatchWithHandlers`](match_with_handlers.md). Reach for
[`MatchWithFieldHandlers`](match_with_field_handlers.md) when the handler needs the variant's name,
and for `MatchWithHandlers` when different variants need providers the field list cannot generate.

## Under the hood

`MatchWithValueHandlers<Provider>` is a type alias over
[`UseInputDelegate`](../handler/use_input_delegate.md), which keys on the input type and, for each
enum, maps it to [`MatchWithHandlers`](match_with_handlers.md) over a list built from the enum's
[`HasFields`](../../traits/shape/has_fields.md):

```rust
pub type MatchWithValueHandlers<Provider = UseContext> =
    UseInputDelegate<MatchWithFieldHandlersInputs<HandleFieldValue<Provider>>>;

delegate_components! {
    <Input: HasFieldHandlers<MapExtractFieldAndHandle<Provider>>, Provider>
    new MatchWithFieldHandlersInputs<Provider> {
        Input: MatchWithHandlers<Input::Handlers>
    }
}
```

Wrapping `Provider` in [`HandleFieldValue`](handle_field_value.md) is its one difference from
`MatchWithFieldHandlers`, and it makes each payload arrive bare. The list is built by the
`HasFieldHandlers` machinery described on
[`MatchWithFieldHandlers`](match_with_field_handlers.md#under-the-hood).

The borrowed forms are structs with a delegation table rather than aliases. The owned-input entries
key a `UseInputDelegate` on `&'a Input` or `&'a mut Input` and reach `MatchWithHandlersRef` or
`MatchWithHandlersMut`; the `…Ref` entries wrap that in [`PromoteRef`](../handler/promote_ref.md),
which borrows the input and hands each borrowed payload to the provider's `…Ref` impl:

```rust
delegate_components! {
    <Provider>
    MatchWithValueHandlersRef<Provider> {
        [
            ComputerComponent,
            TryComputerComponent,
            AsyncComputerComponent,
            HandlerComponent,
        ]:
            UseInputDelegate<MatchWithFieldHandlersInputsRef<HandleFieldValue<Provider>>>,
        [
            ComputerRefComponent,
            TryComputerRefComponent,
            AsyncComputerRefComponent,
            HandlerRefComponent,
        ]:
            PromoteRef<UseInputDelegate<MatchWithFieldHandlersInputsRef<HandleFieldValue<PromoteRef<Provider>>>>>,
    }
}
```

## Common Mistakes

**The input must be an enum.** The list is built by walking a sum-type field list, so a struct,
whose field list is a product, has no list and no extractor. Wiring a `Scene` struct fails with:

```text
error[E0277]: the trait bound `MatchWithHandlers<_>: cgp::prelude::Computer<App, (), Scene>` is not satisfied
```

Handle a struct's fields one by one instead, or build a record with the
[builders](build_with_handlers.md).

**The borrowed forms route fallible components they cannot answer.** Their tables list
`TryComputerComponent`, `HandlerComponent`, and the fallible `…Ref` components, but the borrowed
matchers behind them implement only `Computer` and `AsyncComputer`, so wiring
`HandlerComponent: MatchWithValueHandlersRef<ComputeAreaOfRef>` fails at the check:

```text
   = note: required for `cgp::prelude::MatchWithValueHandlersRef<ComputeAreaOfRef>` to implement `IsProviderFor<cgp::prelude::HandlerComponent, App, ((), &'a Shape)>`
```

The owned `MatchWithValueHandlers` has the same limit. Wire the matcher to `ComputerComponent` or
`AsyncComputerComponent`, and lift it with a promotion as
[`MatchWithHandlers`](match_with_handlers.md#common-mistakes) shows.

**Variants with no fields all reach the same handler.** The
[variant derives](../../derives/derive_cgp_variant.md) give each such variant, such as `Closed` or
`Paused()`, the payload `Nil`. The provider receives only the payload, so its `Nil` impl serves
every one of them and cannot tell which variant it got. Use
[`MatchWithFieldHandlers`](match_with_field_handlers.md) when they need different handling: its
provider receives `Field<Symbol!("Closed"), Nil>` and can read the variant name from the tag. The
borrowed forms handle these variants too, with a `&Nil` or `&mut Nil` payload.

## Related constructs

- [`MatchWithFieldHandlers`](match_with_field_handlers.md) — the sibling that keeps the variant tag
  on the payload.
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher this builds its list for.
- [`UseInputDelegate`](../handler/use_input_delegate.md) — the input dispatcher this alias is built
  on.
- [`HandleFieldValue`](handle_field_value.md), [`UseContext`](../use_context.md) — the wrapper it
  adds and the default per-variant provider.
- [`#[cgp_auto_dispatch]`](../../macros/cgp_auto_dispatch.md) — generates trait impls for an enum
  on top of these matchers.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing each variant of an enum to a handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern it serves.

## Source

- [`providers/matchers/match_with_field_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/matchers/match_with_field_handlers.rs),
  with the first-argument forms in `match_first_with_field_handlers.rs`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
