---
sidebar_label: 'Debugging the wiring'
sidebar_position: 4
description: 'The common cgp-serde wiring mistakes, each with the code that makes it, the error cargo cgp check reports, and the fix.'
---

# Debugging the wiring

Most compile errors in a [cgp-serde](../index.md) **context**, the type whose wiring holds an
application's choices, come from a handful of wiring mistakes, and each has an error that is easy to
recognize once seen. cgp-serde rebuilds Serde's `Serialize` and `Deserialize` as components of
[CGP](/docs/). This guide shows each mistake with the code that makes it, the error, and the fix.

The errors below come from [`cargo cgp check`](/docs/cargo-cgp/check), CGP's error tool, run in
place of `cargo check`. `cargo cgp check` leads with the root cause for the classes it recognizes,
and the tool does not yet reshape every class. Each mistake is caught by a
[`check_components!`](/docs/reference/macros/check_components) table listing the context's types,
as [wiring a context](./wiring-a-context.md#5-check-every-value-type) recommends, and the errors are
shorter and clearer there than at the first call.

## A type the traversal reaches has no entry

**The most common mistake is a missing entry for a nested type.** Here `Payload` is wired to
[`SerializeRecordFields`](../reference/providers/serialize_record_fields.md), but the type of its `data` field
has no entry:

```rust
#[derive(CgpData)]
pub struct Payload {
    pub quantity: u64,
    pub data: Vec<u8>,
}

pub struct App;

delegate_components! {
    App {
        open ValueSerializerComponent;
        @ValueSerializerComponent.u64: UseSerde,
        @ValueSerializerComponent.Payload: SerializeRecordFields,
    }
}

check_components! {
    App {
        ValueSerializerComponent: [u64, Payload],
    }
}
```

The check fails on `Payload`, and the tool names the missing entry, then the path that leads to it:

```text
error[E0277]: [CGP-E001] the consumer trait `CanSerializeValue<Payload>` is not implemented for context `App`
   = note: root cause: [CGP-E107] context `App` does not contain any delegate entry for `@ValueSerializerComponent.Vec<u8>`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanSerializeValue<Payload>` for context `App`
             └─ [CGP-E104] redirect lookup to `@ValueSerializerComponent` in `App`
               └─ [CGP-E102] provider trait impl `ValueSerializer<Payload>` with context `App` for provider `SerializeRecordFields`
                 └─ [CGP-E105] trait impl `FieldsSerializer` for `Struct! { quantity: u64, data: Vec<u8> }`
                   └─ [CGP-E105] trait impl `FieldsSerializer` for `Struct! { data: Vec<u8> }`
                     └─ [CGP-E101] consumer trait impl `CanSerializeValue<Vec<u8>>` for context `App`
                       └─ [CGP-E104] redirect lookup to `@ValueSerializerComponent` in `App`
                         └─ [CGP-E107] context `App` does not contain any delegate entry for `@ValueSerializerComponent.Vec<u8>`
```

The chain reads from the top down: serializing `Payload` goes to `SerializeRecordFields`, which walks the
fields to `data`, which asks the context for a `Vec<u8>`, which the table does not have. However
deep the missing type sits, the root cause names it. The fix is an entry for `Vec<u8>`, such as
`@ValueSerializerComponent.Vec<u8>: SerializeHex`.

## An entry for a type no field has

The same error, naming a different type, covers three entries that are easy to miss because no field
in the data has the type.

**A reference, for `SerializeIterator`.** Wiring a `Vec<u64>` to
[`SerializeIterator`](../reference/providers/serialize_iterator.md) with only a `u64` entry fails,
because walking the vector by reference yields `&u64`:

```text
error[E0277]: [CGP-E001] the consumer trait `CanSerializeValue<Vec<u64>>` is not implemented for context `Ctx`
   = note: root cause: [CGP-E107] context `Ctx` does not contain any delegate entry for `@ValueSerializerComponent.&u64`
```

The fix is the generic reference entry, `@ValueSerializerComponent.<'a, T> &'a T: SerializeDeref`,
which covers every reference at once.

**What an encoding produces.** Wiring `DateTime<Utc>` to
[`SerializeTimestamp`](../reference/providers/serialize_timestamp.md) without an `i64` entry fails
with a missing `@ValueSerializerComponent.i64`, because the timestamp is serialized through the
context. Hex, base64, and RFC 3339 dates need `String` the same way. The
[`messages`](../examples/messages.md#try-a-change) example shows this case in full.

**A service a provider takes from the context.** Wiring `&'b Coord` to
[`DeserializeAndAllocate`](../reference/providers/deserialize_and_allocate.md) without an entry for
`AllocatorComponent` fails, because the provider allocates through the context. In the repository's
arena test, with the allocator entry removed, the check on the borrowed types reports:

```text
error[E0277]: [CGP-E001] the consumer traits `CanDeserializeValue<&Coord>` and `CanDeserializeValue<Payload<'_>>` are not implemented for context `App<'_>`
   = note: root cause: [CGP-E107] context `App<'_>` does not contain any delegate entry for `AllocatorComponent`
```

The fix is the entry `AllocatorComponent: AllocateWithArena`, or another allocation provider; see
[context services](../architecture/context-services.md).

## A reading check leaves out the lifetime

**A check on the deserializing component without `Life<'de>` reports a provider that does not
implement the component, rather than a missing entry.** Here `u64` is wired, and the check still
fails:

```rust
check_components! {
    App {
        ValueDeserializerComponent: u64,
    }
}
```

```text
error[E0277]: [CGP-E002] the provider trait `ValueDeserializer<u64>` with context `App` is not implemented for provider `RedirectLookup<App, @ValueDeserializerComponent>`
```

The clue is further down the error, in the list of what the provider does implement, which includes
`IsProviderFor<ValueDeserializerComponent, __Context__, (Life<'_>, Value)>`. The component's
parameters are a lifetime and a type, and the check gave only the type. The fix is to write the
entry as `(Life<'de>, u64)`, with `<'de>` declared on the table:

```rust
check_components! {
    <'de> App {
        ValueDeserializerComponent: (Life<'de>, u64),
    }
}
```

## The JSON method is missing its error wiring

**`deserialize_json_string` needs the context's error components**, and a context without them
cannot call it:

```rust
pub struct Ctx;

delegate_components! {
    Ctx {
        open ValueDeserializerComponent;
        @ValueDeserializerComponent.u64: UseSerde,
    }
}

let r: Result<u64, _> = Ctx.deserialize_json_string("1");
```

Plain `cargo check` reports only that the method exists for `Ctx` but its trait bounds were not
satisfied. `cargo cgp check` names what is missing:

```text
error[E0599]: [CGP-E009] the trait `CanDeserializeJsonString<_>` is not implemented for context `Ctx`
   = note: root causes:
             - [CGP-E107] context `Ctx` does not contain any delegate entry for `@ValueDeserializerComponent`
             - [CGP-E107] context `Ctx` does not contain any delegate entry for `ErrorTypeProviderComponent`
             - [CGP-E107] context `Ctx` does not contain any delegate entry for `ErrorRaiserComponent`
```

The two error entries are the real causes. The first cause appears because the call failed before
the compiler worked out the target type, so the tree asks for an unknown type; it goes away once
the error entries are wired. The fix is the two entries from [wiring a
context](./wiring-a-context.md#4-wire-an-error-type-when-the-json-providers-are-used).

## A provider depends on itself

**Wiring a type to a provider that asks the context for the same type makes the lookup circular.**
[`SerializeWithDisplay`](../reference/providers/serialize_with_display.md) formats a value into a
`String` and asks the context to serialize the `String`, so wiring `String` itself to it leads
straight back:

```rust
delegate_components! {
    Ctx {
        open ValueSerializerComponent;
        @ValueSerializerComponent.String: SerializeWithDisplay,
    }
}

check_components! {
    Ctx {
        ValueSerializerComponent: String,
    }
}
```

The check reports that the wiring never resolves:

```text
error[E0275]: [CGP-E010] the wiring for the consumer trait `CanSerializeValue<String>` on context `Ctx` never resolves — the lookup recurses without terminating
```

The help line printed below it suggests a component delegated back to the context itself, as with
`UseContext`, which is the usual cause of this class; here the cycle runs through the provider
instead, and the headline's consumer trait names the type to look at. Without the check, the first
call reports the compiler's raw overflow error instead, `E0275`,
"overflow evaluating the requirement", naming the adapter that was passed to `serde_json`. The fix
is a provider for the in-between type that asks for nothing, such as
[`UseSerde`](../reference/providers/use_serde.md) or
[`SerializeString`](../reference/providers/serialize_string.md) for `String`.

## The data type is recursive

**A type that contains itself reports the same error, even with every entry present.** A tree node
whose children are nodes, wired in full:

```rust
#[derive(CgpData)]
pub struct Node {
    pub id: u64,
    pub children: Vec<Node>,
}

delegate_components! {
    Ctx {
        open ValueSerializerComponent;
        @ValueSerializerComponent.<'a, T> &'a T: SerializeDeref,
        @ValueSerializerComponent.u64: UseSerde,
        @ValueSerializerComponent.Vec<Node>: SerializeIterator,
        @ValueSerializerComponent.Node: SerializeRecordFields,
    }
}
```

```text
error[E0275]: [CGP-E010] the wiring for the consumer trait `CanSerializeValue<Node>` on context `Ctx` never resolves — the lookup recurses without terminating
```

Serializing a `Node` requires serializing its `Vec<Node>`, which requires `&Node`, which requires
`Node` again, and the compiler treats that cycle as an error. No entry fixes it. The type needs a
provider written for it, one that walks the recursion itself and asks the context only for the
parts that are not recursive; see [writing a provider](./writing-a-provider.md) and [re-entrant
providers](../architecture/reentrant-providers.md#mistakes-are-found-at-compile-time).

## A key does not parse

**A key the wiring syntax cannot read fails inside the macro, before any trait is checked.** The
case serialization runs into is a fixed-size array:

```rust
@ValueDeserializerComponent.[u8; 32]: UseSerde,
```

```text
error: expected `,`
```

Square brackets are the syntax for a list of keys, so the semicolon is read as a broken list. Key
the array on a type alias instead, as [wiring a
context](./wiring-a-context.md#3-write-keys-for-references-lifetimes-and-arrays) describes.

## Where to go next

- [Wiring a context](./wiring-a-context.md): the steps that avoid these mistakes.
- [`cargo cgp check`](/docs/cargo-cgp/check): the tool, and the error classes it recognizes.
- [Checking your wiring](/docs/concepts/check-traits): why CGP checks wiring lazily, and what a
  check forces.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
