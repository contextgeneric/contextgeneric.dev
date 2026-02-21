---
slug: cgp-serde-release
title: 'Announcing cgp-serde: A modular serialization library for Serde powered by CGP'
authors: [soares]
tags: [release]
---

I am excited to announce the release of [**cgp-serde**](https://github.com/contextgeneric/cgp-serde), a modular serialization library for [Serde](https://serde.rs/) that leverages the power of [**Context-Generic Programming**](/) (CGP).

In short, `cgp-serde` extends Serde’s original [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) and [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) traits with CGP, making it possible to write **overlapping** or **orphaned** implementations of these traits and thus bypass the standard Rust **coherence restrictions**.

Furthermore, `cgp-serde` allows us to leverage the powerful [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) concepts in stable Rust today. This unlocks the ability to write context-dependent implementations of `Deserialize`, such as one that uses an arena allocator to deserialize a `'a T` value, a concept detailed in the proposal article.

<!-- truncate -->

## Preface

This is a companion blog post for my [RustLab presentation](https://rustlab.it/talks/how-to-stop-fighting-with-coherence-and-start-writing-context-generic-trait-impls) titled **How to Stop Fighting with Coherence and Start Writing Context-Generic Trait Impls**.

## Quick intro to Context-Generic Programming

For those readers new to the project, here is a quick introduction: Context-Generic Programming (CGP) is a modular programming paradigm that enables you to bypass the **coherence restrictions** in Rust traits, allowing for **overlapping** and **orphan** implementations of any CGP trait.

You can adapt almost any existing Rust trait to use CGP today by applying the `#[cgp_component]` macro to the trait definition. After this annotation, you can write **named** implementations of the trait using `#[cgp_impl]`, which can be defined without being constrained by the coherence rules. You can then selectively enable and reuse the named implementation for your type using the `delegate_components!` macro.

For instance, we can, in principle, annotate the standard library’s [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html) trait with `#[cgp_component]` like this:

```rust
#[cgp_component(HashProvider)]
pub trait Hash { ... }
```

This change does not affect existing code that uses or implements `Hash`, but it allows for new, potentially overlapping implementations, such as one that works for any type that also implements `Display`:

```rust
#[cgp_impl(HashWithDisplay)]
impl<T: Display> HashProvider for T { ... }
```

You can then apply and reuse this implementation on any type by using the `delegate_components!` macro:

```rust
pub struct MyData { ... }
impl Display for MyData { ... }

delegate_components! {
    MyData {
        HashProviderComponent: HashWithDisplay,
    }
}
```

In this example, `MyData` implements the `Hash` trait by using `delegate_components!` to delegate its implementation to the `HashWithDisplay` provider, identified by the key `HashProviderComponent`. Because `MyData` already implements `Display`, the `Hash` trait is now automatically implemented through CGP via this delegation.

If you are eager to learn more about CGP, please check out the [project homepage](/) for all the details. For now, let us return to examine the new features introduced in `cgp-serde`.

---

## Context-Generic Serialization Traits

The key highlight of `cgp-serde` is its introduction of context-generic versions of the Serde traits. First, the [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) trait is redefined as follows:

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

Compared to the original `Serialize` trait, `cgp-serde` provides the `CanSerializeValue` CGP trait, which moves the original `Self` type from `Serialize` to an explicit generic parameter called `Value`. The `Self` type in `CanSerializeValue` now represents a **context** type, which can be used for **dependency injection**. The `serialize` method also accepts an extra `&self` value, making it possible to retrieve additional runtime dependencies from this context.

In a similar manner, `cgp-serde` defines a context-generic version of the [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) trait as follows:

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

Analogous to `CanSerializeValue`, the `CanDeserializeValue` trait moves the original `Self` type in `Deserialize` to become the `Value` generic parameter. This `deserialize` method similarly accepts an additional `&self` value, which can be utilized to supply runtime dependencies, such as an arena allocator.

### Provider Traits

In addition to having the extra `Context` parameter as the `Self` type, both `CanSerializeValue` and `CanDeserializeValue` are annotated with the `#[cgp_component]` macro, which is the mechanism that unlocks additional CGP capabilities on these traits.

The `provider` argument to `#[cgp_component]` automatically generates the **provider traits** called `ValueSerializer` and `ValueDeserializer`. These traits are the ones you will use for implementing **named** serialization implementations that can bypass the coherence restrictions.

Conversely, in CGP, we refer to the original traits `CanSerializeValue` and `CanDeserializeValue` as the **consumer traits**. The general rule of thumb is that a CGP component is **used** through its consumer trait but **implemented** using its provider trait.

### `UseDelegate` Provider

Our CGP trait definitions also include a second `derive_delegate` entry within the `#[cgp_component]` macro. This entry generates a generic `UseDelegate` provider that enables **static dispatch** of provider implementations based on the specific `Value` type. The practical application and use of `UseDelegate` will be explained in greater detail later in this article.

---

## Overlapping Provider Implementations

Compared to the original Serde definitions of `Serialize` and `Deserialize`, the greatest improvement offered by `CanSerializeValue` and `CanDeserializeValue` is the ability to define **overlapping** and **orphan** implementations of the trait. Let us now examine a few concrete examples of how this crucial feature works.

### Serialize with Serde

To maintain full backward compatibility with the existing Serde ecosystem, the most straightforward implementation of `ValueSerializer` utilizes Serde’s own `Serialize` trait to perform serialization. This is implemented within `cgp-serde` as shown below:

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

First, we define a unit struct named `UseSerde`, which acts as the **name** for our provider implementation. We then define a blanket trait implementation annotated with `#[cgp_impl]`, explicitly setting `UseSerde` as the provider type.

Following this, we define our implementation on the `ValueSerializer` provider trait, rather than the `CanSerializeValue` consumer trait. This implementation is defined to work with any `Context` and `Value` types, provided that the target `Value` implements the original `Serialize` trait. Inside our `serialize` implementation, we ignore the `&self` context and simply call `Serialize::serialize` on the value.

While this implementation itself is not remarkable, it crucially highlights that `cgp-serde` is fully compatible with the standard Serde crate. Consequently, if we wish to reuse an existing `Serialize` implementation for a given value type, we can simply utilize `UseSerde` to serialize that type through `CanSerializeValue`.

Another important detail to notice is that our blanket implementation for `UseSerde` works universally for *any* `Context` and `Value` types satisfying the bounds. As we will soon see, we can define **more than one** such blanket implementation using CGP.

### Serialize with `Display`

Just as we can implement `ValueSerializer` for any `Value` type that implements `Serialize`, we can also implement `ValueSerializer` for any `Value` type that implements the `Display` trait. This alternative behavior is implemented by `cgp-serde` as follows:

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

In the very first line, the inclusion of the `new` keyword in `#[cgp_impl]` instructs the macro to automatically generate the necessary provider struct definition for us, so that we don't have to define them manually:

```rust
struct SerializeWithDisplay;
```

Our blanket implementation for `SerializeWithDisplay` works with any `Value` type that implements `Display`. Crucially, this implementation also requires the `Context` type to implement `CanSerializeValue<String>`. This means we use the `Context` to **look up** the serialization implementation for `String` and then employ it within our current provider implementation.

Inside the method body, we first use `to_string` to convert our value into a standard string, and then we call `self.serialize` to serialize that string value using the context's `CanSerializeValue<String>` implementation.

To appreciate what is enabled by this implementation, consider how this might be implemented directly on Serde's `Serialize` trait:

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

If you have any experience with Rust traits, you will immediately recognize that it is practically impossible to define this blanket `Serialize` implementation in Serde. More accurately, you are **restricted** to having **at most one** such blanket implementation of `Serialize`. Because of this restriction, it is extremely difficult to justify why this version, which uses the `Value: Display` bound, should be the *chosen* blanket implementation for `Serialize`.

In stark contrast, both `UseSerde` and `SerializeWithDisplay` contain **overlapping** implementations of `ValueSerializer` across *both* the `Context` and `Value` types. In vanilla Rust, this would be outright rejected, as it is perfectly possible, for instance, to have a `Value` type that implements both `Serialize` and `Display`. However, this overlapping is seamlessly enabled in CGP by utilizing the provider trait `ValueSerializer` and the powerful `#[cgp_impl]` macro. We will elaborate on the underlying mechanism in later sections.

Regarding the specific use case of string serialization, it might not seem remarkable that we must look up how to serialize `String` from the context, given that Serde already has an efficient `Serialize` implementation for `String`. Nevertheless, this successfully demonstrates the *potential* to replace the serialization implementation of `String` with a custom one. We will later see how this override capability is highly useful for serializing `Vec<u8>` bytes.

### Serialize Bytes

Just as we can serialize any `Value` that implements `Display`, we can also define a way to serialize any `Value` that contains a byte slice directly into bytes. This behavior is implemented by `cgp-serde` as follows:

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

Our `SerializeBytes` provider can successfully work with any `Value` type that implements `AsRef<[u8]>`. Crucially, this includes `Vec<u8>`, which also implements `AsRef<[u8]>`. This is significant because, unlike the constraints imposed by the standard `Serialize` trait, we can now potentially **override** the serialization implementation of `Vec<u8>` to explicitly use `SerializeBytes`, ensuring it is serialized as raw bytes instead of a list of `u8` values.

### Serialize Iterator

Similar to how we implemented `SerializeWithDisplay`, we can define a `SerializeIterator` provider that works with any `Value` type that implements [`IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html):

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

Our implementation includes a [*higher-ranked trait bound*](https://doc.rust-lang.org/nomicon/hrtb.html) (HRTB) `for<'a> &'a Value: IntoIterator`, which permits us to call `into_iter` on any reference `&Value`. Likewise, we introduce a HRTB for `Context` to implement `CanSerializeValue` for the associated `Item` type yielded by the iterator produced from `&Value`.

We have omitted the method body of `SerializeIterator` for brevity. Behind the scenes, it utilizes `serialize_seq` to handle the serialization of each item.

The key takeaway here is that the serialization of the iterator's `Item`s is performed via the consumer trait `CanSerializeValue` provided by `Context`. This grants us the ability to deeply customize how the `Item` itself is serialized, without being restricted to a fixed `Serialize` implementation.

Another critical observation is that both `SerializeBytes` and `SerializeIterator` are inherently **overlapping** on the `Vec<u8>` type. This perfectly illustrates how the serialization behavior of `Vec<u8>` is determined entirely by which specific provider is wired into a particular CGP context. We will examine this topic further in later sections.

---

## Modular Serialization Demo

To fully demonstrate the modular serialization capabilities provided by `cgp-serde`, we will set up a practical example involving the serialization of encrypted messages. This is where you see the power of CGP in action.

Suppose we are developing a naive encrypted messaging library, defining the following core data types:

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

We start with an `EncryptedMessage` struct containing message metadata and encrypted data. These messages are grouped within a `MessagesByTopic` struct, which also includes an encrypted topic string. Finally, the `MessagesArchive` struct holds messages grouped by multiple topics, along with a password-protected decryption key.

The key technical challenge we aim to solve is how to serialize this message archive into different JSON formats, depending on the specific application consuming the data. Specifically, we need to support the following two formats simultaneously:

- **Application A:** Serializes bytes as hexadecimal strings and dates using the RFC 3339 format.
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

- **Application B:** Serializes bytes as Base64 strings and dates using Unix timestamps.
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

In a real-world scenario, you might have many more applications using your library, and your data types could have numerous fields requiring customization. With the original design of Serde, achieving this deep level of customization across nested data types would be quite challenging. Typically, a type like `EncryptedMessage` would have a single, fixed `Serialize` implementation. Even Serde’s powerful [remote derive](https://serde.rs/remote-derive.html) feature would require defining ad-hoc serialization for every data type involved.

### Wiring of serializer components

With `cgp-serde`, it is straightforward to define custom application contexts that can deeply customize how each field in our data structures is serialized. For instance, we can define an `AppA` context for Application A like this:

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

In the code above, we use the `delegate_components!` macro to create effective **type-level lookup tables** that configure the specific provider implementations used by `AppA`. The **component key**, `ValueSerializerComponent`, tells the compiler that we are configuring the provider for the `CanSerializeValue` trait within `AppA`.

The value assigned to this entry is `UseDelegate`, followed by an *inner* table named `SerializerComponentsA`. This inner table is used for **static dispatch** of the provider implementation based on the `Value` type being serialized. For example, the key `Vec<u8>` is mapped to the value `SerializeHex`, indicating that the `SerializeHex` provider is used whenever `Vec<u8>` needs to be serialized.

The `delegate_components!` macro also includes shorthands for mapping multiple keys to the same value. For instance, both `u64` and `String` are dispatched to the generic `UseSerde` provider, which is neatly grouped using an array syntax. We can also set **generic keys** in the table, such as mapping all `&'a T` references to the `SerializeDeref` provider.

We will cover more details about the mechanics of this type-level lookup table in later sections. For now, let us look at how we implement `AppB` to perform the serialization required for Application B:

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

If we meticulously compare the `delegate_components!` entries in both `AppA` and `AppB`, we discover that the only substantive differences are in the following entries:


* The serialization for `Vec<u8>` is handled by `SerializeHex` in `AppA`, but is switched to `SerializeBase64` in `AppB`.
* The serialization for `DateTime<Utc>` is handled by `SerializeRfc3339Date` in `AppA`, but is replaced by `SerializeTimestamp` in `AppB`.
* An additional serialization entry for `i64` is included for `AppB` to specifically handle the serialization of Unix timestamps in `i64` format.

As we can clearly observe, changing the serialization format only required a **few lines of configuration changes** in the wiring. This dramatically demonstrates the flexibility of CGP to make application implementations highly configurable and easily adaptable.

In practice, there are further CGP patterns available for `AppA` and `AppB` to share their *common* `delegate_components!` entries through a powerful **preset** mechanism. However, we will omit those details here for brevity.

### Serialization with `serde_json`

A key feature of `cgp-serde` is its continued **backward compatibility** with the existing Serde ecosystem. This means we can effortlessly reuse established libraries like `serde_json` to serialize our encrypted message archive payloads into JSON.

However, since `serde_json` strictly operates on types that implement the original `Serialize` trait, `cgp-serde` provides the `SerializeWithContext` wrapper. This wrapper wraps the value to be serialized together with the application context, providing a context-aware implementation of `Serialize`. Using it, we can serialize our data to JSON like this:

```rust
let app_a = AppA { ... };
let archive = MessagesArchive { ... };

let serialized_a = serde_json::to_string(
    &SerializeWithContext::new(&app_a, &archive)
).unwrap()
```

We first use `SerializeWithContext::new` to wrap the application context and the target value together. We then pass this wrapper to `serde_json::to_string`, which accepts `SerializeWithContext` because it provides a wrapped `Serialize` implementation.

Similarly, we can generate the entirely different JSON output simply by using `AppB` as the application context:

```rust
let app_b = AppB { ... };

let serialized_b = serde_json::to_string(
    &SerializeWithContext::new(&app_b, &archive)
).unwrap()
```

As illustrated, `cgp-serde` makes it remarkably easy to customize the serialization of *any* field, regardless of how deeply it is nested within other data types. By merely changing the application context, we are able to generate JSON output in fundamentally different formats with minimal effort.

### Derive-free serialization with `#[derive(CgpData)]`

Beyond the deep customization we have just explored, another critical feature to highlight is that there is virtually no need to use derive macros to generate any serialization-specific implementation for custom data types. If you look back at the definition of types like `EncryptedMessage`, you will notice that it only uses the general `#[derive(CgpData)]` macro provided by the base CGP library.

Behind the scenes, `#[derive(CgpData)]` generates the necessary support traits for [extensible data types](https://contextgeneric.dev/blog/extensible-datatypes-part-1/), which enables our data types to naturally work with CGP traits like `CanSerializeValue` without requiring library-specific derivation. This is made possible, because CGP enables `cgp-serde` to implement a generic `SerializeFields` provider that can work with any struct that derives `CgpData`, without being restricted by the overlapping constraints.

This mechanism shows how `cgp-serde` fundamentally solves the orphan implementation problem: it avoids requiring library authors to derive library-specific implementations on their data types at all. For instance, our encrypted messaging library does not even need to include `cgp-serde` or `serde` as a dependency. As long as the library uses the base `cgp` crate to derive `CgpData`, we can serialize its data types using the `SerializeFields` provider.

Furthermore, the use of extensible data types applies not only to the traits in `cgp-serde`. A general derivation of `CgpData` will automatically enable the library’s data types to work with *other* CGP traits in the same way they work with `cgp-serde`. Because of this universal applicability, CGP can shield library authors from endless external requests to apply derive macros for every popular trait on their data types, simply to work around the archaic orphan rules in Rust.

### Full Example

The complete working example of this customized serialization is available on [GitHub](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-tests/src/tests/messages.rs).

---

## Capabilities-Enabled Deserialization Demo

Now that we have demonstrated how `cgp-serde` enables highly modular serialization, let us turn our attention to how it unlocks new use cases for deserialization. Specifically, we will show how `cgp-serde` enables the use case explained in the [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal. We will demonstrate implementing a deserializer for the borrowed type `&'a T` using an arena allocator that is retrieved via **dependency injection** from the context itself.

### Coordinate Arena

To illustrate the use of an arena deserializer, let us devise an example application: storing a massive quantity of 3D coordinates, perhaps for rendering complex 3D graphics. We can define our basic coordinate structure as follows:

```rust
#[derive(CgpData)]
pub struct Coord {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}
```

In this demo, the `Coord` struct is minimal, but imagine its actual size is much larger. If we were to use `Box<Coord>` to allocate every coordinate on the heap, the frequent calls to `Box::new(coord)` could lead to severe memory pressure and fragmentation. Instead, we want to employ an [**arena allocator**](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/) to allocate all coordinates into a single, fixed memory region. This setup allows all coordinates to be easily deallocated with a single operation when the function scope exits.

When using arena allocators, our base coordinate value will be `&'a Coord`, a borrowed type with a specific lifetime. We can then store these borrowed coordinates in other data structures, such as a cluster:

```rust
#[derive(CgpData)]
pub struct Cluster<'a> {
    pub id: u64,
    pub coords: Vec<&'a Coord>,
}
```

With our data structures defined, a major challenge emerges: how do we deserialize a cluster of coordinates from a format like JSON and ensure the coordinates are allocated using a custom arena allocator provided by us?

### Arena Deserializer

To tackle the arena allocator use case, we will utilize the popular [`typed-arena`](https://docs.rs/typed-arena/latest/typed_arena/) crate, specifically leveraging its [`Arena`](https://docs.rs/typed-arena/latest/typed_arena/struct.Arena.html) type for memory allocation.

First, we define an *auto getter* trait to retrieve an `Arena` from our context:

```rust
#[cgp_auto_getter]
pub trait HasArena<'a, T: 'a> {
    fn arena(&self) -> &&'a Arena<T>;
}
```

The `HasArena` trait is automatically implemented for any `Context` type, provided it derives `HasField` and contains an `arena` field of the type `&'a Arena<T>`. The nested reference (`&&'a`) is required here, since `#[cgp_auto_getter]` by default returns a reference to a field value in the context, but our field value itself is an explicit reference `&'a Arena<T>`.

Next, we leverage `HasArena` to retrieve the arena allocator from a generic context within our `ValueDeserializer` implementation:

```rust
#[cgp_impl(new DeserializeAndAllocate)]
impl<'de, 'a, Context, Value> ValueDeserializer<'de, &'a Value> for Context
where
    Context: HasArena<'a, Value> + CanDeserializeValue<'de, Value>,
{
    fn deserialize<D>(&self, deserializer: D) -> Result<&'a Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let owned_value = self.deserialize(deserializer)?;
        let borrowed_value = self.arena().alloc(owned_value);

        Ok(borrowed_value)
    }
}
```

We define a new provider, `DeserializeAndAllocate`, which implements `ValueDeserializer` specifically for the borrowed `&'a Value` type. To support this, it requires the `Context` to implement `HasArena<'a, Value>` to get the allocator `&'a Arena`. Additionally, it also requires `Context` to implement `CanDeserializeValue` for the *owned* `Value` type, to perform the initial deserialization on the stack before moving it into the arena.

Inside the method body, we first use the context to deserialize an owned version of the value on the stack. We then call `self.arena()` to retrieve the arena allocator and use its `alloc` method to move and allocate the value onto the arena.

As you can see, with the generalized dependency injection capability provided by CGP, we are able to retrieve any necessary value or type from the context during deserialization. This effectively allows us to emulate the `with` clause in the seminal [Context and Capabilities](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal and provide any required capability during the deserialization process.

### Deserialization Context

Using `cgp-serde`, defining a deserializer context that includes an arena allocator is refreshingly straightforward. We begin by defining the context structure as follows:

```rust
#[derive(HasField)]
pub struct App<'a> {
    pub arena: &'a Arena<Coord>,
}
```

Our `App` context is explicitly parameterized by a lifetime `'a`. It contains an `arena` field that holds a reference to an [`Arena`](https://docs.rs/typed-arena/latest/typed_arena/struct.Arena.html) that lives for the duration of `'a`, and is specialized for the object type `Coord`.

The explicit lifetime `'a` is necessary here because the `alloc` method returns a `&'a Coord` value that shares this same lifetime. By being explicit, we accurately inform the Rust compiler that the allocated coordinates will live exactly as long as `'a`, which may outlive `App` itself.

We also derive `HasField` on `App`, which enables `App` to automatically implement `HasArena<'a, Coord>`. This is made possible, because the `arena` field in `App` matches the format expected by the blanket implementation generated by `#[cgp_auto_getter]`.

With the `App` context defined, let us examine the component wiring for the `ValueDeserializer` providers:

```rust
delegate_components! {
    <'s> App<'s> {
        ValueDeserializerComponent:
            UseDelegate<new DeserializeComponents {
                u64: UseSerde,
                [
                    Coord,
                    <'a> Cluster<'a>,
                ]:
                    DeserializeRecordFields,
                <'a> &'a Coord:
                    DeserializeAndAllocate,
                <'a> Vec<&'a Coord>:
                    DeserializeExtend,
            }>,
        ErrorTypeProviderComponent:
            UseAnyhowError,
        ErrorRaiserComponent:
            RaiseAnyhowError,
    }
}
```

Similar to the serialization lookup tables, here we are configuring the `ValueDeserializer` providers for `App` via the `ValueDeserializerComponent` key and the `UseDelegate` dispatcher. Notice that this table contains several keys with generic lifetimes, `<'a>`, reflecting the use of structs with explicit lifetimes.

As evident in the table, for the value types `Coord` and `Cluster<'a>`, we use a special provider called `DeserializeRecordFields` to deserialize the structs using the **extensible data types** facility derived from `#[derive(CgpData)]`. Crucially, for `&'a Coord`, we select the `DeserializeAndAllocate` provider we defined earlier.

### Error Handling

Besides the `ValueDeserializerComponent`, our `App` context is also configured with **error handling** components provided by CGP. This is essential because we plan to use `serde_json` to deserialize the value, which may naturally return errors.

For simplicity, we choose to use the `cgp-error-anyhow` crate to handle errors using the highly flexible [`anyhow`](https://crates.io/crates/anyhow) crate. In the entry for `ErrorTypeProviderComponent`, we use the `UseAnyhowError` provider to select the type [`anyhow::Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html) as the primary error type for `App`.

Subsequently, in the entry for `ErrorRaiserComponent`, we use `RaiseAnyhowError` to correctly promote source errors, like [`serde_json::Error`](https://docs.rs/serde_json/latest/serde_json/struct.Error.html), into `anyhow::Error` using its standard `From` implementation.

This clearly demonstrates the flexibility afforded by CGP in error handling: the concrete error type is chosen by the application context, and it can also customize how each source error is gracefully handled.

### Deserializing JSON

Now that the component wiring for `App` is complete, let us attempt to use `serde_json` to deserialize a JSON string. First, we create a mock JSON string representing a cluster of coordinates:

```rust
let serialized = r#"
{
    "id": 8,
    "coords": [
        { "x": 1, "y": 2, "z": 3 },
        { "x": 4, "y": 5, "z": 6 }
    ]
}
"#;
```

Next, we instantiate our arena and the application context:

```rust
let arena = Arena::new();
let app = App { arena: &arena };
```

In the case of deserialization, there is a minor complication: we cannot directly use the simple [`serde_json::from_str`](https://docs.rs/serde_json/latest/serde_json/fn.from_str.html) with our `App` context. This is because unlike serialization, `serde_json::from_str` doesn't accept additional parameters that we can use to "pass" around the `app` value. Instead, `cgp-serde` works with the lower-level [`Deserializer`](https://docs.rs/serde_json/latest/serde_json/de/struct.Deserializer.html) implementation in `serde_json`, allowing us to pass `serde_json`'s deserializer directly to the `CanDeserializeValue::deserialize` method, together with the `app` context.

Fortunately, these low-level implementation details are neatly abstracted away by `cgp-serde`, and all we need to do is call the convenient `deserialize_json_string` method on our `App` context:

```rust
let deserialized: Cluster<'_> = app
    .deserialize_json_string(&serialized)
    .unwrap();
```

As we can see, we have successfully utilized the custom arena allocator provided by our `App` context to perform the deserialization, resulting in a borrowed `Cluster` where the coordinates live in the arena.

### Full Example

The full working example of the arena allocator deserialization is available on [GitHub](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-tests/src/tests/arena_simplified.rs).

---

## Implementation Details

In this section, we will delve into the underlying implementation details of CGP that make the impressive level of modularity in `cgp-serde` possible. For audiences who are new to Context-Generic Programming, this is your chance to quickly grasp the essential concepts of CGP required to confidently use `cgp-serde`.

### Provider Traits

When the `#[cgp_component]` macro is applied to a consumer trait, such as `CanSerializeValue`, it automatically generates a companion **provider trait** called `ValueSerializer`. This generated trait looks like the following:

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

Compared to the consumer trait `CanSerializeValue`, the provider trait `ValueSerializer` shifts the original `Self` type into a new `Context` generic parameter. Consequently, all references to `self` and `Self` are appropriately replaced with `context` and `Context`. The `Self` type in a provider trait is instead used as the **provider name**, which are the unique, dummy structs - like `UseSerde` or `SerializeHex` - that are defined and owned by the library module. This is the core trick: CGP circumvents Rust’s coherence restrictions by guaranteeing that we always own a unique provider type when implementing a provider trait.

### Desugaring of `#[cgp_impl]`

The ability to define overlapping provider implementations, such as `UseSerde` and `SerializeWithDisplay`, is achieved through the clever use of the `ValueSerializer` provider trait. While these implementations look like forbidden blanket implementations, a provider implementation like `SerializeWithDisplay` is actually **desugared** by the `#[cgp_impl]` macro into this form:

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

As clearly shown, `#[cgp_impl]` shifts the `Context` parameter away from the `Self` position to become the first generic parameter of `ValueSerializer`. The `Self` type for the implementation instead becomes `SerializeWithDisplay`, the unique dummy struct that we defined. Because the implementing library owns `SerializeWithDisplay`, the Rust compiler permits the trait implementation even if it is otherwise overlapping on the `Context` and `Value` types. This is the central mechanism that allows CGP to define both overlapping and orphan implementations. Next, we will examine how these provider implementations are statically *wired* to a concrete application context.

### Type-Level Lookup Tables

In the [serialization example](#wiring-of-serializer-components) for `AppA`, when the `delegate_components!` macro is invoked, it is conceptually equivalent to building a **type-level lookup table** for that context. This table effectively configures the dispatch mechanism at compile time:

| Name | Value |
|----|----|
| `ValueSerializerComponent` | `UseDelegate<SerializerComponentsA>` |

In the example, the type-level table for `AppA` only contains one entry, with `ValueSerializerComponent` as the key. This entry is used by the `CanSerializeValue` trait to look up for the provider implementation.

In the entry value, the use of the `new SerializerComponentsA { ... }` constructs an **inner table**, `SerializerComponentsA`, which holds further mapping of providers based on the serialization value type:

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

This table is passed as the `SerializerComponentsA` type to `UseDelegate`, which performs the actual dispatch based on the value type.

When the trait system must look up an implementation, such as for serializing `Vec<EncryptedMessage>`, it follows a precise, recursive path:


  * The system begins by checking if `AppA` implements `CanSerializeValue<Vec<EncryptedMessage>>`. This requires looking up the `ValueSerializerComponent` key in the `AppA` table.
  * `AppA`'s table returns `UseDelegate<SerializerComponentsA>`. This value must now implement `ValueSerializer<AppA, Vec<EncryptedMessage>>`.
  * `UseDelegate` implements `ValueSerializer` by performing a secondary lookup on the `SerializerComponentsA` table, using `Vec<EncryptedMessage>` as the key.
  * `SerializerComponentsA` returns the value `SerializeIterator`. This means `SerializeIterator` must now implement `ValueSerializer<AppA, Vec<EncryptedMessage>>`.
  * For `SerializeIterator` to satisfy this requirement, it requests a new constraint: that `AppA` must implement `CanSerializeValue<EncryptedMessage>`.
  * The entire lookup process is repeated from the beginning for the inner type, `EncryptedMessage`, until it eventually points to the concrete provider `SerializeFields`.

This table lookup process, while seem complicated, works conceptually similarly to how [vtable lookups](https://en.wikipedia.org/wiki/Virtual_method_table) are performed for [`dyn` traits in Rust](https://www.youtube.com/watch?v=pNA-XAIrDTk) and in object-oriented languages like Java. The fundamental difference, and a major selling point, is that CGP’s lookup tables are fully implemented at the **type level**. This means the tables are resolved entirely at compile time, resulting in **zero runtime overhead**.

### Implementation of Lookup Tables

Behind the scenes, the `delegate_components!` macro constructs these type-level lookup tables using the `DelegateComponent` trait, which is defined by the base `cgp` crate as follows:

```rust
pub trait DelegateComponent<Name: ?Sized> {
    type Delegate;
}
```

In essence, `DelegateComponent` allows any type to serve as a table. By implementing the trait, we effectively set a "value" (`Delegate`) for a specific "key" (`Name`) in that table. For instance, the `ValueSerializerComponent` entry in `AppA` is set through this implementation:

```rust
impl DelegateComponent<ValueSerializerComponent> for AppA {
    type Delegate = UseDelegate<SerializerComponentsA>;
}
```

Similarly, the `Vec<EncryptedMessage>` entry in the `SerializerComponentsA` table is defined through the following implementation:

```rust
impl DelegateComponent<Vec<EncryptedMessage>> for SerializerComponentsA {
    type Delegate = SerializeIterator;
}
```

CGP then generates essential **blanket implementations** on the consumer and provider traits. These implementations utilize the `DelegateComponent` entries to resolve the correct provider implementation at compile time.

For example, the initial lookup mechanism for the consumer trait `CanSerializeValue` is implemented via this blanket implementation:

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

The consumer trait `CanSerializeValue` is thus implemented for a context like `AppA` if `AppA` contains a lookup table entry where `ValueSerializerComponent` is the key and the resulting `Delegate` "value" successfully implements `ValueSerializer`.

Similarly, a blanket implementation is generated for `ValueSerializer` as follows:

```rust
#[cgp_impl(Provider)]
impl<Context, Value: ?Sized, Provider> ValueSerializer<Value> for Context
where
    Provider: DelegateComponent<ValueSerializerComponent>,
    Provider::Delegate: ValueSerializer<Context, Value>,
{
    fn serialize<S>(&self, value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Provider::Delegate::serialize(self, value, serializer)
    }
}
```

The blanket implementation above looks almost identical as before, except that the delegation lookup is done on the `Provider` type. This essentially allows a provider to delegate its provider implementation to *another* provider.

Following that, the special provider `UseDelegate` has the following blanket implementation:

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

This implementation shows that `UseDelegate` uses the `Value` type as the lookup "key" in a given components table, such as `SerializerComponentsA`.

If we carefully compare the three versions of the blanket implementations, we would observe that the key differences lie in which type is used as the type-level lookup table, and which type is used as the key for the lookup.

---

## Future Work

The initial release of `cgp-serde` serves as a compelling proof of concept, demonstrating how CGP can be used to solve the coherence problem in Rust. While you can certainly begin experimenting with `cgp-serde` today for modular serialization in your applications, there are still a few rough edges that need polishing before it reaches the quality level suitable for mission-critical production use.

This section highlights the areas we plan to address, and what you might want to wait for before fully committing to `cgp-serde` for your main projects.

### Serialization providers for extensible variants

Currently, `cgp-serde` has implemented providers like `SerializeFields` and `DeserializeRecordFields` to enable datatype-generic serialization for any struct that uses `#[derive(CgpData)]`. This decoupling of serialization logic from data type definitions is key to reducing the derive bloat caused by orphan rule restrictions.

However, the equivalent providers for Rust *enums* and *extensible variants* have not yet been implemented. This means that you cannot currently use the modular serialization features of `cgp-serde` to serialize enum types in your application. This limitation is purely due to time constraints; I was unable to dedicate enough time to finish the implementation for extensible variants before this initial release.

### Helpers for JSON deserialization

At the moment, `cgp-serde` only provides the `deserialize_json_string` helper method to deserialize a JSON string using a context. Crucially, I have not yet implemented other common helper methods, such as `from_slice` and `from_value`. If you need the functionality equivalent to these methods, you would currently have to study the internals of `deserialize_json_string` and write your own deserialization wrappers.

The need for additional wrappers during deserialization arises because functions like `serde_json::from_str` do not accept any argument where we can "pass" around the deserialization context. Therefore we must explicitly work around this by constructing library-specific deserializers like [`serde_json::Deserializer`](https://docs.rs/serde_json/latest/serde_json/struct.Deserializer.html) and then passing it along with the context to the `CanDeserializeValue::deserialize` method.

Fortunately, library functions like `serde::from_str` are generally lightweight wrappers around library-specific deserializers. This makes re-creating similar, easy-to-use helpers for `cgp-serde` a relatively straightforward task. The challenge here is simply a matter of time: I need to properly survey the common deserialization methods used in the wild and aim to support as many as possible. On the plus side, these wrapper implementations are low-hanging fruit and represent simple tasks for newcomers to contribute to the project. If you are interested in helping, please do submit a [pull request](https://github.com/contextgeneric/cgp-serde/pulls)!

### Helpers for other serialization formats

Just as custom deserialization wrappers are required for `serde_json`, we will likely need similar wrappers for other popular serialization formats, such as [`toml`](https://docs.rs/toml/latest/toml/).

In principle, serialization *from* `cgp-serde` should work almost immediately. If you use the `SerializeWithContext` wrapper with any serialization format, it should, theoretically, integrate seamlessly. However, this has not yet been thoroughly tested, so more verification is required. Assuming serialization works out of the box, the main task needed to support other formats will be implementing deserialization wrappers similar to what we have done for `serde_json`.

### Documentation

A significant area for improvement is documentation. Both CGP and `cgp-serde` are currently severely lacking in comprehensive documentation. To make `cgp-serde` truly usable for the broader community, we will need to write far more documentation and tutorials explaining how to effectively use it for modular serialization.

With my time being extremely limited, I will likely only prioritize documenting `cgp-serde` over further developing CGP if there is real, demonstrable demand from developers wanting to use it for their applications. While I strongly believe the modular serialization provided by `cgp-serde` will be incredibly useful, my experience with developing CGP suggests that the community may not yet fully grasp or care about modular serialization as much as I do. Therefore, if the use cases presented by `cgp-serde` are important to you, please communicate your feedback so I can properly prioritize my development efforts!

### Performance benchmark

Since `cgp-serde` exclusively employs static dispatch, I am highly confident that the serialization performance should align closely with the original `serde` implementation. However, I have not yet had the time to conduct a proper benchmark, so we currently lack concrete evidence of `cgp-serde`'s performance characteristics.

In addition to validation, there are potential optimizations that could further boost `cgp-serde`'s speed. Once proper benchmarking is done, I can apply targeted optimizations if any performance bottlenecks are clearly identified.

The primary point of contention in benchmarking will likely be the serialization and deserialization performance of struct fields. This is because `cgp-serde` uses extensible data types to provide a **generic** implementation of serialize and deserialize for any struct. In contrast, `serde` uses derive macros to generate **specific** implementations of `Serialize` and `Deserialize` tailored to each struct. The critical question, then, is whether our generic implementation can run as fast as the macro-generated, highly specific implementations.

There are a few reasons why the macro-generated implementation by `serde` might be faster, particularly during deserialization. `serde` generates a `match` statement on *string literals* to determine which field it needs to deserialize. Conversely, `cgp-serde` must perform a sequential string comparison of an incoming field key against each field's string tag and then choose the correct branch if a match is found. The Rust compiler can likely generate much more efficient, string-based pattern matching for `serde`.

We can only confirm if this gap exists by conducting a proper benchmark, specifically comparing scenarios like deserializing structs with many fields or fields with similar prefixes, to see if `cgp-serde`'s performance significantly worsens. If the performance difference is substantial, I will dedicate time to optimizing it. But if the difference is negligible, the current implementation is likely good enough.

One potential optimization I have considered is building a similar fast string matching table lazily using `LazyLock` when the first deserialization call occurs. We would need to build this table at runtime because our generic code can only inspect one field at a time, making it impossible to generate the same multi-string-literal `match` statement as a macro.

In any case, if you are interested in benchmarking or optimizing `cgp-serde`, your contributions to the project are highly welcome!

---

## Conclusion

In this article, we have provided a comprehensive preview of the powerful modular serialization features unlocked by `cgp-serde`. The most exciting part of this entire design is that almost nothing in `cgp-serde` is specifically engineered just for Serde or for serialization. Instead, everything you have learned here — from custom overlapping implementations to capabilities-enabled deserialization — is a direct result of the general design patterns offered by Context-Generic Programming for building any kind of application or library. This means you can easily take the same patterns used in `cgp-serde` and re-apply them to other traits and challenges within your own projects.

If this deep dive has piqued your interest in learning more about the fundamental concepts of CGP, please be sure to check out our [project homepage](/). In particular, we encourage you to read our articles on how CGP can be used to solve the famous [**Expression Problem**](https://en.wikipedia.org/wiki/Expression_problem) and how it enables the use of [**extensible records and variants**](/blog/extensible-datatypes-part-1/) in stable Rust. You can also explore how CGP is utilized to implement [**type-level DSLs**](/blog/hypershell-release/), using shell scripting as a practical example domain-specific language.

CGP is still in the early stages of development, so keep a close eye on the project's updates and progress. We are just getting started on redefining modularity in Rust!
