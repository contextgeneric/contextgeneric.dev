---
sidebar_label: 'MatchWithFieldHandlers'
sidebar_position: 4
---

# `MatchWithFieldHandlers`

Match an enum by building the per-variant handler list automatically, passing each payload with its
variant tag still attached.

## Overview

`MatchWithFieldHandlers<Provider>` synthesizes the per-variant handler list from the input type's own
field list, like [`MatchWithValueHandlers`](match_with_value_handlers.md), but hands each matched
payload to `Provider` as a `Field<Tag, Value>` with the variant name still attached rather than as a
bare value. It runs on a **context**, the type a capability runs against. Reach for it when the handler
needs to know which variant it received. Like every CGP provider, it carries no runtime value.

The two matchers differ by exactly one [`HandleFieldValue`](handle_field_value.md) wrapper:
`MatchWithValueHandlers` wraps `Provider` in it to strip the tag, and `MatchWithFieldHandlers` does not.
`Provider` defaults to [`UseContext`](../use_context.md). The borrowed and multi-argument variants are
`MatchWithFieldHandlersRef` and the first-argument aliases `MatchFirstWithFieldHandlers`,
`MatchFirstWithFieldHandlersRef`, and `MatchFirstWithFieldHandlersMut`.

## Usage

Import it from `cgp::extra::dispatch`. It takes one optional type parameter, the per-variant provider.
Because it is itself a [`UseInputDelegate`](../handler/use_input_delegate.md) alias, it is wired
directly to a handler component and dispatches on the input type:

```rust
use cgp::extra::dispatch::MatchWithFieldHandlers;

delegate_components! {
    App {
        ComputerComponent: MatchWithFieldHandlers<FieldToString>,
    }
}
```

`FieldToString` receives each matched payload as a `Field<Tag, Value>`, so the variant tag stays
attached to it. For a handler that wants only the payload, use
[`MatchWithValueHandlers`](match_with_value_handlers.md).

## When to use it

**Reach for `MatchWithFieldHandlers` when the per-variant handler needs the variant tag**, for example
to log which case ran or to route further by name. When the handler wants only the payload, which is the common
case, [`MatchWithValueHandlers`](match_with_value_handlers.md) is simpler.

## Under the hood

`MatchWithFieldHandlers<Provider>` is a type alias over
[`UseInputDelegate`](../handler/use_input_delegate.md):

```rust
pub type MatchWithFieldHandlers<Provider = UseContext> =
    UseInputDelegate<MatchWithFieldHandlersInputs<Provider>>;
```

The per-variant list is built from a context's field list by three cooperating traits. `MapFieldHandler`
is a type-level function from a field's `Tag` to the adapter that should handle it; `ToFieldHandlers`
walks the [sum list](../../types/either.md) of a field list and applies that function to each
field, producing a [`Product!`](../../macros/product.md) list of adapters; and `HasFieldHandlers` reads a
context's [`HasFields`](../../traits/shape/has_fields.md) list and runs `ToFieldHandlers` over it:

```rust
impl<Context, Fields, M> HasFieldHandlers<M> for Context
where
    Context: HasFields<Fields = Fields>,
    Fields: ToFieldHandlers<M>,
{
    type Handlers = Fields::Handlers;
}
```

`ToFieldHandlers` is implemented inductively over the sum list: for `Either<Field<Tag, Value>, Rest>`
it produces `Cons<M::FieldHandler<Tag>, Rest::Handlers>`, and for the terminating `Void` it produces
`Nil`. The `MapFieldHandler` marker the crate supplies here is `MapExtractFieldAndHandle<Provider>`,
whose `FieldHandler<Tag>` is [`ExtractFieldAndHandle`](extract_field_and_handle.md)`<Tag, Provider>`, so
the matcher runs one extract adapter per variant without the user spelling out the list.
`MatchWithValueHandlers` differs only by wrapping `Provider` in
[`HandleFieldValue`](handle_field_value.md) first.

## Related constructs

- [`MatchWithValueHandlers`](match_with_value_handlers.md) — the sibling that strips the tag; the two
  differ by one `HandleFieldValue` wrapper.
- [`MatchWithHandlers`](match_with_handlers.md) — the explicit form the list expands to.
- [`ExtractFieldAndHandle`](extract_field_and_handle.md) — the per-variant adapter the list is built
  from.
- [`UseInputDelegate`](../handler/use_input_delegate.md) — the input dispatcher this alias is built on.
- [`HasFields`](../../traits/shape/has_fields.md) — the field list the handler list is synthesized from.

The ideas behind it:

- [Dispatching](/docs/concepts/dispatching) — routing each variant of an enum to a handler.
- [Extensible variants](/docs/concepts/extensible-variants) — the enum pattern it serves.

## Source

- [`providers/matchers/match_with_field_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/matchers/match_with_field_handlers.rs),
  with the field-to-handler machinery in `to_field_handlers.rs` and the first-argument forms in
  `match_first_with_field_handlers.rs`.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
