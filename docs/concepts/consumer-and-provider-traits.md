---
title: 'Connect a method call to the provider it selects'
sidebar_label: 'Consumer and provider traits'
sidebar_position: 2
description: 'How CGP splits one trait into a consumer trait for callers and a provider trait for implementations, and how wiring connects a call to its provider.'
---

# Consumer and provider traits

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations: a
component connects callers to implementations through two related traits. Callers use the
**consumer trait** to invoke methods on a context, the type receiving the call. Implementers use
the **provider trait** to supply those methods through separate provider types. The context chooses
a provider through wiring, so implementations can vary while callers keep the same interface.

This page follows the connection between the two traits, from a method call through the generated
forwarding implementations to the chosen provider. It then shows how the macros produce those
connections, where application state lives, and what the separation costs.

## Ordinary Rust traits

An ordinary Rust trait lets different types expose the same method with different behavior.
For example, a production application may fetch stored objects from Amazon S3, while a test
application reads them from a local folder. Each type can implement `CanFetchStorageObject`,
giving callers a shared interface:

```rust
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

pub struct App {
    pub storage_client: aws_sdk_s3::Client,
    pub bucket_id: String,
}

pub struct TestApp {
    pub local_dir: String,
}

impl CanFetchStorageObject for App {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        // fetch `object_id` from `self.bucket_id` through `self.storage_client`
    }
}

impl CanFetchStorageObject for TestApp {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(format!("{}/{object_id}", self.local_dir))?)
    }
}
```

Here, `App` and `TestApp` are the contexts on which `fetch_storage_object` operates. Each represents
an application and carries the data its implementation needs: an S3 client and a bucket for `App`,
and a folder for `TestApp`. The S3 request is omitted, while the test implementation reads the
object from a file in its folder. To keep the example short, the method is synchronous, although a
real storage client would be async, as
[Swap a Storage Backend per Application](/docs/tutorials/storage-backend) shows.

To share each behavior with other application types, it might seem natural to write a
[blanket implementation](/docs/reference/glossary#blanket-implementation). The S3 implementation
could work for any type that has an S3 client, expressed as a `HasS3Client` bound, and the local
implementation for any type that names a folder, through `HasLocalDir`. Offering both, however,
creates a conflict:

```rust
impl<T: HasS3Client> CanFetchStorageObject for T { /* ... */ }
impl<T: HasLocalDir> CanFetchStorageObject for T { /* ... */ }   // error[E0119]
```

A type could satisfy both bounds, which would leave Rust with two implementations of
`CanFetchStorageObject` for that type, so the compiler rejects the pair. Helper functions and other
composition techniques can still share the code, with each application connecting its trait
implementation to the behavior it wants. CGP makes that connection through a separate trait for
reusable implementations and a wiring table that selects one.

## Splitting one trait in two

The consumer trait keeps the caller's interface, including the original methods and receiver.
The provider trait moves the consumer's `Self` into a `Context` parameter. Its methods receive
the context as an explicit argument, leaving the implementing type free to name the behavior.

The signatures below show the split. They omit a generated checking bound on the provider trait,
which the call trace below names:

```rust
// what callers use
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

// what implementations target
pub trait StorageObjectFetcher<Context> {
    fn fetch_storage_object(
        context: &Context,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>>;
}
```

The diagram shows the two roles and the provider structs that implement the second one.

![Diagram: one trait splits into a consumer trait that callers use and a provider trait that implementations target. Two provider structs implement the provider trait, and a call through the consumer trait goes through the context's wiring, which selects one of them.](/img/consumer-provider-split.svg)

The implementations can now keep their original context bounds while using separate marker
structs as `Self`. With the method bodies omitted, their declarations look like this:

```rust
pub struct FetchS3Object;
pub struct FetchLocalObject;

impl<Context: HasS3Client> StorageObjectFetcher<Context> for FetchS3Object {
    /* ... */
}

impl<Context: HasLocalDir> StorageObjectFetcher<Context> for FetchLocalObject {
    /* ... */
}
```

Even if a context satisfies both `HasS3Client` and `HasLocalDir`, these implementations have
different `Self` types and can coexist. Each local provider struct also lets its crate implement a
provider trait defined elsewhere. The split therefore follows ordinary Rust coherence rules, as
[Bypassing coherence](./coherence.md) explains.

The same split applies to a component with several methods, associated types, or constants. Those
items can stay together when one provider choice should determine their implementation; this
example uses a single method to make the relationship easier to see.

## Providers as type-level names

The structs `FetchS3Object` and `FetchLocalObject` serve only as names for implementations. They are
zero-sized, and CGP neither constructs values of them nor stores application data in them. Instead,
that data stays in the context, while explicit method arguments supply inputs such as the object's
ID.

Because the state belongs to the context, it helps to distinguish the two meanings of `Self`
when reading provider code. In [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), the method's
`self` receiver refers to the context, and the macro rewrites references to `Self` to the context
type. The provider's own associated types and constants retain their provider meaning. In a raw
Rust provider implementation, `Self` simply names the provider type following `for`.

## How a call finds its provider

With the behaviors defined, each application needs to choose which provider supplies its
`fetch_storage_object` method. Wiring records that choice in a table of types that the compiler
resolves:

```rust
delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchS3Object,
    }
}

delegate_components! {
    TestApp {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}
```

The table maps the same component key to a different provider for each context.

![Diagram: the wiring table has three columns, context, component key, and provider. App maps StorageObjectFetcherComponent to FetchS3Object, and TestApp maps it to FetchLocalObject. A call resolves through the table at compile time to a direct call on the selected provider.](/img/wiring-table.svg)

The key in both tables is `StorageObjectFetcherComponent`, a marker generated alongside the two
traits. Those traits, the marker, and the forwarding implementations form the **component**. The
entry for this key selects `FetchS3Object` on `App` and `FetchLocalObject` on `TestApp`.

When a caller writes `app.fetch_storage_object("alice.png")`, the compiler follows `App`'s selection
to `FetchS3Object` and checks whether that provider can work with `App`. Because this is static
dispatch, the running program needs neither a wiring table nor a provider lookup.

To see how method syntax reaches that selection, consider the two generated blanket
implementations. The consumer implementation supplies `CanFetchStorageObject` whenever the context
implements `StorageObjectFetcher` for itself:

```rust
impl<Context> CanFetchStorageObject for Context
where
    Context: StorageObjectFetcher<Context>,
{
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        Context::fetch_storage_object(self, object_id)
    }
}
```

Satisfying `Context: StorageObjectFetcher<Context>` leads to the second implementation. It
gives a delegating type the provider trait of the type selected by its table. Here,
`DelegateComponent` represents the table entry, and its `Delegate` associated type names the
selected provider:

```rust
impl<Context, Provider> StorageObjectFetcher<Context> for Provider
where
    Provider: DelegateComponent<StorageObjectFetcherComponent>
        + IsProviderFor<StorageObjectFetcherComponent, Context, ()>,
    Provider::Delegate: StorageObjectFetcher<Context>,
{
    fn fetch_storage_object(
        context: &Context,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        Provider::Delegate::fetch_storage_object(context, object_id)
    }
}
```

For the call on `App`, both `Context` and `Provider` initially stand for `App`. In this
implementation, `Provider` names the type whose table is consulted, so its `Delegate` is
`FetchS3Object`. As long as `FetchS3Object` implements `StorageObjectFetcher<App>`, the delegation
gives `App` its own `StorageObjectFetcher<App>` implementation, which in turn supplies
`CanFetchStorageObject` through the first blanket implementation. The context stays `App`
throughout, so the final provider receives the same application value on which the caller invoked
the method.

The other bound, [`IsProviderFor`](/docs/reference/traits/wiring/is_provider_for), is the checking
bound omitted from the earlier `StorageObjectFetcher` declaration. Provider macros generate it with each
provider's bounds, and [Checking your wiring](./check-traits.md#the-isproviderfor-marker) explains how a
check uses it to name a missing requirement. [Type-level tables](./type-level-tables.md) shows, in plain
Rust, how a trait with an associated type works as a table the compiler looks up.

The listings use shorter names to keep this connection readable. In an actual expansion, the
parameters are `__Context__` and `__Provider__`, and the delegate is spelled as
`<__Provider__ as DelegateComponent<StorageObjectFetcherComponent>>::Delegate`. You can inspect the
full expansion of a concrete component with [`cargo cgp expand`](/docs/tooling/cargo-cgp/expand).

## Writing a component

The macros generate these connections while letting you write methods from the context's point
of view. `#[cgp_component]` produces the trait pair and forwarding implementations, and
`#[cgp_impl]` lets each provider keep a familiar receiver. For the storage example, that gives:

```rust
use cgp::prelude::*;

#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

#[cgp_impl(new FetchS3Object)]
impl StorageObjectFetcher {
    fn fetch_storage_object(
        &self,
        #[implicit] storage_client: &aws_sdk_s3::Client,
        #[implicit] bucket_id: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        // fetch `object_id` from `bucket_id` through `storage_client`
    }
}

#[cgp_impl(new FetchLocalObject)]
impl StorageObjectFetcher {
    fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(format!("{local_dir}/{object_id}"))?)
    }
}

#[derive(HasField)]
pub struct App {
    pub storage_client: aws_sdk_s3::Client,
    pub bucket_id: String,
}

#[derive(HasField)]
pub struct TestApp {
    pub local_dir: String,
}

delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchS3Object,
    }
}

delegate_components! {
    TestApp {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}
```

Each `#[implicit]` argument reads a same-named field exposed by the context's `HasField` derive.
The macro turns it into a local binding and removes it from the public signature, so callers still
write `app.fetch_storage_object(object_id)`. The S3 request is still a placeholder, while the local
provider reads the object from a file in the test application's folder.

The fields differ between providers: `FetchS3Object` needs `storage_client` and `bucket_id`, while
`FetchLocalObject` needs `local_dir`. Because these are
[impl-side dependencies](./impl-side-dependencies.md), neither requirement appears in
`CanFetchStorageObject`. A caller can therefore use the selected behavior with that trait bound
alone:

```rust
pub fn fetch_profile_picture(
    app: &impl CanFetchStorageObject,
    object_id: &str,
) -> anyhow::Result<Vec<u8>> {
    app.fetch_storage_object(object_id)
}
```

The same `fetch_profile_picture` function accepts `App` as well as `TestApp`. Its bound names only
`CanFetchStorageObject`, so each application's wiring supplies the behavior without the function
knowing the provider or its fields.

Because the consumer trait is still an ordinary Rust trait, a context can also implement it
directly and omit providers and wiring, as at the start of the page. This lets an application
adopt components one trait at a time. For a given context, however, a direct implementation would
overlap with a usable wired implementation of the same consumer trait, so it cannot have both.

## What it costs

For an operation with one implementation, the trait pair and wiring may add more machinery than
you need. Use a plain trait or consider [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), which generates
a trait and blanket implementation from a function without component wiring.

When you do use wiring, the compiler checks whether it supplies a usable implementation once
code requires the trait. Until then, a missing entry or unsatisfied provider dependency may remain
undetected. [`check_components!`](/docs/reference/macros/check_components) brings those checks to
a place you choose, usually beside the table. If one fails,
[`cargo cgp check`](/docs/tooling/cargo-cgp/check) leads with the root cause for the classes it recognizes,
though it does not yet reshape every class. [Checking your wiring](./check-traits.md) explains
those diagnostics and their limits.

Even with static dispatch, a reader tracing the call has to find the table entry and follow it
to the provider. The split is useful when it lets several contexts reuse a provider or select
different behaviors while a caller such as `fetch_profile_picture` keeps the same interface.

## Where to go next

To apply the same separation in your own code, consider the dependencies each provider needs
and the choices each context must make. These pages develop those ideas and show the component
model in practice:

- [Bypassing coherence](./coherence.md): Why distinct provider types permit reusable alternatives.
- [Type-level tables](./type-level-tables.md): How a trait works as a compile-time table, in plain
  Rust.
- [Checking your wiring](./check-traits.md): The checking bound on the provider trait, and how a check
  uses it.
- [Impl-side dependencies](./impl-side-dependencies.md): How a provider states requirements without
  adding them to the caller's interface.
- [Area calculation tutorial](/docs/tutorials/area-calculation/): Building the same split from plain
  functions and calling providers before introducing wiring.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component),
  [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
  [`delegate_components!`](/docs/reference/macros/delegate_components): Exact syntax and generated code.
- [Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch): How compile-time wiring compares with a runtime vtable.
- [Comparison: Policy-based design](/docs/comparisons/policy-based-design): The corresponding split in C++ policy classes and host templates.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
