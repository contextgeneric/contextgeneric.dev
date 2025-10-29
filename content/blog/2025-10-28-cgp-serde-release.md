+++

title = "Announcing cgp-serde: A modular serialization library for Serde powered by CGP"

authors = ["Soares Chen"]

+++

# Overview

I am excited to announce the release of [**`cgp-serde`**](https://github.com/contextgeneric/cgp-serde), a modular serialization library for [Serde](https://serde.rs/) that is powered by [**Context-Generic Programming**](/) (CGP).

In a nutshell, `cgp-serde` extends the original [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) and [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html) traits of Serde, and makes it possible for anyone to bypass the **coherence restrictions** in Rust and write **overlapping** or **orphaned** implementations of these traits.

Additionally, `cgp-serde` enables us to use the [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) concepts in stable Rust today. This makes it possible for us write context-dependent implementations of `Deserialize`, such as one that uses an arena allocator to deserialize a `&'a T` value that is described in the proposal article.

## Quick intro to Context-Generic Programming

For readers who are new to the project, here is a quick introduction: Context-Generic Programming (CGP) is a modular programming paradigm that allows you to bypass the **coherence** restrictions in Rust traits, and write **overlapping** and **orphan** implementations for any CGP trait.

You can use CGP with almost any existing Rust trait today, by applying the `#[cgp_component]` macro on the trait. After that, you can write **named** implementation of the trait using `#[cgp_impl]`, which can be written without the coherence restrictions. Then, you can choose to use the named implementation for your type using the `delegate_components!` macro.

For example, in principle it is now possible to annotate the standard library’s [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html) trait with `#[cgp_component]`:

```rust
#[cgp_component(HashProvider)]
pub trait Hash { ... }
```

This does not affect existing code that uses or implements `Hash`, but it allows new overlapping implementations, such as one that works for any type that implements `Display`:

```rust
#[cgp_impl(HashWithDisplay)]
impl<T: Display> HashProvider for T { ... }
```

You can then reuse this implementation on any type using `delegate_components!`:

```rust
pub struct MyData { ... }
impl Display for MyData { ... }

delegate_components! {
    MyData {
        HashProviderComponent: HashWithDisplay,
    }
}
```

The example `MyContext` above implements the `Hash` trait by using `delegate_components!` to delegate the implementation to the `HashWithDisplay` provider, through the specified key `HashProviderComponent`. Because `MyData` already implements `Display`, the `Hash` trait is now also automatically implemented through CGP.

If you would like to learn more about CGP, check out the [project homepage](/) for more details. For now, let's head back and look at the new features introduced in `cgp-serde`.

---

# Context-Generic Serialization Traits

The key highlight for `cgp-serde` is that it introduces context-generic versions of the Serde traits. First, the [`Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html) trait is redefined as follows:

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

## `UseDelegate` Provider

Our CGP trait definitions also contain a second `derive_delegate` entry in `#[cgp_component]`. This generates a special `UseDelegate` provider that can be used for **static dispatch** of provider implementations based on the `Value` type. The use of `UseDelegate` will be explained later in this article.

---

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

---

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

In the code above, we use `delegate_components!` on to effectively create **type-level lookup tables** that configure the provider implementations used by `AppA`. The first key, `ValueSerializerComponent`, indicates that we are configuring the providers of `CanSerializeValue` for `AppA`.

The value of that entry is `UseDelegate`, followed by an *inner* table called `SerializerComponentsA` that performs **static dispatch** of the provider implementation based on the `Value` type. For example, we have the key `Vec<u8>` with the value `SerializeHex`, indicating that the `SerializeHex` provider is used when the `Vec<u8>` is being serialized.

The `delegate_components!` macro also allows shorthands for specifying multiple keys with the same value. For example, both `u64` and `String` are dispatched to the `UseSerde` provider, so we can group the keys using an array syntax. We can also have *generic* keys to be set in the table, such as the case of mapping all `&'a T` references to the `SerializeDeref` provider.

We will cover more details about how the type-level lookup table works in later sections. For now, let's look at how we implement `AppB` to perform the serialization for application B:

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

As we can see, the wiring configuration between `AppA` and `AppB` only require a **few lines of changes**, to have the change of serialization format. This demonstrates the flexibility of CGP to make application implementation highly configurable.

In practice, there are further CGP patterns available for `AppA` and `AppB` to share their common entries through a **preset**. But we will omit the details for brievity.

## Serialization with `serde_json`

`cgp-serde` remains **backward compatible** with the existing Serde ecosystem. This means that we can reuse existing libraries such as `serde_json` to serialize our encrypted message archive payloads into JSON.

However, since `serde_json` works only with `Serialize`, `cgp-serde` provides the `SerializeWithContext` wrapper to wrap the serialize value with the application context to provide a context-aware implementation of `Serialize`. With it, we can serialize our data to JSON as follows:

```rust
let app_a = AppA { ... };
let archive = MessagesArchive { ... };

let serialized_a = serde_json::to_string(
    &SerializeWithContext::new(&app_a, &archive)
).unwrap()
```

We first use `SerializeWithContext::new` to wrap both the application context and the target value together. We then pass it to `serde_json::to_string`, which accepts `SerializeWithContext` that provides a wrapped `Serialize` implementation.

Similarly, we can get a different JSON output by using `AppB` as the application context:


```rust
let app_b = AppB { ... };

let serialized_b = serde_json::to_string(
    &SerializeWithContext::new(&app_b, &archive)
).unwrap()
```

As we can see, `cgp-serde` makes it easy to customize the serialization of any field nested within other data types. By changing the application context, we are able to generate JSON output in different formats with minimal effort.

## Derive-free serialization with `#[derive(CgpData)]`

Aside from the deep customization provided by `cgp-serde`, another key feature to highlight is that there is no need to use derive macros to generate any serialize implementation for custom data types. If you look back at the definition of the types like `EncryptedMessage`, you will see that the it only uses a general `#[derive(CgpData)]` macro provided by CGP.

Behind the scene, `#[derive(CgpData)]` generates the support traits for [extensible data types](https://contextgeneric.dev/blog/extensible-datatypes-part-1/), and enables our data types to work with CGP traits like `CanSerializeValue` without requiring custom derivation. Instead, `cgp-serde` provides a `SerializeFields` provider that can work with any struct that derives `CgpData`.

This shows that `cgp-serde` also solves the orphan implementation problem, by not requiring libraries to derive library-specific implementations on their data types. For instance, our encrypted messaging library do not even need to include `cgp-serde` as its dependency. As long as the library uses the base `cgp` crate to derive `CgpData`, we will be able to serialize its data types through `SerializeFields`.

Further more, the use of extensible data types applies not only to the traits in `cgp-serde`. Instead, a general derivation of `CgpData` will enable the library data types to work with other CGP traits in similar ways as `cgp-serde`. Because of this, CGP can shield library authors from external requests to use derive macros for all popular traits, just to work around the orphan rules in Rust.

## Full Example

The full example of the customized serialization is available on [GitHub](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-tests/src/tests/messages.rs).

---

# Capabilities-Enabled Deserialization Demo

Now that we have demonstrated how `cgp-serde` enables modular serialization, let's also look at how `cgp-serde` enables the use case explained in the [**context and capabilities**](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal. In particular, we will demonstrate how we can implement a deserializer for `&'a T` using an arena allocator that is retrieved via **dependency injection** from the context.

## Coordinate Arena

With our arena deserializer defined, let's come up with an example application of how we could use it. Let's suppose that we want to store a large quantity of 3D coordinates, such as for rendering 3D graphics. We could define a `Coord` struct as follows:

```rust
#[derive(CgpData)]
pub struct Coord {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}
```

In this demo, the `Coord` struct only have 3 `u64` fields, but let's pretend that its actual size is much larger. If we use `Box<Coord>` to allocate the coordinates on the heap, it might lead to very high memory pressure with the frequent call to `Box::new(coord)`. Instead, we might want to use an [**arena allocator**](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/) to allocate the coordinates in a fixed memory region, with all the coordinates being deallocated easily at the end of the function.

When using arena allocators, we will get `&'a Coord` as the base coordinate value, which we can store in other data structures, such as a cluster of coordinates:

```rust
#[derive(CgpData)]
pub struct Cluster<'a> {
    pub id: u64,
    pub coords: Vec<&'a Coord>,
}
```

With our data structures defined, a challenge that comes next is: how can we deserialize a cluster of coordinates from formats such as JSON, and allocate the coordinates using a custom arena allocator that we provide?

## Arena Deserializer

To demonstrate the use of arena allocator, we will make use of the [`typed-arena`](https://docs.rs/typed-arena/latest/typed_arena/) crate to allocate memory using the [`Arena`](https://docs.rs/typed-arena/latest/typed_arena/struct.Arena.html) trait.

We will first define an *auto getter* trait to retrieve an `Arena` from the context:

```rust
#[cgp_auto_getter]
pub trait HasArena<'a, T: 'a> {
    fn arena(&self) -> &&'a Arena<T>;
}
```

The `HasArena` trait is automatically implemented for any `Context` type, if it derives `HasField` and contains an `arena` field with the field type being `&'a Arena<T>`. The nested reference is used here, because we need an explicit lifetime to work with the `Arena` type provided by `typed-arena`.

Next, we can make use of `HasArena` to retrieve the arena value from a generic context in our implementation of `ValueDeserializer`:

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
        let value = self.deserialize(deserializer)?;
        let value = self.arena().alloc(value);

        Ok(value)
    }
}
```

We define a new `DeserializeAndAllocate` that implements `ValueDeserializer` for any `&'a Value` type. To support that, it requires the `Context` to implement `HasArena<'a, Value>`, in addition to requiring the context to implement `CanDeserializeValue` for the owned `Value` type.

Inside the method body, we first use the `context` to deserialize an owned version of the value. We then call `self.arena()` to get the arena allocator, and use `alloc` to allocate the value on the arena.

As we can see, with the generalized dependency injection capability provided by CGP, we are able to retrieve any value or type from the context during deserialization. This effectively allows us to emulate the `with` clause in the [Context and Capabilities](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) proposal and provide any capability that is needed during deserialization.

## Deserialization Context

Using `cgp-serde`, it is straightforward to define a deserializer context that includes an arena allocator. We would first define the context as follows:

```rust
#[derive(HasField)]
pub struct App<'a> {
    pub arena: &'a Arena<Coord>,
}
```

Our `App` context is parameterized by a lifetime `'a`, and contains an `arena` field that contains a reference to a [`Arena`](https://docs.rs/typed-arena/latest/typed_arena/struct.Arena.html) with the lifetime `'a`, and the object type `Coord`.

The lifetime `'a` is necessary here, because the [`alloc`](https://docs.rs/typed-arena/latest/typed_arena/struct.Arena.html#method.alloc) method returns a `&'a Coord` value that has the same `'a` lifetime. By being explicit, we can better inform the Rust compiler that the coordinates would live as long as `'a`.

We also derive `HasField` on `App`, so that `App` would automatically implement the `HasArena` trait that we defined in the earlier section.

With the `App` context defined, let's take a look at the component wiring for the `ValueDeserializer` providers:

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

Similar to the type-level lookup tables earlier, this time we are configuring the `ValueDeserializer` providers for `App` through the `ValueDeserializerComponent` key, and the `UseDelegate` dispatcher. In this table, we have a lot of keys with generic `<'a>` lifetimes, since we are working with structs that contain explicit lifetimes.

As we can see from the table, for the value types `Coord` and `Cluster<'a>`, we use `DeserializeRecordFields` to deserialize the structs using the **extensible data types** facilities derived from `#[derive(CgpData)]`. But for `&'a Coord`, we choose the `DeserializeAndAllocate` provider that we have defined in the previous section.

## Error Handling

Aside from `ValueDeserializerComponent`, our `App` is also configured with **error handling** components provided by CGP. This is necessary, because we want to use `serde_json` to deserialize the value, which may return errors.

For simplicity, we opt to use the `cgp-error-anyhow` crate to handle errors using [`anyhow`](https://crates.io/crates/anyhow). In the entry for `ErrorTypeProviderComponent`, we use the `UseAnyhowError` provider to use the type [`anyhow::Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html) as the error type for `App`.

Following that, in the entry for `ErrorRaiserComponent`, we use `RaiseAnyhowError` to raise [`serde_json::Error`](https://docs.rs/serde_json/latest/serde_json/struct.Error.html) into `anyhow::Error` using its `From` implementation.

This demonstrates the flexibility provided by CGP on error handling - the concrete error type can be chosen by the application context, and it can also customize how each source error is handled.

## Deserializing JSON

Now that we have finished the component wiring for `App`, let's try to use `serde_json` to deserialize a JSON string. First, we will create a mock JSON string as follows:

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

And we will instantiate our arena and application context as follows:

```rust
let arena = Arena::new();
let app = App { arena: &arena };
```

For the case of deserialization, there are some complication involved, in that we cannot directly use the [`serde_json::from_str`](https://docs.rs/serde_json/latest/serde_json/fn.from_str.html) to deserialize a JSON string with our `App` context. Instead, `cgp-serde` also works with the lower-level [`Deserializer`](https://docs.rs/serde_json/latest/serde_json/de/struct.Deserializer.html) implementation in `serde_json`, so that we can pass `serde_json`'s deserializer directly to `CanDeserializeValue::deserialize`.

Fortunately, this implementation details are abstracted away by `cgp-serde`, and all we have to do is to call the `deserialize_json_string` method on our `App` context:

```rust
let deserialized: Cluster<'_> = app
    .deserialize_json_string(&serialized)
    .unwrap();
```

As we can see, we have now successfully make use of the custom arena allocator provided by our `App` context to perform deserialization.

## Full Example

The full example of the arena allocator deserialization is available on [GitHub](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde-tests/src/tests/arena_simplified.rs).

<!-- This is because unlike the `serialize` method which contains a `value` argument, there is no where that we can "wrap" our context in the `deserialize` method.

Instead, `cgp-serde` wraps the context throughout the deserialization process by using the [`DeserializeSeed`](https://docs.rs/serde/latest/serde/de/trait.DeserializeSeed.html) trait, which accepts an additional `self` argument that we can use to wrap the context type.   -->

---

# Implementation Details

In this section, we are going to explore some of the implementation details of CGP that enables the level of modularity for `cgp-serde`. For audiences who are new to CGP, you can use this section to quickly learn about the basic concepts of CGP that are relevant to be used with `cgp-serde`.

## Provider Traits

When the `#[cgp_component]` macro is used on a consumer trait like `CanSerializeValue`, it generates a companion provider trait `ValueSerializer` as follows:

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

## Desugaring of `#[cgp_impl]`

The overlapping implementations of providers like `UseSerde` and `SerializeWithDisplay` are made possible through the use of the `ValueSerializer` provider trait. Although they look like blanket implementations, behind the scene, a provider implementation like `SerializeWithDisplay` is desugared by `#[cgp_impl]` into the following:

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

## Type-Level Lookup Tables

When `delegate_components!` is used on a context like `AppA`, it generates constructs that are conceptually equivalent to constructing the following type-level lookup table on `AppA`:

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

The main difference is that CGP's lookup tables are implemented at the type-level, meaning that the tables don't exist at runtime and thus has **no runtime overhead**.

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

CGP also generates **blanket implementations** on the consumer and provider traits that make use of the lookup table entries in `DelegateComponent` to figure out how to lookup for a provider implementation at compile time.

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

---

# Future Work

The initial release of `cgp-serde` serves as a proof of concept showcase of how CGP can be used to solve the coherence problem in Rust. In principle, you can already experiment on using `cgp-serde` today for modular serialization of data types in your applications. However, there are still some rough edges that need to be polished up, before `cgp-serde` can reach the level of quality that is suitable for production use.

This section highlights some of the work that you might want to wait for before using `cgp-serde` for mission critical applications.

## Serialization providers for extensible variants

Currently, `cgp-serde` provides providers such as `SerializeFields` and `DeserializeRecordFields` to support datatype-generic serialization of any struct that use `#[derive(CgpData)]`. This allows the serialization logic to be fully decoupled from the data type definitions, and reduce the derive bloats caused by orphan rules restrictions.

However, I have not yet implemented the equivalent providers for *enums* and *extensible variants*. This means that you cannot yet use the modular serialization provided by `cgp-serde` to serialize enum types in your application.

Note that this limitation is simply due to time constraints, as I couldn't find enough time to finish the implementation for extensible variants in time for this initial release.

## Helpers for JSON deserialization

Currently, `cgp-serde` only provide the `deserialize_json_string` helper method to deserialize JSON string with a deserialization context. However, I haven't implemented the other helper methods, such as `from_slice` and `from_value`. This means that if you want to use the equivalent of these methods, you would have to read into the internals of `deserialize_json_string` and write your own deserialization wrappers.

The main reason why additional wrappers are needed for deserialization is because there is no `self` argument in the original `deserialize` method in `serde::Deserialize`. So instead, we need to work around that by explicitly constructing a library-specific [`Deserializer`](https://docs.rs/serde_json/latest/serde_json/struct.Deserializer.html), and then pass it to the `deserialize` method in `CanDeserializeValue` together with the context.

Fortunately, since library functions like `serde::from_str` are just lightweight wrappers around the library-specific deserializers, we can re-create similar helpers for `cgp-serde` without much problems.

Nevertheless, the challenge here is just a matter of time constraint, as I have to first properly survey the common deserialization methods that are used in the wild, and try to support as many of them as I could.

On the plus side, the wrapper implementations are low hanging fruits that should be simple enough for newcomers to contribute to the project. So do submit a [pull request](https://github.com/contextgeneric/cgp-serde/pulls) if you are interested in contributing!

## Helpers for other serialization formats

Just as deserialization wrappers are needed for `serde_json`, we will probably also need deserialization wrappers for other popular serialization formats, such as [`toml`](https://docs.rs/toml/latest/toml/).

In principle, support for serialization from `cgp-serde` should work out of the box, if you use the `SerializeWithContext` wrapper with any serialization format. However, I have not yet thouroughly tested it, and so more verification is required.

If serialization just work, then the main work that is required to support other serialization formats would be to implement similar deserialization wrappers as what we have done for `serde_json`.

## Documentation

Both CGP and `cgp-serde` are currently severly lacking in documentation. To make `cgp-serde` usable to the broader community, we will likely need to write a lot more documentation and tutorials on how to use `cgp-serde` for modular serialization.

On the other hand, since my time is very limited, I will likely only prioritize documenting `cgp-serde` over developing CGP if there is a real demand of developers wanting to use `cgp-serde` for their applications. While I do believe that the modular serialization provided by `cgp-serde` is going to be very useful, my experience of developing CGP also tells me that perhaps the community do not care as much about modular serialization as I do.

So if the use cases presented by `cgp-serde` is important enough for you to care, please do communicate your feedback so that I can properly prioritize my work!

## Performance benchmark

Since `cgp-serde` exclusively uses only static dispatch, I am pretty confident that the serialization performance should be very close to the original `serde` implementation. However, I haven't have the time yet to do proper benchmark. So we don't have concrete evidence that `cgp-serde` is highly performant.

Additionally, there are some potential optimizations that could be done to further improve the performance of `cgp-serde`. So as proper benchmark is being done, I might also apply some further optimization in case if I can properly identify the slow paths.

In particular, the main contention point in the benchmark would likely be on the serialization and deserialization performance of the struct fields. This is because `cgp-serde` uses extensible data types to have a **generic** implementation of serialize and deserialize for any struct. On the other hand, `serde` makes use of derive macros to generate **specific** implementation of `Serialize` and `Deserialize` for each struct. Because of this, the main question is that can the generic implementation of serialize and deserialize run as fast as the macro-generated implementations.

There are a few reasons why the macro-generated implementation by `serde` could run faster than `cgp-serde`, particularly during deserialization. This is because `serde` generates `match` statement on *string literals* to determine which field it needs to serialize. On the other hand, `cgp-serde` needs to perform sequential string comparison of an incoming field key with each string tag, and choose a specific branch if the string value matches. As a result, the Rust compiler can probably generate much more efficient string-based pattern matching to determine the field that needs to be deserialized.

We can only know if that is the case by doing proper benchmark, and do comparison like whether having many struct fields with similar prefixes could significantly worsen the deserialization performance of `cgp-serde`. If the performance difference is big enough, I will probably spend some time to try optimizing it. But if the difference is not too significant, it may be good enough to leave the current implementation as-is.

A potential optimization that I have in mind is that we can probably build similar fast string matching table lazily using `LazyLock` when the first deserialization is called. The reason we need to build the table at runtime is because our generic code can only look at one field at a time, and thus cannot generate something that is equivalent to a match statement with multiple string literals.

In any case, if you are interested in benchmarking or optimizing `cgp-serde`, you are also very welcome to contribute to the project!
