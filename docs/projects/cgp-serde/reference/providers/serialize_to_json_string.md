---
title: 'SerializeToJsonString — write JSON as an operation on the context in cgp-serde'
sidebar_label: 'SerializeToJsonString'
sidebar_position: 21
description: 'The cgp-serde provider that writes a value as a compact JSON string through the context''s wiring, turning serde_json errors into the context''s error type.'
---

# `SerializeToJsonString`

Write a value as a compact JSON string, as a fallible operation the context runs.

## Overview

`SerializeToJsonString` makes writing JSON something a **context**, the type whose wiring holds an
application's choices, can do through CGP's
[`TryComputer`](/docs/reference/components/handler/try_computer), an interface for a fallible
computation chosen by a marker type. It runs `serde_json` over the value with the context attached,
so the JSON follows the context's choices, and it turns a `serde_json::Error` into the context's own
error type.

## Definition

```rust
#[cgp_impl(new SerializeToJsonString)]
#[use_type(HasErrorType.Error)]
#[uses(CanSerializeValue<Value>, CanRaiseError<serde_json::Error>)]
impl<Code, Value> TryComputer<Code, &Value> {
    type Output = String;

    fn try_compute(&self, _code: PhantomData<Code>, value: &Value) -> Result<String, Error> { ... }
}
```

[`#[use_type]`](/docs/reference/attributes/use_type) brings the context's error type into scope as
`Error`, and [`#[uses]`](/docs/reference/attributes/uses) declares what the provider needs: a
serializer for the value, and a way to raise a `serde_json::Error`.

## Usage

Import it from `cgp_serde_json::providers`, and wire it under `TryComputerComponent`, keyed on the
marker `SerializeJson` from `cgp_serde_json::code`. The context also needs an error type. From the
[`basic`](../../examples/basic.md) example:

```rust
open {
    ValueSerializerComponent,
    ValueDeserializerComponent,
    TryComputerComponent,
};

ErrorTypeProviderComponent:
    UseAnyhowError,

ErrorRaiserComponent:
    RaiseAnyhowError,

@TryComputerComponent.SerializeJson:
    SerializeToJsonString,
```

A caller then runs it with `try_compute`, from `cgp::extra::handler::CanTryCompute`:

```rust
let serialized = context
    .try_compute(PhantomData::<SerializeJson>, &value)
    .unwrap();
```

## Behavior

The provider calls `serde_json::to_string` on the value wrapped with the context in
[`SerializeWithContext`](../types/serialize_with_context.md), so the output is compact JSON shaped
by the context's wiring. A `serde_json::Error`, including one a provider raised during the
traversal, is turned into the context's error type with `raise_error`. The provider accepts any
marker type, so `SerializeJson` is a convention rather than a requirement.

## Context dependencies

`CanSerializeValue<Value>` for the value, and an error type that can be raised from a
`serde_json::Error`: `HasErrorType` and `CanRaiseError<serde_json::Error>`.

## Pairing

[`DeserializeFromJsonString`](./deserialize_from_json_string.md) and
[`DeserializeFromJsonReader`](./deserialize_from_json_reader.md) read JSON back.

## When to use it

**Reach for `SerializeToJsonString` when writing JSON should return the context's own error type**,
or be one of the operations a context's wiring chooses. For pretty-printed output or writing to a
stream, call `serde_json` with a [`SerializeWithContext`](../types/serialize_with_context.md)
directly, which returns `serde_json`'s error; see [using a format](../../guides/formats.md).

## Related constructs

- [`SerializeWithContext`](../types/serialize_with_context.md), the adapter the provider passes to
  `serde_json`.

## The ideas behind it

- [Modular error handling](/docs/concepts/modular-error-handling): how a provider raises into the
  context's error type.
- [Handlers](/docs/concepts/handlers): CGP's computation interfaces, including `TryComputer`.

## Source

- [`crates/cgp-serde-json/src/providers/to_string.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-json/src/providers/to_string.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
