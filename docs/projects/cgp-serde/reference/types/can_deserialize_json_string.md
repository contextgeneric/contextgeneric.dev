---
title: 'CanDeserializeJsonString — a deserialize_json_string method in cgp-serde'
sidebar_label: 'CanDeserializeJsonString'
sidebar_position: 3
description: 'The cgp-serde trait that gives any suitably wired context a deserialize_json_string method, reading a value from a JSON string with no TryComputer entries.'
---

# `CanDeserializeJsonString`

Give a context a `deserialize_json_string` method that reads a value from a JSON string.

## Overview

`CanDeserializeJsonString<T>` is the shortest way to read JSON with a context. Any **context**, the
type whose wiring holds an application's choices, that can read `T` and has an error type gets the
method automatically, with no `TryComputer` entries to wire.

## Definition

```rust
#[cgp_fn(CanDeserializeJsonString)]
#[use_type(HasErrorType.Error)]
pub fn deserialize_json_string<T>(&self, serialized: &str) -> Result<T, Error>
where
    DeserializeFromJsonString: for<'a> TryComputer<Self, DeserializeJson<T>, &'a str, Output = T>,
{ ... }
```

[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) turns the function into a trait,
`CanDeserializeJsonString<T>`, implemented for every context that meets its bounds. It lives in
`cgp_serde_json::impls`.

## Usage

Import the trait, and call the method on a context, giving the target type on the result. The
method takes no turbofish, since `#[cgp_fn]` moves `T` onto the trait. From the repository's arena
test:

```rust
let deserialized: Payload<'_> = app.deserialize_json_string(&serialized).unwrap();
```

## Behavior

The method calls [`DeserializeFromJsonString`](../providers/deserialize_from_json_string.md)
directly, rather than through the context's `TryComputerComponent`, so the context needs no
`TryComputer` entries. It shares that provider's behavior: it rejects trailing input, turns
`serde_json` errors into the context's error type, and cannot produce a value that borrows from the
string.

## Context dependencies

A deserializer for `T` for every lifetime, and an error type that can be raised from a
`serde_json::Error`: `HasErrorType` and `CanRaiseError<serde_json::Error>`. A context without the
error components does not have the method; [the debugging
guide](../../guides/debugging-wiring.md#the-json-method-is-missing-its-error-wiring) shows the error
and the fix.

## Pairing

Writing JSON is done with [`SerializeToJsonString`](../providers/serialize_to_json_string.md)
through `try_compute`, or with `serde_json` and a
[`SerializeWithContext`](./serialize_with_context.md).

## When to use it

**Reach for `deserialize_json_string` to read an owned value from a string with the least wiring.**
To read from bytes or a stream, or to produce a value that borrows from the input, wire
[`DeserializeFromJsonReader`](../providers/deserialize_from_json_reader.md) instead.

## Related constructs

- [`DeserializeFromJsonString`](../providers/deserialize_from_json_string.md), the provider it
  calls.

## The ideas behind it

- [`#[cgp_fn]`](/docs/reference/macros/cgp_fn): a function turned into a trait any matching context
  implements.
- [Modular error handling](/docs/concepts/modular-error-handling): the error components it needs.

## Source

- [`crates/cgp-serde-json/src/impls/deserialize.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-json/src/impls/deserialize.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
