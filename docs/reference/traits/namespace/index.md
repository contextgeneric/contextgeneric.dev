---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Namespaces and defaults

The lookup traits behind a namespace and a per-type default.

## Overview

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a context opts into
and then selectively overrides. Resolving one of those defaults means asking, for a given key, what the
namespace delegates to — and these three traits answer it, differing only in how many types take part
in the key. You name them where the syntax requires it: a `namespace` header or a `for … in` loop
inside [`delegate_components!`](../../macros/delegate_components.md), and the
[`#[default_impl]`](../../attributes/default_impl.md) attribute. The macros generate the impls.

- [`DefaultNamespace`](default_namespace.md) keys a default on the component alone, the common case that
  [`#[prefix(...)]`](../../macros/cgp_namespace.md) registers into. It is the one member of the group
  that is in the prelude.
- [`DefaultImpls1`](default_impls1.md) adds one further type, for a per-type default where the same
  component resolves differently for `String` than for `u64`.
- [`DefaultImpls2`](default_impls2.md) does the same under a pair of types. Nothing in the library emits
  it; it is a provided extension point.

One edge is worth carrying across the group: for the two `DefaultImpls` traits the instance type takes
the `Self` position and the component name is a leading parameter, which is the opposite of the
parameter names' suggestion. The [`DefaultImpls1`](default_impls1.md#the-one-thing-to-get-right) page
works it out.

## The ideas behind them

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
