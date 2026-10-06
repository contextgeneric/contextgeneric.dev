---
title: 'SerializeRfc3339Date — encode a date as an RFC 3339 string in cgp-serde'
sidebar_label: 'SerializeRfc3339Date'
sidebar_position: 20
description: 'The cgp-serde provider that writes a DateTime<Utc> as an RFC 3339 string and reads one back with any offset.'
---

# `SerializeRfc3339Date`

Write a `DateTime<Utc>` as an RFC 3339 string, and read one back.

## Overview

`SerializeRfc3339Date` is one of the two date encodings in `cgp-serde-extra`. It formats a
[`chrono`](https://docs.rs/chrono) `DateTime<Utc>` as an RFC 3339 `String` and asks the **context**,
the type whose wiring holds the application's choices, to write it, and it reads a string through
the context and parses it. One struct implements both directions.

## Definition

```rust
pub struct SerializeRfc3339Date;

#[cgp_impl(SerializeRfc3339Date)]
#[uses(CanSerializeValue<String>)]
impl ValueSerializer<DateTime<Utc>> { ... }

#[cgp_impl(SerializeRfc3339Date)]
#[uses(CanDeserializeValue<'de, String>)]
impl<'de> ValueDeserializer<'de, DateTime<Utc>> { ... }
```

## Usage

Import it from `cgp_serde_extra::providers`. The first application in the
[`messages`](../../examples/messages.md) example writes its dates with it:

```rust
@ValueSerializerComponent.DateTime<Utc>:
    SerializeRfc3339Date,
```

## Behavior

Writing uses `chrono`'s `to_rfc3339`, with a `+00:00` offset and any sub-second part the value has:
14:15 on 3 November 2025 is written as `"2025-11-03T14:15:00+00:00"`. Reading accepts any RFC 3339
offset and converts to UTC, so `"2025-11-03T16:15:00+02:00"` reads as 14:15 UTC. Input that is not a
full RFC 3339 timestamp fails with `chrono`'s message; a bare date such as `"2025-11-03"` fails with
`premature end of input`.

## Context dependencies

`CanSerializeValue<String>` and `CanDeserializeValue<'de, String>`.

## Pairing

The same provider reads what it writes, with any offset normalized to UTC.

## When to use it

**Reach for `SerializeRfc3339Date` when dates should be readable and keep their precision.**
[`SerializeTimestamp`](./serialize_timestamp.md) writes the same instant as a number of seconds.

## Related constructs

- [`SerializeTimestamp`](./serialize_timestamp.md), the numeric date encoding.

## The ideas behind it

- [`messages`](../../examples/messages.md): RFC 3339 in one application and timestamps in another.

## Source

- [`crates/cgp-serde-extra/src/providers/date.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-extra/src/providers/date.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
