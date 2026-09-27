---
sidebar_label: 'Re-entrant providers'
sidebar_position: 3
description: 'How a cgp-serde provider hands each nested value back to the context, so one wiring choice reaches every level of a value.'
---

# Re-entrant providers

When an application wires `Vec<u8>` to hex, how does that choice reach a byte field three structs
deep, when none of the providers in between knows about it? [cgp-serde](../index.md) rebuilds
Serde's `Serialize` and `Deserialize` as components of [CGP](/docs/), and its providers never encode
the values nested inside the one they are given. They hand each one back to the **context**, the
type whose wiring holds an application's choices, which is called re-entering it. This page explains
the two ways a provider re-enters the context, and what that asks of the context's wiring.

## A provider asks the context, not the next impl

In Serde, a struct's `Serialize` impl calls each field's own `Serialize` impl, so a `Vec<u8>` field
is written however Serde's impl for `Vec<u8>` writes it. The choice is fixed where that impl was
written.

A cgp-serde provider makes the same call on the context instead. The struct provider,
[`SerializeFields`](../reference/providers/serialize_fields.md), serializes each field by asking the
context to serialize the field's type. The collection provider,
[`SerializeIterator`](../reference/providers/serialize_iterator.md), does the same for each item.
The context answers each request from its wiring table, so a context that wires `Vec<u8>` to hex
gets hex for every `Vec<u8>` it serializes: a top-level value, a field, or an item in a list inside
a field.

This also keeps each provider small. `SerializeFields` knows how to walk a struct and nothing about
the fields' types, and `SerializeIterator` knows how to walk a collection and nothing about the
items. Each is written once, generically, and the context composes them.

## Re-entering directly

A provider that converts a value and encodes the result calls the context directly.
[`SerializeHex`](../reference/providers/serialize_hex.md) turns bytes into a hex string, then asks
the context to serialize the string:

```rust
#[cgp_impl(SerializeHex)]
#[uses(CanSerializeValue<String>)]
impl<Value> ValueSerializer<Value>
where
    Value: ToHex,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str_value = value.encode_hex::<String>();
        self.serialize(&str_value, serializer)
    }
}
```

Inside the impl, `self` is the context. [`#[uses]`](/docs/reference/attributes/uses) declares what
the provider needs from it, here the ability to serialize a `String`, and `self.serialize` is the
call. The requirement is resolved from the context's own wiring, which is CGP's
[impl-side dependencies](/docs/concepts/impl-side-dependencies). So even the string a hex encoding
produces is written the way the context says strings are written.

## Re-entering through an adapter

A provider that hands values to Serde needs another route. Serde's compound writers, such as the one
behind `serialize_seq`, take each element as a value implementing Serde's `Serialize`, and there is
no place to pass a context. So the provider wraps the element and the context together in
[`SerializeWithContext`](../reference/types/serialize_with_context.md), an adapter whose `Serialize`
impl calls the context. This is `SerializeIterator`:

```rust
#[cgp_impl(new SerializeIterator)]
impl<Value> ValueSerializer<Value>
where
    for<'a> &'a Value: IntoIterator,
    Self: for<'a> CanSerializeValue<<&'a Value as IntoIterator>::Item>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let items = value.into_iter();
        let mut serializer = serializer.serialize_seq(None)?;
        for item in items {
            serializer.serialize_element(&SerializeWithContext {
                context: self,
                value: &item,
            })?
        }

        serializer.end()
    }
}
```

The `where` clause says the context must be able to serialize whatever iterating the collection
yields. It is written there rather than in `#[uses]` because the item type depends on the lifetime
of the borrow. Deserializing works the same way in the other direction, with
[`DeserializeWithContext`](../reference/types/deserialize_with_context.md) as the adapter that
Serde's readers accept.

## What the context must wire

Because every nested value goes back through the context, the context needs an entry for every type
the traversal reaches, which is more than the types in the data. Three kinds are easy to miss:

- **Collections.** A `Vec<EncryptedMessage>` field needs an entry of its own, `SerializeIterator`,
  besides the entry for `EncryptedMessage`.
- **References.** `SerializeIterator` walks a collection by reference, so a `Vec<EncryptedMessage>`
  yields `&EncryptedMessage`, and the context must serialize that. One generic entry covers every
  reference, forwarding it to the value behind it:

  ```rust
  @ValueSerializerComponent.<'a, T> &'a T:
      SerializeDeref,
  ```

- **What an encoding produces.** Hex, base64, and RFC 3339 dates each produce a `String` and ask the
  context to serialize it, and Unix timestamps produce an `i64`. A context wires those types even
  when no field has them.

Each provider's reference page says what it asks the context for. The rule of thumb is to start from
each top-level type and follow what each provider asks for until every request lands on a provider,
such as [`UseSerde`](../reference/providers/use_serde.md), that asks for nothing. The [wiring
guide](../guides/wiring-a-context.md) turns this into steps.

## Mistakes are found at compile time

The compiler follows these requests while it type-checks the program, so none of the traversal is
looked up at run time, and a type with no entry is a compile error rather than a failure while
serializing. The error names the whole path from the top-level type down to the missing entry,
which [the debugging guide](../guides/debugging-wiring.md) shows how to read.

The same compile-time resolution rules out two shapes. A provider that re-enters the context for the
very type it handles, such as `String` wired to a provider that formats a value into a `String` and
serializes that, asks for itself forever, and the compiler reports that the lookup never resolves.
And a **recursive data type**, such as a tree node whose children are nodes, cannot be serialized by
the generic providers. Serializing the node requires serializing its children, which requires
serializing the node, and Rust's trait solver treats that cycle as an error, even with every entry
present. Such a type needs a provider written for it, one that walks the recursion itself and asks
the context only for the parts that are not recursive.

## What it costs

Re-entry is what makes one wiring line decide a type's encoding everywhere, and it is also why a
context's table is long: every type the traversal passes through is a question the table must
answer, including references and in-between strings that never appear in the data. The table is
checked completely by the compiler, but a reader has to understand the traversal to know why an
entry such as `i64` is there.

A context also cannot make an exception for one place. Every `Vec<u8>` a context serializes gets the
same encoding, so two fields of the same type in one struct cannot be encoded differently, short of
giving one of them a distinct type.

## Where to go next

- [Derive-free records](./derive-free-records.md): the struct providers that re-enter for each
  field.
- [`messages`](../examples/messages.md): re-entry through three levels of nesting, and the `i64`
  entry left out.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the CGP idea behind a provider
  asking its context for what it needs.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
