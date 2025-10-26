+++

title = "Announcing cgp-serde: A modular serialization library for Serde powered by CGP"

authors = ["Soares Chen"]

+++

# Overview

I am excited to announce the release of [**`cgp-serde`**](https://github.com/contextgeneric/cgp-serde), a modular serialization library for [Serde](https://serde.rs/) that is powered by [**Context-Generic Programming**](/) (CGP).

In a nutshell, `cgp-serde` extends the original [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) and [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) traits of Serde, and makes it possible for anyone to bypass the **coherence restrictions** in Rust and write **overlapping** or **orphaned** implementations of these traits.

Additionally, `cgp-serde` enables us to use the [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) concepts in stable Rust today. This makes it possible for us write context-dependent implementations of `Deserialize`, such as one that uses an arena allocator to deserialize a `&'a T` value that is described in the proposal article.

# Context-Generic Serialization Traits

At its core, `cgp-serde` introduces context-generic versions of the Serde traits. First, the [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) trait is redefined as follows:

```rust
#[cgp_component {
    provider: ValueSerializer,
    derive_delegate: UseDelegate<Value>,
}]
pub trait CanSerializeValue<Value: ?Sized> {
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}
```

Compared to the original `Serialize` trait, `cgp-serde` provides a `CanSerializeValue` CGP trait that has the `Self` type in `Serialize` moved to an explicit generic parameter called `Value`. The `Self` type in `CanSerializeValue` instead represents a *context* type, that can be used to provide *dependency injection*. The `serialize` method also accepts an extra `&self` value, which can be used to retrieve additional runtime dependencies from the context.

Similarly, `cgp-serde` defines a context-generic version of the [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) trait as follows:

```rust
#[cgp_component {
    provider: ValueDeserializer,
    derive_delegate: UseDelegate<Value>,
}]
pub trait CanDeserializeValue<'de, Value> {
    fn deserialize<D>(&self, deserializer: D) -> Result<Value, D::Error>
    where
        D: serde::Deserializer<'de>;
}
```

Similar to `CanSerializeValue`, the trait `CanDeserializeValue` moves the original `Self` type in `Deserialize` to become the `Value` generic paramter. The `deserialize` method also accepts an additional `&self` value, that can be used to provide runtime dependencies such as an arena allocator.

## Provider Traits

In addition to having an additional `Context` parameter as the `Self` type, both `CanSerializeValue` and `CanDeserializeValue` are annotated with the `#[cgp_component]` macro, which unlocks additional CGP capabilities on the traits.

The `provider` argument to `#[cgp_component]` generates for us the **provider traits** that are called `ValueSerializer` and `ValueDeserializer`. These traits will be used for implementing *named* implementations of the serialization traits that can bypass the coherence restrictions.

On the other hand, in CGP we call the original trait `CanSerializeValue` and `CanDeserializeValue` as the **consumer traits**. In CGP, we will use a CGP trait through its consumer trait, but implement them using its provider trait.

Behind the scene, the provider trait `ValueSerializer` is generated as follows:

```rust
pub trait ValueSerializer<Context, Value: ?Sized> {
    fn serialize<S>(
        context: &Context,
        value: &Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;
}
```

Compared to the consumer trait `CanSerializeValue`, the provider trait `ValueSerializer` moves the original `Self` type to a new `Context` generic parameter. All references to `self` and `Self` are also replaced with `context` and `Context`.

The `Self` type in a provider trait are used as the *provider type*, which is essentially dummy structs that are owned by the defining module. Essentially, CGP works around Rust's coherence restrictions by allowing us to always own a unique provider type when implementing a provider trait. We will see later how the provider trait is implemented.

## `UseDelegate` Provider

Our CGP trait definitions also contain a second `derive_delegate` entry in `#[cgp_component]`. This generates a special `UseDelegate` provider that can be used for **static dispatch** of provider implementations based on the `Value` type. The use of `UseDelegate` will be explained later in this article.

# Overlapping Provider Implementations

Compared to the original definitions of `Serialize` and `Deserialize` in Serde, the biggest improvement offered by `CanDeserializeValue` and `CanDeserializeValue` is that we can now define **overlapping** and **orphan** implementations of the trait. Let's take a look at a few examples of how this works.

## Serialize with Serde

To remain backward compatible with Serde, the simplest implementation of `CanSerializeValue` is one that uses the `Serialize` trait from Serde to perform the serialization. This is implemented in `cgp-serde` as follows:

```rust
pub struct UseSerde;

#[cgp_impl(UseSerde)]
impl<Context, Value> ValueSerializer<Value> for Context
where
    Value: Serialize,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.serialize(serializer)
    }
}
```

First, we define a dummy struct called `UseSerde`, and use it as the *name* of our provider implementation. We then define a blanket trait implementation that is annotated with `#[cgp_impl]`, and specify `UseSerde` as the provider type.

Following that, we define our implementation on the `ValueSerializer` provider trait, instead of the `CanSerializeValue` consumer trait. This implementation is defined to work with any `Context` and `Value` types, provided that the target `Value` implements `Serialize`. Inside our serialize implementation, we ignore `&self` and simply use `Serialize::serialize` to serialize the value.

There is nothing remarkable with this implementation. But the key is that this highlights that `cgp-serde` can remain compatible with the original Serde crate. So if we want to reuse an existing `Serialize` implementation of a value type, we can just use `UseSerde` to serialize that type in `CanSerializeValue`.

Another important thing to notice is that our blanket implementation for `UseSerde` works with *any* `Context` and `Value` types. And as we will see next, we can more than one of such blanket implementations defined with CGP.

## Serialize with `Display`

Just as we can implement `ValueSerializer` for any `Value` type that implements `Serialize`, we can also implement `ValueSerializer` for any `Value` type that implements `Display`. This is implemented by `cgp-serde` as follows:

```rust
#[cgp_impl(new SerializeWithDisplay)]
impl<Context, Value> ValueSerializer<Value> for Context
where
    Context: CanSerializeValue<String>,
    Value: Display,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = value.to_string();
        self.serialize(&str_value, serializer)
    }
}
```

In the first line, we include a `new` keyword in `#[cgp_impl]`, so that the macro will generate the following provider struct definition for us:

```rust
struct SerializeWithDisplay;
```

Our blanket implementation for `SerializeWithDisplay` works with any `Value` type that implements `Display`. Additionally, our implementation also works with any `Context` type that implements `CanSerializeValue<String>`. That is, we use the `Context` to *look up* the serialize implementation of `String`, and use it in our provider implementation.

Inside our method body, we first use `to_string` to convert our value into a string, and then we use `self.serialize` to serialize the string value through the context using `CanSerializeValue<String>`.

To better understand how this implementation works, we can imagine how this could be implemented on Serde's `Serialize` trait:

```rust
impl<Value> Serialize for Value
where
    Value: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}
```

Crucially, if you have some experience in Rust traits, you might notice that it is practically not possible to define this `Serialize` implementation in Serde. Or more accurately, you can have at most one of such blanket implementation of `Serialize`. And because of that restriction, it is very challenging to justify why this version that uses `Value: Display` should be the *chosen one* implementation for `Serialize`.

On the other hand, both `UseSerde` and `SerializeWithDisplay` contains **overlapping** implementations of `ValueSerializer` for *both* the `Context` and `Value` types. In vanilla Rust, this would have been rejected, as it is for example possible to have a `Value` type that both implements `Serialize` and `Display`. However, this is made possible in CGP with the use of the provider trait `ValueSerializer` and the macro `#[cgp_impl]`. We will explain in later sections on how this really works.

For this particular use case of string serialization, it might not look remarkable that we need to look up from the context on how to serialize `String`, since Serde already have an efficient implementation of `Serialize` for `String`. However, this demonstrates a *potential* for us to replace the serialization implementation of `String` with something else. We will see later how this override can be useful for the case of serializing `Vec<u8>` bytes.

## Desugaring of `#[cgp_impl]`

Behind the scene, the overlapping implementations of `UseSerde` and `SerializeWithDisplay` are made possible through the use of the `ValueSerializer` provider trait. Although they look like blanket implementations, behind the scene, a provider implementation like `SerializeWithDisplay` is desugared by `#[cgp_impl]` into the following:

```rust
impl<Context, Value> ValueSerializer<Context, Value> for SerializeWithDisplay
where
    Context: CanSerializeValue<String>,
    Value: Display,
{
    fn serialize<S>(
        context: &Context,
        value: &Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str_value = value.to_string();
        context.serialize(&str_value, serializer)
    }
}
```

As we can see, `#[cgp_impl]` moves the `Context` parameter from the `Self` position to become the first generic parameter in `ValueSerializer`. The `Self` type instead becomes `SerializeWithDisplay`, which is the dummy struct that we have just defined.

Because we own the `Self` type `SerializeWithDisplay`, Rust allows us to define the provider trait implementation, even if it is overlapping with other implementations on `Context` and `Value`. This is the core mechanism of how CGP enables overlapping and orphan implementations to be defined. Later, we will also look at how the provider implementations are *wired* with a concrete context.

## Serialize Bytes

Just as we can serialize any `Value` that implements `Display`, we can serialize any `Value` that contains a byte slice into bytes. This is implemented by `cgp-serde` as follows:

```rust
#[cgp_impl(SerializeBytes)]
impl<Context, Value> ValueSerializer<Value> for Context
where
    Value: AsRef<[u8]>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(value.as_ref())
    }
}
```

Our `SerializeBytes` provider can work with any `Value` type that implements `AsRef<[u8]>`. Crucially, this includes `Vec<u8>`, which also implements `AsRef<[u8]>`. This means that unlike the `Serialize`, we can potentially **override** the serialize implementation of `Vec<u8>` to use `SerializeBytes`, so that it is serialized as bytes instead of a list of `u8` values.

## Serialize Iterator

Similar to how we implement `SerializeWithDisplay`, we can define a `SerializeIterator` provider that works with any `Value` type that implements [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html):

```rust
#[cgp_impl(new SerializeIterator)]
impl<Context, Value> ValueSerializer<Value> for Context
where
    for<'a> &'a Value: IntoIterator,
    Context: for<'a> CanSerializeValue<<&'a Value as IntoIterator>::Item>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    { ... }
}
```

Our implementation contains a [*higher-ranked trait bound*](https://doc.rust-lang.org/nomicon/hrtb.html) (HRTB) `for<'a> &'a Value: IntoIterator`, which allows us to call `into_iter` on any `&Value` reference. Similarly, we have a HRTB for `Context` to implement `CanSerializeValue` for the associated `Item` type from the iterator produced by `&Value`.

We omit the method body of `SerializeIterator` for brievity. Behind the scene, it uses [`serialize_seq`](https://docs.rs/serde/latest/serde/trait.Serializer.html#tymethod.serialize_seq) to perform serialization for each item.

The key to note here is that the serialization of the iterator's `Item`s is done through the consumer trait `CanSerializeValue` provided by `Context`. This means that we will be able to deeply customize how the `Item` is serialized, without being restricted to a specific `Serialize` implementation.

Another key observation is that both `SerializeBytes` and `SerializeIterator` are **overlapping** on `Vec<u8>`. This shows that how `Vec<u8>` is serialized depends on which provider is wired in a CGP context. We will revisit this in later sections.

# Modular Serialization Demo

To demonstrate the modular serialization capabilities provided by `cgp-serde`, we will set up an example use case of serializing encrypted messages.

Suppose that we are building a naive encrypted messaging library, and we define the following data types:

```rust
#[derive(CgpData)]
pub struct EncryptedMessage {
    pub message_id: u64,
    pub author_id: u64,
    pub date: DateTime<Utc>,
    pub encrypted_data: Vec<u8>,
}

#[derive(CgpData)]
pub struct MessagesByTopic {
    pub encrypted_topic: Vec<u8>,
    pub messages: Vec<EncryptedMessage>,
}

#[derive(CgpData)]
pub struct MessagesArchive {
    pub decryption_key: Vec<u8>,
    pub messages_by_topics: Vec<MessagesByTopic>,
}
```

We first have an `EncryptedMessage` struct that contains some encrypted message data. The messages are grouped in a `MessagesByTopic` struct, which also contains an encrypted topic about the group of messages. Finally, we have a `MessagesArchive` struct that contains messages grouped by multiple topics, as well as a password-protected decryption key.

The key challenge that we want to address here is how can we serialize our message archive in different JSON formats, depending on the application. In particular, we want to support the following two formats:

- Application A: Bytes as hex strings and date as RFC 3339 format
  <details>
    <summary>Click here for example serialization for App A</summary>

  ```json
  {
    "decryption_key": "746f702d736563726574",
    "messages_by_topics": [
      {
        "encrypted_topic": "416c6c2061626f757420434750",
        "messages": [
          {
            "message_id": 1,
            "author_id": 2,
            "date": "2025-11-03T14:15:00+00:00",
            "encrypted_data": "48656c6c6f2066726f6d20527573744c616221"
          },
          {
            "message_id": 4,
            "author_id": 8,
            "date": "2025-12-19T23:45:00+00:00",
            "encrypted_data": "4f6e65207965617220616e6e697665727361727921"
          }
        ]
      }
    ]
  }
  ```
  </details>

- Application B: Bytes as base64 strings and date as Unix timestamps
  <details>
    <summary>Click here for example serialization for App B</summary>

  ```json
  {
    "decryption_key": "dG9wLXNlY3JldA==",
    "messages_by_topics": [
      {
        "encrypted_topic": "QWxsIGFib3V0IENHUA==",
        "messages": [
          {
            "message_id": 1,
            "author_id": 2,
            "date": 1762179300,
            "encrypted_data": "SGVsbG8gZnJvbSBSdXN0TGFiIQ=="
          },
          {
            "message_id": 4,
            "author_id": 8,
            "date": 1766187900,
            "encrypted_data": "T25lIHllYXIgYW5uaXZlcnNhcnkh"
          }
        ]
      }
    ]
  }
  ```
  </details>

In practice, there may be more than two applications that use our library, and there may be more data types in our library, with more fields that need to be customized.

With the original design of Serde, it would be pretty challenging to provide this level of deep customization of how our data types can be serialized. Typically, a type like `EncryptedMessage` would have a unique `Serialize` implementation that cannot be further customized. Even if we use the [remote derive](https://serde.rs/remote-derive.html) feature in Serde, it would require ad hoc serialization definition for all data types involved.

## Wiring of serializer components

With `cgp-serde`, it is rather straightforward to define custom application contexts that can deeply customize how each field in our data structures is serialized. For example, we can define an `AppA` context for application A as follows:

```rust
pub struct AppA;

delegate_components! {
    AppA {
        ValueSerializerComponent:
            UseDelegate<new SerializerComponentsA {
                <'a, T> &'a T:
                    SerializeDeref,
                [
                    u64,
                    String,
                ]:
                    UseSerde,
                Vec<u8>:
                    SerializeHex,
                DateTime<Utc>:
                    SerializeRfc3339Date,
                [
                    Vec<EncryptedMessage>,
                    Vec<MessagesByTopic>,
                ]:
                    SerializeIterator,
                [
                    MessagesArchive,
                    MessagesByTopic,
                    EncryptedMessage,
                ]:
                    SerializeFields,
            }>
    }
}
```

And similarly, we will implement `AppB` to perform the serialization for application B as follows:

```rust
pub struct AppB;

delegate_components! {
    AppB {
        ValueSerializerComponent:
            UseDelegate<new SerializerComponentsB {
                <'a, T> &'a T:
                    SerializeDeref,
                [
                    i64,
                    u64,
                    String,
                ]:
                    UseSerde,
                Vec<u8>:
                    SerializeBase64,
                DateTime<Utc>:
                    SerializeTimestamp,
                [
                    Vec<EncryptedMessage>,
                    Vec<MessagesByTopic>,
                ]:
                    SerializeIterator,
                [
                    MessagesArchive,
                    MessagesByTopic,
                    EncryptedMessage,
                ]:
                    SerializeFields,
            }>
    }
}
```

If we compare the `delegate_components!` entries in both `AppA` and `AppB`, we will find out that the only differences are in the following:

- The serialization for `Vec<u8>` is handled by `SerializeHex` in `AppA`, but `SerializeBase64` in `AppB`.
- The serialization for `DateTime<Utc>` is handled by `SerializeRfc3339Date` in `AppA`, but `SerializeTimestamp` in `AppB`.
- A serialization entry for `i64` is added for `AppB` to handle the serialization of Unix timestamps in `i64` format.

As we can see, the wiring configuration between `AppA` and `AppB` only require a few lines of changes, to have the change of serialization format. In practice, there are further CGP patterns available for `AppA` and `AppB` to share their common entries through a *preset*. But we will omit the details for brievity.

## Serialization with `serde_json`

`cgp-serde` remains backward compatible with the existing Serde ecosystem. This means that we can reuse existing libraries such as `serde_json` to serialize our encrypted message archive payloads into JSON.

However, since `serde_json` works only with `Serialize`, `cgp-serde` provides the `SerializeWithContext` wrapper to wrap the serialize value with the application context to provide a context-aware implementation of `Serialize`. With it, we can serialize our data to JSON as follows:

```rust
let app_a = AppA { ... };
let archive = MessagesArchive { ... };

let serialized_a = serde_json::to_string(&SerializeWithContext::new(&app_a, &archive)).unwrap()
```

We first use `SerializeWithContext::new` to wrap both the application context and the target value together. We then pass it to `serde_json::to_string`, which accepts `SerializeWithContext` that provides a wrapped `Serialize` implementation.

Similarly, we can get a different JSON output by using `AppB` as the application context:


```rust
let app_b = AppB { ... };

let serialized_b = serde_json::to_string(&SerializeWithContext::new(&app_b, &archive)).unwrap()
```

As we can see, `cgp-serde` makes it easy to customize the serialization of any field nested within other data types. By changing the application context, we are able to generate JSON output in different formats with minimal effort.

## Derive-free serialization with `#[derive(CgpData)]`

Aside from the deep customization provided by `cgp-serde`, another key feature to highlight is that there is no need to use derive macros to generate any serialize implementation for custom data types. If you look back at the definition of the types like `EncryptedMessage`, you will see that the it only uses a general `#[derive(CgpData)]` macro provided by CGP.

Behind the scene, `#[derive(CgpData)]` generates the support traits for [extensible data types](https://contextgeneric.dev/blog/extensible-datatypes-part-1/), and enables our data types to work with CGP traits like `CanSerializeValue` without requiring custom derivation. Instead, `cgp-serde` provides a `SerializeFields` provider that can work with any struct that derives `CgpData`.

This shows that `cgp-serde` also solves the orphan implementation problem, by not requiring libraries to derive library-specific implementations on their data types. For instance, our encrypted messaging library do not even need to include `cgp-serde` as its dependency. As long as the library uses the base `cgp` crate to derive `CgpData`, we will be able to serialize its data types through `SerializeFields`.

Further more, the use of extensible data types applies not only to the traits in `cgp-serde`. Instead, a general derivation of `CgpData` will enable the library data types to work with other CGP traits in similar ways as `cgp-serde`. Because of this, CGP can shield library authors from external requests to use derive macros for all popular traits, just to work around the orphan rules in Rust.

## Type-Level Lookup Tables

Behind the scene, the code above are conceptually equivalent to constructing the following type-level lookup table on `AppA`:

| Name | Value |
|----|----|
| `ValueSerializerComponent` | `UseDelegate<SerializerComponentsA>` |

and a new type-level lookup table called `SerializerComponentsA`:

| Name | Value |
|----|----|
| `&'a T` | `SerializeDeref` |
| `u64` | `UseSerde` |
| `String` | `UseSerde` |
| `Vec<u8>` | `SerializeHex` |
| `DateTime<Utc>` | `SerializeRfc3339Date` |
| `Vec<EncryptedMessage>` | `SerializeIterator` |
| `Vec<MessagesByTopic>` | `SerializeIterator` |
| `MessagesArchive` | `SerializeFields` |
| `MessagesByTopic` | `SerializeFields` |
| `EncryptedMessage` | `SerializeFields` |

When the trait system needs to look up for a trait implementation, such as `Vec<EncryptedMessage>`, it would perform the following lookup:

- `AppA` needs to implement `CanSerializeValue<Vec<EncryptedMessage>>`. To do so, we need to look up the type-level table in `AppA` with `ValueSerializerComponent` as the lookup key.
- `AppA`'s table contains an entry for `ValueSerializerComponent`, with the value being `UseDelegate<SerializerComponentsA>`. We would need the value `UseDelegate<SerializerComponentsA>` to implement `ValueSerializer<AppA, Vec<EncryptedMessage>>`.
- `UseDelegate` implements `ValueSerializer` by performing a further lookup on the `SerializerComponentsA` table, with `Vec<EncryptedMessage>` being the key.
- `SerializerComponentsA` contains an entry for `Vec<EncryptedMessage>`, with the value being `SerializeIterator`. So we need `SerializeIterator` to implement `ValueSerializer<AppA, Vec<EncryptedMessage>>`.
- For `SerializeIterator` to implement `ValueSerializer<AppA, Vec<EncryptedMessage>>`, it needs `AppA` to implement `CanSerializeValue<EncryptedMessage>`.
- The whole lookup process is repeated from the top again, until it reaches the `EncryptedMessage` entry in `SerializerComponentsA`, which points to `SerializeFields`.

The table lookup process may seem complicated, but it actually works very similar to how [vtable lookups](https://en.wikipedia.org/wiki/Virtual_method_table) are performed in [`dyn` traits in Rust](https://www.youtube.com/watch?v=pNA-XAIrDTk) and also in object-oriented laguages like Java.

The main difference is that CGP's lookup tables are implemented at the type-level, meaning that the tables don't exist at runtime and thus have **no runtime overhead**.

## Implementation of Lookup Tables

Behind the scene, the `delegate_components!` macro constructs the type-level lookup tables using the `DelegateComponent` trait, which is defined by `CGP` as follows:

```rust
pub trait DelegateComponent<Name: ?Sized> {
    type Delegate;
}
```

Essentially, `DelegateComponent` allows us to use any type as a table, and set a "value" on the "key" of the table by implementing the trait. For example, the `ValueSerializerComponent` entry in `AppA` is set through the following implementation:

```rust
impl DelegateComponent<ValueSerializerComponent> for AppA {
    type Delegate = UseDelegate<SerializerComponentsA>;
}
```

And similarly, the `Vec<EncryptedMessage>` entry in `SerializerComponentsA` is set through the following implementation:

```rust
impl DelegateComponent<Vec<EncryptedMessage>> for SerializerComponentsA {
    type Delegate = SerializeIterator;
}
```

CGP also generates *blanket implementations* on the consumer and provider traits that make use of the lookup table entries in `DelegateComponent` to figure out how to lookup for a provider implementation at compile time.

For example, the lookup mechanism for `CanSerializeValue` is implemented as follows:

```rust
impl<Context, Value: ?Sized> CanSerializeValue<Value> for Context
where
    Context: DelegateComponent<ValueSerializerComponent>,
    Context::Delegate: ValueSerializer<Context, Value>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Context::Delegate::serialize(self, value, serializer)
    }
}
```

Essentially, the consumer trait `CanSerializeValue` is implemented for a context like `AppA`, if `AppA` contains a lookup table entry with `ValueSerializerComponent` being the key, and the `Delegate` "value" in the lookup entry implements `ValueSerializer`.

Similarly, the lookup mechanism for `UseDelegate` is implemented as follows:

```rust
#[cgp_impl(UseDelegate<Components>)]
impl<Context, Value> ValueSerializer<Value> for Context
where
    Components: DelegateComponent<Value>,
    Components::Delegate: ValueSerializer<Context, Value>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Components::Delegate::serialize(self, value, serializer)
    }
}
```

Essentially, `UseDelegate` uses the `Value` type as the lookup "key" in a given components table, such as `SerializerComponentsA`. Aside from the difference in the lookup "key", the implementation is similar to the earlier blanket implementation for `CanSerializeValue`.
