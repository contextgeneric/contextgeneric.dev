---
title: 'Overview of the features, with examples'
sidebar_label: 'Overview'
sidebar_position: 3
description: 'Explore Context-Generic Programming through Rust examples, from choosing implementations to checking wiring and abstracting over dependencies.'
---

# Overview

When deployments and tests need different implementations of the same interface, sharing code can
become difficult. [Context-Generic Programming](/docs/) (CGP) is a language extension for Rust, with
pluggable trait implementations at compile-time. The examples below show how those choices work, how
the compiler checks them, and where the approach has limits.

If you are new to CGP, you can begin with its simplest form under
[*Still Ordinary Rust*](#still-ordinary-rust), or run a program with the
[Quickstart](/docs/quickstart).

The examples follow an application that fetches users' profile pictures from object storage, as in
the [`cgp` README](https://github.com/contextgeneric/cgp/blob/main/README.md). Each is a fragment of
a program that compiles against `cgp` 0.8.0. To focus on how the code fits together, the fragments
omit imports, supporting definitions, and requests to the storage services. They also omit the
context checks explained under [*Type-Safe Wiring*](#type-safe-wiring). The methods are synchronous
to keep the fragments short. A real storage client would be async, as in
[Swap a Storage Backend per Application](/docs/tutorials/storage-backend), which builds the
example step by step.

## Key features

### One Interface, Many Implementations

Suppose one deployment stores its pictures in Amazon S3, another uses Google Cloud Storage, and
tests read from a local folder. In plain Rust, each application type can implement a fetching trait
itself, but then the S3 code belongs to that one type. To share that code, you might try writing a
blanket implementation for every type with an S3 client and another for every type with a Google
Cloud client. Rust rejects that pair, because one type could have both clients.

CGP avoids the conflict by giving each implementation a type of its own, called a **provider**.
Because the providers are different types, their implementations never overlap. Here is the
interface, defined as a CGP **component**, and the provider that fetches from S3:

```rust
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
        // Fetch the object from Amazon S3.
    }
}
```

The `#[cgp_component]` attribute keeps `CanFetchStorageObject` as the trait that callers use, and it
generates `StorageObjectFetcher`, the trait that each provider implements. The `#[implicit]`
arguments read the client and the bucket from the fields of whichever type uses the provider, so a
caller passes only the object's ID.

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

Code that calls `fetch_storage_object` stays the same in all three contexts. Rust's coherence rules
still apply to everything the macros generate. CGP only arranges the implementations so that they no
longer conflict.
[Bypassing coherence](/docs/concepts/coherence) explains the rules, and
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) shows how a call reaches
the chosen provider.

### Zero-Cost Abstraction

Choosing a provider adds no runtime overhead because the compiler resolves the wiring into a direct
call. For `App`, the wiring above amounts to this implementation, which you could also write by
hand:

```rust
impl CanFetchStorageObject for App {
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        <FetchS3Object as StorageObjectFetcher<App>>::fetch_storage_object(self, object_id)
    }
}
```

Here, `StorageObjectFetcher<App>` is the provider trait with `App` as its context parameter.
That parameter lets one provider serve many contexts. The call names the provider directly,
without a runtime registry, reflection, or a virtual method table, and the provider itself is never
instantiated: it exists only as a type.

The work inside the provider, such as the request to S3, still has its usual runtime cost.
Generic code can also increase compile time and code size. To inspect the generated code, use
[`cargo cgp expand`](/docs/tooling/cargo-cgp/expand). For a comparison with `dyn Trait`, see
[Dynamic dispatch](/docs/comparisons/dynamic-dispatch).

### Type-Safe Wiring

A provider needs more than a place in the wiring table: its context must supply the fields and
traits it uses. The compiler checks those requirements when you use the component or check its
wiring explicitly. For example, `FetchS3Object` needs a `bucket_id` field. This context lacks that
field, so `check_components!` produces a compile error:

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

[cargo-cgp](/docs/tooling/cargo-cgp), CGP's error toolchain, reports the missing field first, followed by
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

Because wiring alone does not check those requirements, check each context where you assemble it.
For a simple context,
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) wires each
entry and checks it in the same step. Even then, the check establishes that the components fit
together, not that the application is correct.

Without cargo-cgp, this error takes more effort to read: the raw compiler message is longer and
represents the field's name as a nested type. `cargo cgp check` leads with the root
cause for the classes it recognizes, and the tool does not yet reshape every class.
[Check traits](/docs/concepts/check-traits) explains the checks, the
[checking tutorial](/docs/tutorials/area-calculation/checking) walks through a mis-wiring, and
[Dependency injection](/docs/comparisons/dependency-injection) compares wiring with a container.

### Abstract Over Every Dependency

So far the component has returned `anyhow::Result`, which ties every implementation to one error
type. To let each context choose its own error type, the component can declare that dependency
without naming a concrete type. Here the same component uses the context's
error type instead of `anyhow::Error`, and a provider declares the conversion it needs with
`#[uses]`, without adding it to the interface:

```rust
#[cgp_component(StorageObjectFetcher)]
#[use_type(HasErrorType.Error)]
pub trait CanFetchStorageObject {
    fn fetch_storage_object(&self, object_id: &str) -> Result<Vec<u8>, Error>;
}

#[cgp_impl(new FetchLocalObject)]
#[uses(CanRaiseError<std::io::Error>)]
#[use_type(HasErrorType.Error)]
impl StorageObjectFetcher {
    fn fetch_storage_object(
        &self,
        #[implicit] local_dir: &str,
        object_id: &str,
    ) -> Result<Vec<u8>, Error> {
        std::fs::read(format!("{local_dir}/{object_id}")).map_err(Self::raise_error)
    }
}
```

This definition of `CanFetchStorageObject` replaces the earlier one. Its `#[use_type]` attribute
lets the trait and the provider write `Error` for whatever error type the context chooses. The
provider's `#[uses]` attribute requires a conversion from a file error into that type. The call to
`Self::raise_error` performs that conversion.

The S3 and Google Cloud providers change in the same way: each returns `Error` and declares, with
`#[uses]`, the conversion it needs from its client's error. A context then chooses its error type in
a single wiring entry, such as `ErrorTypeProviderComponent: UseType<anyhow::Error>`, next to the
providers that convert each source error into it. Another context can choose an error enum of its
own without touching this code, as long as it supplies those conversions as well.

The same arrangement works for other dependencies, such as an async runtime, and it lets a core
library stay `no_std` as long as the providers chosen for it are `no_std` too. A provider that reads
files, like this one, still needs `std`.
[Modular error handling](/docs/concepts/modular-error-handling) explains how a context chooses its
error type, and [Impl-side dependencies](/docs/concepts/impl-side-dependencies) explains how a
provider keeps its needs out of the interface.

### Still Ordinary Rust

CGP is a library on stable Rust, and you can use it without defining providers or wiring at all.
Its simplest form is a function whose `#[implicit]` arguments come from the context's fields:

```rust
#[cgp_fn]
pub fn get_user(
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

Once you are comfortable choosing implementations, you can use the same mechanisms to share types,
work over the shape of data, compose computations and providers, and reuse wiring. These features
are optional, and the linked pages explain each one with examples:

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
- **Composable handlers:** components for synchronous or async computations that may be fallible
  or infallible. Their providers form pipelines that a context selects in its wiring.
  [Handlers](/docs/concepts/handlers) covers the family.
- **Namespaces:** a wiring table that several contexts share, so each context lists only the entries
  where it differs. [Namespaces](/docs/concepts/namespaces) explains them, along with the lighter
  `open` statement for choosing a provider per type.
- **Higher-order providers:** a provider that takes another provider as a parameter, such as a
  wrapper that scales whatever area its inner provider computes.
  [Higher-order providers](/docs/concepts/higher-order-providers) shows how they compose.

## The problems it solves

CGP helps most when a program already needs several implementations or repeats dependencies
through its signatures. If those problems sound familiar, these concept pages explain how CGP
addresses them:

| Problem | Explained in |
| --- | --- |
| Rust rejects two implementations of a trait for the same type | [Bypassing coherence](/docs/concepts/coherence) |
| A trait for a type you do not own needs a newtype | [Bypassing coherence](/docs/concepts/coherence) |
| Tests need a different implementation from production | [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) |
| Feature flags already select between backends | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| Changing the error type or runtime touches every layer | [Modular error handling](/docs/concepts/modular-error-handling) |
| A trait has grown until no implementation needs all of it | [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) |
| An implementation's dependencies leak into the public interface | [Impl-side dependencies](/docs/concepts/impl-side-dependencies) |
| Generic parameters and their bounds repeat through every layer | [Abstract types](/docs/concepts/abstract-types) |
| Generic code needs to work over any struct or enum | [Extensible records](/docs/concepts/extensible-records) |
| A wiring error buries its cause in generated types | [Check traits](/docs/concepts/check-traits) |

## When a plain trait is enough

For a trait with only one implementation, a plain trait is the right tool. CGP adds declarations
and wiring, and its macros and type checking add compile-time work. Its raw compiler errors can
also be long, as [*Type-Safe Wiring*](#type-safe-wiring) shows.

Learning the provider model and wiring patterns takes time too. If you work with an assistant,
[CGP's agent skill](/docs/ai/skills/) helps it with the vocabulary, diagnostics, and wiring,
although you still need to review what it writes. The [front page's costs](/#costs) describe the
project's maturity, and [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) explains when
a plain trait is enough and when providers start to help.

## Where to go next

To try these patterns in your own code or learn how they work, continue with these pages:

- **Run code:** the [Quickstart](/docs/quickstart), then the
  [Hello World tutorial](/docs/tutorials/hello), or build this page's storage example step by step
  in [Swap a Storage Backend per Application](/docs/tutorials/storage-backend).
- **Understand the mechanism:** the [Concepts](/docs/concepts), starting with
  [Bypassing coherence](/docs/concepts/coherence).
- **Compare with what you know:** the [Comparisons](/docs/comparisons).
- **Look up a construct:** the [Reference](/docs/reference).

---

*An AI agent wrote this page using the CGP knowledge base, and its examples were verified by
compiling them. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
