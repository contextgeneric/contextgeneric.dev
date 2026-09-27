---
title: 'SerializeTimestamp — encode a date as a Unix timestamp in cgp-serde'
sidebar_label: 'SerializeTimestamp'
sidebar_position: 18
description: 'The cgp-serde provider that writes a DateTime<Utc> as whole seconds since the Unix epoch and reads one back, asking the context to write and read the i64.'
---

# `SerializeTimestamp`

Write a `DateTime<Utc>` as a Unix timestamp in whole seconds, and read one back.

## Overview

`SerializeTimestamp` is the numeric date encoding in `cgp-serde-extra`. It converts a
[`chrono`](https://docs.rs/chrono) `DateTime<Utc>` to the number of seconds since the Unix epoch, an
`i64`, and asks the **context**, the type whose wiring holds the application's choices, to write
that number. Because it goes through the context, a context that uses it needs an entry for `i64`,
even when no field in its data is an `i64`.

## Definition

```rust
pub struct SerializeTimestamp;

#[cgp_impl(SerializeTimestamp)]
#[uses(CanSerializeValue<i64>)]
impl ValueSerializer<DateTime<Utc>> { ... }

#[cgp_impl(SerializeTimestamp)]
#[uses(CanDeserializeValue<'de, i64>)]
impl<'de> ValueDeserializer<'de, DateTime<Utc>> { ... }
```

## Usage

Import it from `cgp_serde_extra::providers`, and wire `i64` beside it. The second application in the
[`messages`](../../examples/messages.md) example does both:

```rust
@ValueSerializerComponent.[
    i64,
    u64,
    String,
]:
    UseSerde,

@ValueSerializerComponent.DateTime<Utc>:
    SerializeTimestamp,
```

## Behavior

Writing writes the whole seconds since the epoch through the context's entry for `i64`, so 14:15 UTC
on 3 November 2025 is written as `1762179300`. Any part of a second is dropped, so a value with
milliseconds is written as the same number as the whole second before it. Reading reads an `i64` and
builds the instant; a number outside the range `chrono` can represent fails with
`invalid timestamp`.

## Context dependencies

`CanSerializeValue<i64>` and `CanDeserializeValue<'de, i64>`. Leaving out the `i64` entry is the
mistake the [`messages`](../../examples/messages.md#try-a-change) example demonstrates.

## Pairing

The same provider reads what it writes, for instants with no part of a second.

## When to use it

**Reach for `SerializeTimestamp` when dates should be compact numbers and whole seconds are precise
enough.** For sub-second precision, or dates a person should read,
[`SerializeRfc3339Date`](./serialize_rfc3339_date.md) keeps both.

## Related constructs

- [`SerializeRfc3339Date`](./serialize_rfc3339_date.md), the string date encoding.

## The ideas behind it

- [Re-entrant providers](../../architecture/reentrant-providers.md#what-the-context-must-wire): why
  a context wires `i64` for it.

## Source

- [`crates/cgp-serde-extra/src/providers/timestamp.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-extra/src/providers/timestamp.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
