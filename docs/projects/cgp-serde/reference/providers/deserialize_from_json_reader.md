---
title: 'DeserializeFromJsonReader — read JSON from any serde_json reader in cgp-serde'
sidebar_label: 'DeserializeFromJsonReader'
sidebar_position: 20
description: 'The cgp-serde provider that reads a value from a string, bytes, or a stream through serde_json and the context''s wiring, rejecting trailing input.'
---

# `DeserializeFromJsonReader`

Read a value from any `serde_json` reader through the context, and reject anything after it.

## Overview

`DeserializeFromJsonReader` makes reading JSON an operation the **context**, the type whose wiring
holds an application's choices, runs through CGP's
[`TryComputer`](/docs/reference/components/handler/try_computer). It reads from any of
`serde_json`'s input sources: a string, a byte slice, or a stream. It is keyed on the marker type
`DeserializeJson<T>`, from `cgp_serde_json::code`, which names the type to read.

## Definition

```rust
#[cgp_impl(new DeserializeFromJsonReader)]
#[use_type(HasErrorType.Error)]
#[uses(CanDeserializeValue<'de, Value>, CanRaiseError<serde_json::Error>)]
impl<Value, R, 'de> TryComputer<DeserializeJson<Value>, R>
where
    R: Read<'de>,
{
    type Output = Value;

    fn try_compute(
        &self,
        _code: PhantomData<DeserializeJson<Value>>,
        source: R,
    ) -> Result<Value, Error> { ... }
}
```

`Read` is `serde_json::de::Read`, implemented by `serde_json`'s `StrRead`, `SliceRead`, and
`IoRead`. The marker types are defined as:

```rust
pub struct DeserializeJson<T>(pub PhantomData<T>);
pub struct SerializeJson;
```

## Usage

Import it from `cgp_serde_json::providers`, and wire it under `TryComputerComponent` for every
target type with one generic entry:

```rust
@TryComputerComponent.<T> DeserializeJson<T>:
    DeserializeFromJsonReader,
```

A caller passes a reader:

```rust
let payload: Payload = app
    .try_compute(PhantomData::<DeserializeJson<Payload>>, StrRead::new(&text))?;
```

## Behavior

The provider builds a `serde_json::Deserializer` over the reader, reads the value through the
context, and then checks that nothing follows it, so `"a" x` fails with
`trailing characters at line 1 column 5`. Every `serde_json` error is turned into the context's
error type with `raise_error`.

The reader's lifetime flows through to the value's deserializer, so a reader over a string or a byte
slice can produce a value that borrows from the input, such as a `&str`. A stream cannot lend its
input, so a value that borrows from it cannot be read from a stream, as with plain `serde_json`.

## Context dependencies

`CanDeserializeValue<'de, Value>` for the target type, and an error type that can be raised from a
`serde_json::Error`: `HasErrorType` and `CanRaiseError<serde_json::Error>`.

## Pairing

[`SerializeToJsonString`](./serialize_to_json_string.md) writes JSON.

## When to use it

**Reach for `DeserializeFromJsonReader` to read JSON from bytes or a stream, or to produce a value
that borrows from its input.** For a string,
[`DeserializeFromJsonString`](./deserialize_from_json_string.md) takes the string directly and wraps
it in a reader itself.

## Related constructs

- [`DeserializeWithContext`](../types/deserialize_with_context.md), the seed the provider drives.
- [`CanDeserializeJsonString`](../types/can_deserialize_json_string.md), a method for reading a
  string with no `TryComputer` entry.

## The ideas behind it

-  [The bridge to
  Serde](../../architecture/serde-bridge.md#going-out-handing-a-context-to-a-serde-api): why reading
  needs a seed and a format's own deserializer.
- [Modular error handling](/docs/concepts/modular-error-handling): raising into the context's error
  type.

## Source

- [`crates/cgp-serde-json/src/providers/from_reader.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-json/src/providers/from_reader.rs)
- [`crates/cgp-serde-json/src/code/`](https://github.com/contextgeneric/cgp-serde/tree/main/crates/cgp-serde-json/src/code)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
