---
title: 'DeserializeFromJsonString — read JSON from a string in cgp-serde'
sidebar_label: 'DeserializeFromJsonString'
sidebar_position: 23
description: 'The cgp-serde provider that reads a value from any string-like input by wrapping it in a serde_json reader and handing it to an inner provider.'
---

# `DeserializeFromJsonString`

Read a value from a string by wrapping it in a `serde_json` reader and handing it to an inner
provider.

## Overview

`DeserializeFromJsonString` saves a caller from building a reader. It is wired in the table of a
**context**, the type whose wiring holds an application's choices, and accepts a `String`, a `&str`,
or any `AsRef<str>`, wraps it in `serde_json`'s string reader, and passes it to an inner provider,
[`DeserializeFromJsonReader`](./deserialize_from_json_reader.md) unless another is named. It is a
small [higher-order provider](/docs/concepts/higher-order-providers), parameterized by the provider
it delegates to.

## Definition

```rust
pub struct DeserializeFromJsonString<InDeserializer = DeserializeFromJsonReader>(
    pub PhantomData<InDeserializer>,
);

#[cgp_impl(DeserializeFromJsonString<InDeserializer>)]
#[use_type(HasErrorType.Error)]
impl<Code, Value, S, InDeserializer> TryComputer<Code, S>
where
    InDeserializer: for<'a> TryComputer<Self, Code, StrRead<'a>, Output = Value>,
    S: AsRef<str>,
{
    type Output = Value;

    fn try_compute(&self, code: PhantomData<Code>, source: S) -> Result<Value, Error> { ... }
}
```

## Usage

Import it from `cgp_serde_json::providers`, and wire it under `TryComputerComponent` for every
target type. From the [`basic`](../../examples/basic.md) example:

```rust
@TryComputerComponent.<T> DeserializeJson<T>:
    DeserializeFromJsonString,
```

A caller passes the string, here a `&String`:

```rust
let deserialized: Payload = context
    .try_compute(PhantomData::<DeserializeJson<Payload>>, &serialized)
    .unwrap();
```

## Behavior

The provider wraps the string in a `StrRead` and calls the inner provider with it, which reads the
value and checks for trailing input. Because it must work for a reader of any lifetime, the value it
produces cannot borrow from the input string. A value that borrows from elsewhere is unaffected: the
repository's arena test reads a `Payload<'a>` that borrows from the context's arena through this
provider.

## Context dependencies

Whatever the inner provider requires. For the default, that is a deserializer for the target type
for every lifetime, and an error type that can be raised from a `serde_json::Error`.

## Pairing

[`SerializeToJsonString`](./serialize_to_json_string.md) writes JSON.

## When to use it

**Reach for `DeserializeFromJsonString` to read an owned value from a string.** For a value that
borrows from its input, wire [`DeserializeFromJsonReader`](./deserialize_from_json_reader.md) and
pass it a string reader directly.

## Related constructs

- [`CanDeserializeJsonString`](../types/can_deserialize_json_string.md), a method that calls this
  provider without a `TryComputer` entry.

## The ideas behind it

- [Higher-order providers](/docs/concepts/higher-order-providers): a provider that delegates to
  another named in its type.

## Source

- [`crates/cgp-serde-json/src/providers/from_str.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-json/src/providers/from_str.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
