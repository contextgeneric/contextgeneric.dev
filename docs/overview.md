---
title: 'Overview of the features, with examples'
sidebar_label: 'Overview'
sidebar_position: 3
description: 'A tour of what Context-Generic Programming adds to Rust: the five headline features, each with a compiled example and the condition it holds under, then what lies beyond them.'
---

# Overview

[Context-Generic Programming](/docs/) (CGP) is a language extension for Rust, with pluggable trait
implementations at compile-time. This page is a tour of what that gives you. It takes the five
headline features one at a time, and for each one it starts from a problem you may already have,
shows a short example, says where the feature stops helping, and links the page that explains it in
full. A short list after them names what CGP offers beyond the five, for when they feel familiar.
If you would rather run a program first, start with the [Quickstart](/docs/quickstart).

Every example is a fragment of a program that compiles against `cgp` 0.8.0. The fragments leave out
the imports, the definitions they do not need to show, and the check each context needs, which
*Type-Safe Wiring* explains. They also share one scenario, the one the
[`cgp` README](https://github.com/contextgeneric/cgp/blob/main/README.md) uses: an application that
fetches users' profile pictures from object storage. The examples skip the requests to each storage
service, because the features are about how the code is arranged rather than what it does.

## The headline features

### One Interface, Many Implementations

Suppose the application stores its pictures in Amazon S3, a second deployment of it uses Google
Cloud Storage, and its tests read from a local folder. In plain Rust, each application type can
implement a fetching trait itself, but then the S3 code belongs to that one type. Reusing it means
writing a blanket implementation for every type with an S3 client and another for every type with a
Google Cloud client. Rust rejects that pair, because one type could have both clients.

CGP avoids the conflict by giving each implementation a type of its own, called a **provider**.
Because the providers are different types, their implementations never overlap. Here is the
interface, defined as a CGP **component**, and the provider that fetches from S3:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>>;
}

#[cgp_impl(new FetchS3Object)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] storage_client: &aws_sdk_s3::Client,
        #[implicit] bucket_id: &str,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        // Fetch the object from Amazon S3.
    }
}
```

The attribute keeps `CanFetchStorageObject` as the trait that callers use, and it generates
`StorageObjectFetcher`, the trait that each provider implements. The `#[implicit]` arguments read the
client and the bucket from the fields of whichever type uses the provider, so a caller passes only
the object's ID.

Two more providers follow the same pattern: `FetchGCloudObject` reads a Google Cloud `Storage`
client from the same `storage_client` field, and `FetchLocalObject` reads files from a `local_dir`
field for tests. Each application type, which CGP calls a **context**, then chooses one provider in
its **wiring**:

```rust
delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchS3Object,
    }
}

delegate_components! {
    GCloudApp {
        StorageObjectFetcherComponent: FetchGCloudObject,
    }
}

delegate_components! {
    TestApp {
        StorageObjectFetcherComponent: FetchLocalObject,
    }
}
```

Code that calls `fetch_storage_object` stays the same in all three contexts. That does not mean
Rust's coherence rules have been disabled. They still apply to everything the macros generate, and
CGP only changes how the implementations are arranged, so that they no longer conflict.
[Bypassing coherence](/docs/concepts/coherence) explains the rules, and
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) shows how a call reaches
the chosen provider.

### Zero-Cost Abstraction

A natural question about choosing implementations is what the choice costs at runtime. The answer is
nothing, because the compiler resolves the wiring into a direct call. `App`'s wiring above amounts to
the following implementation, which you could also write by hand:

```rust
impl CanFetchStorageObject for App {
    async fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        <FetchS3Object as StorageObjectFetcher<App>>::fetch_storage_object(self, object_id)
            .await
    }
}
```

Nothing sits between the call and the provider: no runtime registry, no reflection, and no virtual
method table. The provider itself is a type that is never instantiated. Its own work, such as the
request to S3, still runs, and generic code can add compile time and code size. If you want to see
what the macros generate, [`cargo cgp expand`](/docs/cargo-cgp/expand) prints it, and
[Dynamic dispatch](/docs/comparisons/dynamic-dispatch) compares this approach with `dyn Trait`.

### Type-Safe Wiring

Choosing providers is only useful if a wrong choice is caught early. With CGP it is caught at compile
time: a context that cannot give its provider what it needs fails to build once its wiring is
checked. For example, `FetchS3Object` reads a `bucket_id` field, so wiring it on a context that
lacks one, and checking that wiring with `check_components!`, is a compile error:

```rust
#[derive(HasField)]
pub struct App {
    pub storage_client: aws_sdk_s3::Client,
}

delegate_components! {
    App {
        StorageObjectFetcherComponent: FetchS3Object,
    }
}

check_components! {
    App {
        StorageObjectFetcherComponent,
    }
}
```

[cargo-cgp](/docs/cargo-cgp), CGP's error toolchain, reports the missing field first, followed by
the chain of requirements that led to it. Here is its report, without the source excerpt that points
at the check:

```text
error[E0277]: [CGP-E001] missing field `bucket_id` on `App`
   = note: root cause: [CGP-E110] missing field `bucket_id` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanFetchStorageObject` for context `App`
             └─ [CGP-E102] provider trait impl `StorageObjectFetcher` with context `App` for provider `FetchS3Object`
               └─ [CGP-E110] missing field `bucket_id` on `App`
   = help: add the field `bucket_id` to `App`
```

The check matters, because wiring on its own checks nothing. An entry compiles until the component
is used or checked, so check each context where you assemble it. For a simple context,
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) wires each
entry and checks it in the same step. Even then, the check establishes that the components fit
together, not that the application is correct.

Reading the failure has a cost too. Without the tool, the raw compiler error for this mistake is
longer, and it spells the field's name as a nested type. `cargo cgp check` leads with the root
cause for the classes it recognizes, and the tool does not yet reshape every class.
[Check traits](/docs/concepts/check-traits) explains the checks, the
[checking tutorial](/docs/tutorials/area-calculation/checking) walks through a mis-wiring, and
[Dependency injection](/docs/comparisons/dependency-injection) compares wiring with a container.

### Abstract Over Every Dependency

So far the component has returned `anyhow::Result`, which ties every implementation to one error
type. Core logic does not have to fix that choice. It can name the dependencies it needs, such as
an error type, and leave each context to supply them. Here the same component uses the context's
error type instead of `anyhow::Error`, and a provider declares the conversion it needs with
`#[uses]`, without adding it to the interface:

```rust
#[async_trait]
#[cgp_component(StorageObjectFetcher)]
#[use_type(HasErrorType.Error)]
pub trait CanFetchStorageObject {
    async fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
}

#[cgp_impl(new FetchLocalObject)]
#[uses(CanRaiseError<std::io::Error>)]
#[use_type(HasErrorType.Error)]
impl StorageObjectFetcher {
    async fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> Result<Vec<u8>, Error> {
        std::fs::read(format!("{local_dir}/{object_id}")).map_err(Self::raise_error)
    }
}
```

The S3 and Google Cloud providers change in the same way: each returns `Error` and declares, with
`#[uses]`, the conversion it needs from its client's error. A context then chooses its error type in
a single wiring entry, such as `ErrorTypeProviderComponent: UseType<anyhow::Error>`, next to the
providers that convert each source error into it. Another context can choose an error enum of its
own without touching this code, as long as it supplies those conversions as well.

The same arrangement works for other dependencies, such as an async runtime, and it lets a core
library stay `no_std` as long as the providers chosen for it are `no_std` too. A provider that reads
files, like this one, still needs `std`, and its blocking read suits a test context, where an
application would use an async file API.
[Modular error handling](/docs/concepts/modular-error-handling) explains how a context chooses its error type, and
[Impl-side dependencies](/docs/concepts/impl-side-dependencies) explains how a provider keeps its
needs out of the interface.

### Still Ordinary Rust

None of this asks you to leave ordinary Rust behind. CGP is a library on stable Rust, and its
simplest form is a function whose `#[implicit]` arguments come from the context's fields:

```rust
#[cgp_fn]
#[async_trait]
pub async fn get_user(
    &self,
    #[implicit] database: &PgPool,
    user_id: &UserId,
) -> anyhow::Result<User> {
    // Query `database` for the user row.
}
```

`#[cgp_fn]` turns the function into a trait with a
[blanket implementation](/docs/reference/glossary#blanket-implementation), so any context with a
`PgPool` field named `database` gains `get_user`, with no wiring at all. The rest of CGP stays close
to ordinary Rust too. Providers use familiar `impl` syntax, and the trait that callers use remains
an ordinary trait, so the hand-written implementation under *Zero-Cost Abstraction* is a valid way
to implement it directly. This is why you can adopt CGP one component at a time. The price is
learning the provider model and maintaining the wiring.
[Implicit arguments](/docs/concepts/implicit-arguments) explains the field reads, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) helps you judge how much of CGP a
problem needs.

## Beyond choosing implementations

Choosing implementations is the core of CGP, but the same machinery reaches further. You need none of
what follows to start, and each part is easier to learn once the features above feel familiar, so
this page names them and leaves their examples to the pages that explain them:

- **Abstract types:** a context chooses types as well as implementations. When several traits import
  the context's error type, they all agree on it, so a function that uses them needs no bound to
  equate their errors, and the error type stops being a parameter that every layer has to carry.
  [Abstract types](/docs/concepts/abstract-types) explains how.
- **Extensible records and variants:** a derive exposes a struct's fields or an enum's variants to
  the trait system, so generic code can build a record from separate pieces, or call a trait on an
  enum through whichever variant it holds without writing a `match`.
  [Extensible records](/docs/concepts/extensible-records),
  [Extensible variants](/docs/concepts/extensible-variants), and
  [Dispatching](/docs/concepts/dispatching) cover the patterns.
- **Composable handlers:** a family of components for computations, synchronous or async, fallible
  or not, whose providers chain into pipelines that a context chooses in its wiring.
  [Handlers](/docs/concepts/handlers) covers the family.
- **Namespaces:** a wiring table that several contexts share, so each context lists only the entries
  where it differs. [Namespaces](/docs/concepts/namespaces) explains them, along with the lighter
  `open` statement for choosing a provider per type.
- **Higher-order providers:** a provider that takes another provider as a parameter, such as a
  wrapper that scales whatever area its inner provider computes.
  [Higher-order providers](/docs/concepts/higher-order-providers) shows how they compose.

## The problems it solves

The features above are easier to judge against the problems you have. CGP helps most where
a program already needs several implementations, or where dependencies spread through its
signatures. The table below starts from the problem and links the section of this page that covers
it, where there is one, along with the Concepts page that explains it:

| Problem | On this page | Explained in |
| --- | --- | --- |
| Rust rejects two implementations of a trait for the same type | [One Interface, Many Implementations](#one-interface-many-implementations) | [Bypassing coherence](/docs/concepts/coherence) |
| A trait for a type you do not own needs a newtype | Not shown here | [Bypassing coherence](/docs/concepts/coherence) |
| Tests need a different implementation from production | [One Interface, Many Implementations](#one-interface-many-implementations) | [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) |
| Feature flags already select between backends | [One Interface, Many Implementations](#one-interface-many-implementations) | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| Changing the error type or runtime touches every layer | [Abstract Over Every Dependency](#abstract-over-every-dependency) | [Modular error handling](/docs/concepts/modular-error-handling) |
| A trait has grown until no implementation needs all of it | Not shown here | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| An implementation's dependencies leak into the public interface | [Abstract Over Every Dependency](#abstract-over-every-dependency) | [Impl-side dependencies](/docs/concepts/impl-side-dependencies) |
| Generic parameters and their bounds repeat through every layer | [Beyond choosing implementations](#beyond-choosing-implementations) | [Abstract types](/docs/concepts/abstract-types) |
| Generic code needs to work over any struct or enum | [Beyond choosing implementations](#beyond-choosing-implementations) | [Extensible records](/docs/concepts/extensible-records) |
| A wiring error buries its cause in generated types | [Type-Safe Wiring](#type-safe-wiring) | [Check traits](/docs/concepts/check-traits) |

## When a plain trait is enough

All of this comes at a cost, so for a trait with only one implementation, a plain trait is the right
tool. CGP adds declarations and wiring that a plain trait does not need, compile-time work for its
macros and type checking, and long raw errors. `cargo cgp check` leads with the root cause for the
classes it recognizes, and the tool does not yet reshape every class. Learning the patterns also
takes time. If you work with an assistant, [CGP's agent skill](/docs/ai/skills/) helps it with the
vocabulary, the diagnostics, and the wiring, although you still need to review what it writes. The
[front page's costs](/#costs) state the project's maturity, and
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) explains when a plain trait is enough
and when providers start to help.

## Where to go next

Where you go from here depends on which part of this tour caught your interest:

- **Run code:** the [Quickstart](/docs/quickstart), then the
  [Hello World tutorial](/docs/tutorials/hello).
- **Understand the mechanism:** the [Concepts](/docs/concepts), starting with
  [Bypassing coherence](/docs/concepts/coherence).
- **Compare with what you know:** the [Comparisons](/docs/comparisons).
- **Look up a construct:** the [Reference](/docs/reference).

---

*An AI agent wrote this page using the CGP knowledge base, and its examples were verified by
compiling them. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
