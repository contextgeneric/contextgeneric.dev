---
title: 'MatchWithFieldHandlers — match with the tag'
description: 'The convenience matcher that builds the per-variant handler list from an enum''s variants and passes each payload as a Field tagged with its variant name.'
sidebar_label: 'MatchWithFieldHandlers'
sidebar_position: 4
---

# `MatchWithFieldHandlers`

Match an enum by building the per-variant handler list from its own variants, passing each payload
with its variant name attached.

## Overview

`MatchWithFieldHandlers<Provider>` builds the per-variant list from the input enum's field list,
like [`MatchWithValueHandlers`](match_with_value_handlers.md), but hands each payload to `Provider`
as a [`Field<Tag, Value>`](../../types/field.md), where `Tag` is the variant's name as a type-level
string. A provider over `Field<Tag, Value>` can read that name, so one provider can report or route
on which variant it received, on a [**context**](/docs/reference/glossary#context), the type the
implementation runs against. `Provider` defaults to [`UseContext`](../use_context.md). Like every
CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`; unlike the value form, it is not in the prelude. It takes one
optional type parameter, the per-variant provider, and is wired directly to a handler component,
since it dispatches on the input type itself:

```rust
use cgp::extra::dispatch::MatchWithFieldHandlers;

delegate_components! {
    App {
        ComputerComponent: MatchWithFieldHandlers<Describe>,
    }
}
```

The borrowed form `MatchWithFieldHandlersRef<Provider>` matches `&Input` and hands the provider a
`Field<Tag, &Value>`. There is no `Mut` form, and the multi-argument forms are on
[`MatchFirstWithHandlers`](match_first_with_handlers.md#when-to-use-it).

## Examples

One provider describes every variant of two enums, reading the variant name from the tag:

```rust
use core::fmt::Display;
use cgp::prelude::*;
use cgp::core::field::traits::StaticString;
use cgp::extra::dispatch::MatchWithFieldHandlers;
use cgp::extra::handler::CanCompute;

#[derive(CgpData)]
pub enum Reading {
    Temperature(i32),
    Label(String),
}

#[derive(CgpData)]
pub enum Event {
    Started(u64),
    Stopped(bool),
}

#[cgp_computer]
pub fn describe<Tag, Value>(field: Field<Tag, Value>) -> String
where
    Tag: StaticString,
    Value: Display,
{
    format!("{}: {}", Tag::VALUE, field.value)
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: MatchWithFieldHandlers<Describe>,
    }
}

check_components! {
    App {
        ComputerComponent: [((), Reading), ((), Event)],
    }
}

pub fn demo() {
    let code = PhantomData::<()>;

    assert_eq!(
        App.compute(code, Reading::Temperature(21)),
        "Temperature: 21"
    );
    assert_eq!(
        App.compute(code, Reading::Label("north".to_owned())),
        "Label: north"
    );
    assert_eq!(App.compute(code, Event::Stopped(true)), "Stopped: true");
}
```

[`StaticString`](../../traits/formatting/static_string.md) recovers the variant name from the tag,
so `Describe` prints it without naming either enum, and the one wiring entry serves both. `App` is
an [environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Reach for `MatchWithFieldHandlers` when the per-variant handler needs the variant's name**, for
example to log which case ran or to look up a setting by name. When the handler wants only the
payload, which is the usual case, [`MatchWithValueHandlers`](match_with_value_handlers.md) is
simpler, and when different variants need different providers,
[`MatchWithHandlers`](match_with_handlers.md) takes a spelled-out list.

## Under the hood

`MatchWithFieldHandlers<Provider>` is a type alias over
[`UseInputDelegate`](../handler/use_input_delegate.md), whose table maps each input enum to
[`MatchWithHandlers`](match_with_handlers.md) over a list built from the enum:

```rust
pub type MatchWithFieldHandlers<Provider = UseContext> =
    UseInputDelegate<MatchWithFieldHandlersInputs<Provider>>;

delegate_components! {
    <Input: HasFieldHandlers<MapExtractFieldAndHandle<Provider>>, Provider>
    new MatchWithFieldHandlersInputs<Provider> {
        Input: MatchWithHandlers<Input::Handlers>
    }
}
```

Three traits build the list. `MapFieldHandler` maps a variant's `Tag` to the adapter that handles
it; `ToFieldHandlers` walks a [sum-type](../../types/either.md) field list and applies that map to
each variant; and `HasFieldHandlers` runs `ToFieldHandlers` over a type's
[`HasFields`](../../traits/shape/has_fields.md) list:

```rust
pub trait MapFieldHandler {
    type FieldHandler<Tag>;
}

impl<Tag, Value, RestFields, M> ToFieldHandlers<M> for Either<Field<Tag, Value>, RestFields>
where
    M: MapFieldHandler,
    RestFields: ToFieldHandlers<M>,
{
    type Handlers = Cons<M::FieldHandler<Tag>, RestFields::Handlers>;
}

impl<Provider> ToFieldHandlers<Provider> for Void {
    type Handlers = Nil;
}

impl<Context, Fields, M> HasFieldHandlers<M> for Context
where
    Context: HasFields<Fields = Fields>,
    Fields: ToFieldHandlers<M>,
{
    type Handlers = Fields::Handlers;
}
```

The map used here is `MapExtractFieldAndHandle<Provider>`, whose `FieldHandler<Tag>` is
[`ExtractFieldAndHandle`](extract_field_and_handle.md)`<Tag, Provider>`, so the list holds one
extract adapter per variant. Only the `Either`/`Void` list has impls, which is why the convenience
matchers apply to enums and not to structs. The first-argument aliases use
`MapExtractFirstFieldAndHandle` in the same place.

## Common Mistakes

**The mistakes of `MatchWithValueHandlers` apply unchanged.** The input must be an enum, and the
matcher answers only `Computer` and `AsyncComputer`;
[`MatchWithValueHandlers`](match_with_value_handlers.md#common-mistakes) shows both.

## Related constructs

- [`MatchWithValueHandlers`](match_with_value_handlers.md) — the sibling that strips the tag with
  [`HandleFieldValue`](handle_field_value.md).
- [`MatchWithHandlers`](match_with_handlers.md) — the matcher the list is built for.
- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the per-variant adapter in the list.
- [`Field`](../../types/field.md), [`StaticString`](../../traits/formatting/static_string.md) — the
  tagged payload and the trait that reads its name.
- [`UseInputDelegate`](../handler/use_input_delegate.md) — the input dispatcher this alias is built
  on.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing each variant of an enum to a handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern it serves.

## Source

- [`providers/matchers/match_with_field_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/matchers/match_with_field_handlers.rs),
  with the list machinery in
  [`to_field_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/matchers/to_field_handlers.rs).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
