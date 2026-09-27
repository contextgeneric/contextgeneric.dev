---
sidebar_label: 'Writing a provider'
sidebar_position: 2
description: 'How to write a new cgp-serde provider: one struct for both directions, nested values handed back to the context, and errors reported through Serde.'
---

# Writing a provider

A new encoding in [cgp-serde](../index.md) is a new provider: a type that implements one or both of
the library's serialization components, which a **context**, the type whose wiring holds an
application's choices, then names in its wiring. cgp-serde rebuilds Serde's `Serialize` and
`Deserialize` as components of [CGP](/docs/). This guide shows a provider pair, then how to
structure one, how to hand nested values back to the context, and how to report errors.

## Start from an example

This provider writes a `Duration` as a whole number of milliseconds. It converts to an in-between
type, `u64`, and asks the context to encode that, so how the number is finally written stays the
context's choice:

```rust
use core::time::Duration;

use cgp::prelude::*;
use cgp_serde::components::{
    CanDeserializeValue, CanSerializeValue, ValueDeserializer, ValueDeserializerComponent,
    ValueSerializer, ValueSerializerComponent,
};
use serde::ser::Error;

pub struct SerializeMillis;

#[cgp_impl(SerializeMillis)]
#[uses(CanSerializeValue<u64>)]
impl ValueSerializer<Duration> {
    fn serialize<S>(&self, value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let millis = u64::try_from(value.as_millis()).map_err(S::Error::custom)?;
        self.serialize(&millis, serializer)
    }
}

#[cgp_impl(SerializeMillis)]
#[uses(CanDeserializeValue<'de, u64>)]
impl<'de> ValueDeserializer<'de, Duration> {
    fn deserialize<D>(&self, deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let millis = self.deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
```

Inside each impl, `self` is the context. A context wires the provider like any other, beside an
entry for `u64` in each direction:

```rust
delegate_components! {
    App {
        open { ValueSerializerComponent, ValueDeserializerComponent };
        @ValueSerializerComponent.u64: UseSerde,
        @ValueSerializerComponent.Duration: SerializeMillis,
        @ValueDeserializerComponent.u64: UseSerde,
        @ValueDeserializerComponent.Duration: SerializeMillis,
    }
}
```

A `Duration` of 1,500 milliseconds is then written to JSON as `1500` and read back unchanged.

## Structure the provider

**Implement both directions on one struct when they are one decision.** Declare the struct once,
then write each direction as its own [`#[cgp_impl]`](/docs/reference/macros/cgp_impl) naming it, as
above. A context names the same provider in both tables, and the two impls sit together, so they are
written to agree. Use `#[cgp_impl(new Name)]` only for a provider with one direction, since `new`
declares the struct, and a second `new` would declare it twice.

**Follow the library's naming.** Name a provider that implements both directions, or only writing,
`Serialize…`, and one that implements only reading `Deserialize…`; see [component
design](../architecture/component-design.md#one-provider-can-serve-both-directions).

**Write the impl in the consumer-trait shape**, `impl<Value> ValueSerializer<Value>` with `self` as
the context, as the library's own providers do. `#[cgp_impl]` fills in the context parameter.

## Hand nested values back to the context

**Encode anything that is not a leaf through the context.** A provider that writes a nested value
itself, by calling Serde's impl or writing the bytes directly, fixes that value's encoding for every
application that uses the provider. Asking the context keeps it a choice, which is the property the
whole library rests on; see [re-entrant providers](../architecture/reentrant-providers.md). There
are two ways, and the choice depends on who makes the nested call:

- **Converting to another type.** Call `self.serialize(&converted, serializer)` or
  `self.deserialize(deserializer)` yourself, and declare what you need with
  [`#[uses]`](/docs/reference/attributes/uses), as `SerializeMillis` does for `u64`.
- **Passing items to one of Serde's compound writers or readers.** Wrap each item in
  [`SerializeWithContext`](../reference/types/serialize_with_context.md) for `serialize_element` or
  `serialize_entry`, or pass a
  [`DeserializeWithContext`](../reference/types/deserialize_with_context.md) seed to
  `next_element_seed` or `next_value_seed`.

When the nested type depends on the lifetime of a borrow, as an iterator's items do, write the
requirement as a higher-ranked bound in the `where` clause, `Self: for<'a> CanSerializeValue<…>`,
since `#[uses]` does not express one.

**Never ask the context for the type you are handling.** A provider for `String` that asks the
context to serialize a `String` depends on itself, and the context fails to compile. When a provider
needs another provider for the same type, take it as a type parameter, as
[`DeserializeDefault`](../reference/providers/deserialize_default.md) does, and bind it with
[`#[use_provider]`](/docs/reference/attributes/use_provider).

## Report errors through Serde

**Report a failure with the format's own error**, not with CGP's error components. Map it with
`S::Error::custom` or `D::Error::custom`, from `serde::ser::Error` and `serde::de::Error`, using any
error that implements `Display`, as `SerializeMillis` does for a duration too long for a `u64`. The
format adds its position to the message, and the error reaches the caller the way a failure inside
Serde's own impls does. CGP's [error handling](/docs/concepts/modular-error-handling) belongs at the
edge, where the JSON providers turn the finished `serde_json::Error` into the context's error type;
see [the bridge to Serde](../architecture/serde-bridge.md#where-errors-are-reported).

## Make it work with every input a format sends

A few habits keep a provider working across formats and inputs:

- **Ask the context for owned in-between types.** Ask for a `String` rather than a `&'de str` unless
  the result must borrow from the input. A format can lend a string only when it is stored in the
  input as is, so a JSON string with an escape in it, or any input read from a stream, has no
  `&'de str` to lend.
- **Accept owned data in a visitor.** A Serde visitor that implements `visit_borrowed_bytes` should
  also implement `visit_bytes` and `visit_byte_buf`, and likewise for strings, so that input the
  format had to copy is accepted.
- **Declare lengths when they are known.** Pass `Some(len)` to `serialize_seq` or `serialize_map`
  when the value knows its length, so length-prefixed binary formats can write it.
- **Make the two directions agree.** Check that what the serializing half writes is what the
  deserializing half reads, in each format you target.

## Test the provider

Wire the provider on a test context, assert the wiring with
[`check_components!`](/docs/reference/macros/check_components), listing the reading side with
[`Life<'de>`](/docs/reference/types/life), and round-trip at least one value through a real format.

## Where to go next

- [Wiring a context](./wiring-a-context.md): wiring the new provider beside the library's.
- [Re-entrant providers](../architecture/reentrant-providers.md): the two ways a provider hands a
  nested value back.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the CGP idea behind `#[uses]`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
