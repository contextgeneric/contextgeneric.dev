---
title: 'DeserializeDefault — read a null value as the default in cgp-serde'
sidebar_label: 'DeserializeDefault'
sidebar_position: 16
description: 'The cgp-serde higher-order provider that reads a null input as the type''s Default and passes any other input to an inner provider named in the wiring.'
---

# `DeserializeDefault`

Read a null input as the type's `Default`, and pass any other input to an inner provider.

## Overview

`DeserializeDefault<Provider>` makes a value optional in the input without making it an `Option`. It
is a [higher-order provider](/docs/concepts/higher-order-providers): it takes another provider as a
type parameter and calls it directly, for the same type, whenever the input is not null, instead of
asking the **context**, the type whose wiring holds an application's choices, which provider to use.

## Definition

```rust
#[cgp_impl(new DeserializeDefault<Provider>)]
#[use_provider(Provider: ValueDeserializer<'a, Value>)]
impl<'a, Value, Provider> ValueDeserializer<'a, Value>
where
    Value: Default,
{ ... }
```

[`#[use_provider]`](/docs/reference/attributes/use_provider) states that the inner `Provider` must
read `Value` for the same context.

## Usage

Import it from `cgp_serde::providers`, and name the inner provider in the wiring:

```rust
@ValueDeserializerComponent.u64: DeserializeDefault<UseSerde>,
```

## Behavior

The provider asks the format for an optional value. A null input produces `Value::default()`, so the
entry above reads `null` as `0`, and any other input goes to the inner provider, so `7` reads as
`7`.

The inner provider is named in the wiring rather than looked up through the context, because asking
the context to read the same `Value` would lead back to `DeserializeDefault` itself. The parameter
has no default, so the inner provider is always named.

The provider handles a null value, not an absent one. A struct field wired to it still has to appear
in the input for [`DeserializeRecordFields`](./deserialize_record_fields.md), which reports a
missing field before any field's provider runs.

## Context dependencies

None through the context. The inner provider must read `Value` for the same context, and brings its
own requirements.

## Pairing

It reads only. A default value is written like any other.

## When to use it

**Reach for `DeserializeDefault` when the input may carry `null` for a value that has a sensible
default.** It does not cover a field missing from the input altogether, since
`DeserializeRecordFields` requires every field of a struct to be present, including a field whose
type is an `Option`.

## Related constructs

- [`UseSerde`](./use_serde.md) is the usual inner provider.

## The ideas behind it

- [Higher-order providers](/docs/concepts/higher-order-providers): a provider parameterized by
  another provider.
- [Writing a provider](../../guides/writing-a-provider.md#hand-nested-values-back-to-the-context):
  why a provider for a type never asks the context for that same type.

## Source

- [`crates/cgp-serde/src/providers/default.rs`](https://github.com/contextgeneric/cgp-serde/blob/main/crates/cgp-serde/src/providers/default.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
