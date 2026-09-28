---
title: 'ComputerRef — compute from a borrowed input'
description: 'The handler-family component for a synchronous computation that cannot fail and only reads its input, which it takes as &Input.'
sidebar_label: 'ComputerRef'
sidebar_position: 6
---

# `ComputerRef`

The by-reference member of the handler family: a [`Computer`](./computer.md) that borrows its input.

## Overview

`ComputerRef` is a [`Computer`](./computer.md) that takes its input by reference instead of by
value. A synchronous, infallible computation that only *reads* its argument fits `ComputerRef`,
whose method receives `&Input` where `CanCompute` receives `Input`, so the caller keeps ownership.
Everything else is the same: it turns the input into an `Output` under a phantom `Code` tag, against
a [**context**](/docs/reference/glossary#context) (the type the implementation runs against), and
returns the `Output` directly with no failure path.

It differs from `Computer` on the input axis alone. Like `Computer` it never names an error type, so
it does not have [`HasErrorType`](../has_error_type.md) as a
[supertrait](/docs/reference/glossary#supertrait). See the [handler family overview](./index.md) for
how the members relate.

## Definition

`CanComputeRef` is defined as:

```rust
#[cgp_component(ComputerRef)]
#[prefix(@cgp.extra.handler in DefaultNamespace)]
#[derive_delegate(UseDelegate<Code>)]
#[derive_delegate(UseInputDelegate<Input>)]
pub trait CanComputeRef<Code, Input> {
    type Output;

    fn compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
}
```

Its attributes:

- [`#[cgp_component]`](../../macros/cgp_component.md) — turns the trait into a component: its argument names the provider trait `ComputerRef` that implementations target and the wiring key `ComputerRefComponent`, while `CanComputeRef` stays the consumer trait callers use.
- [`#[prefix]`](../../attributes/prefix.md) — registers the component in `DefaultNamespace` under the path `@cgp.extra.handler`, so a context that joins that namespace binds its provider at `@cgp.extra.handler.ComputerRefComponent` rather than at the bare key.
- [`#[derive_delegate]`](../../attributes/derive_delegate.md) — generates the legacy `UseDelegate` and `UseInputDelegate` providers, which dispatch on the `Code` tag and on the `Input` type through an inner table; the `open` statement replaces both, dispatching on either parameter or on the pair.

## Usage

`ComputerRefComponent` is in the prelude. The provider trait `ComputerRef` and the consumer trait
`CanComputeRef` are not, and are imported from `cgp::extra::handler`. The method borrows the input:

```rust
fn compute_ref(&self, _code: PhantomData<Code>, input: &Input) -> Self::Output;
```

A context gains the operation by wiring `ComputerRefComponent` to a provider, and the component
dispatches on both the `Code` tag and the `Input` type. The
[`PromoteRef`](../../providers/handler/promote_ref.md) combinator, imported from
`cgp::extra::handler`, bridges the owned and borrowed forms in both directions, and the two
directions ask different things of the inner provider:

- **`PromoteRef<P>` as a `Computer`** takes an owned input that dereferences to `Target`, such as a
  `Box<String>` for `Target = String`, and calls the `ComputerRef` provider `P` on `input.deref()`.
  Any `ComputerRef` provider serves this direction.
- **`PromoteRef<P>` as a `ComputerRef`** passes the borrow `&Input` through as the input of the
  `Computer` provider `P`, so `P` must implement `Computer` for `&'a Input` at every lifetime `'a`.
  A computer written for an owned `u64` does not, so it cannot answer `compute_ref`.

Its async counterpart is [`AsyncComputerRef`](./async_computer_ref.md).

## Examples

A provider that measures a borrowed string, wired for borrowed input and, through `PromoteRef`, for
an owned input that dereferences to it:

```rust
use core::marker::PhantomData;
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, CanComputeRef, ComputerRef, PromoteRef};

#[cgp_new_provider]
impl<Context, Code> ComputerRef<Context, Code, String> for StringLength {
    type Output = usize;

    fn compute_ref(_context: &Context, _code: PhantomData<Code>, input: &String) -> usize {
        input.len()
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerRefComponent: StringLength,
        ComputerComponent: PromoteRef<StringLength>,
    }
}

check_components! {
    App {
        ComputerRefComponent: ((), String),
        ComputerComponent: ((), Box<String>),
    }
}

pub fn demo() {
    let name = "hello".to_owned();

    assert_eq!(App.compute_ref(PhantomData::<()>, &name), 5);
    assert_eq!(App.compute(PhantomData::<()>, Box::new(name)), 5);
}
```

`compute_ref` reads `name` without taking it, so the caller can still move it into the `Box` on the
next line. The `compute` call hands over the owned `Box<String>`, and `PromoteRef` dereferences it
to the `String` that `StringLength` reads. `App` is an [environmental
context](/docs/reference/glossary#environmental-context), and the component is
**[parameter-targeted](/docs/reference/glossary#parameter-targeted-component)**: the computation
acts on the `Input`, while `App` decides the provider.

## When to use it

**Reach for `ComputerRef` when a synchronous, infallible computation only needs to read its input.**
It avoids handing ownership to a computation that does not consume the value, which matters when the
caller reuses the input afterwards.

Reach for [`Computer`](./computer.md) instead when the computation takes the input by value, which
is the more common case, and for [`AsyncComputerRef`](./async_computer_ref.md) when a borrowed-input
computation must also await. If the computation can fail, use
[`TryComputerRef`](./try_computer_ref.md).

## Common Mistakes

**An owned-input `Computer` cannot answer `compute_ref`.** Wiring `ComputerRefComponent` to
`PromoteRef<Double>`, for a `Double` that implements `Computer` for `u64`, fails at the check,
because the promotion asks `Double` for `u64` borrowed at every lifetime. The compiler's note names
that requirement:

```text
   = note: required for `Double` to implement `for<'a> cgp::prelude::Computer<App, (), &'a u64>`
```

The same applies to the `ComputerRefComponent` entry of the `PromoteComputer` bundle and to a
[`#[cgp_computer]`](../../macros/cgp_computer.md) function over an owned parameter. Write the
provider as a `ComputerRef`, or give the function a reference parameter such as `value: &u64`.

## Related constructs

- [`Computer`](./computer.md) — the owned-input counterpart.
- [`AsyncComputerRef`](./async_computer_ref.md) — the async version of this by-reference variant.
- [`TryComputerRef`](./try_computer_ref.md) — the fallible by-reference computer.
- [Handler combinators](../../providers/handler/index.md) — the `PromoteRef` combinator bridging
  owned and borrowed inputs.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its sync, async, fallible, and
  input-passing axes.

## Source

- `Computer` and `ComputerRef`:
  [`computer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/computer.rs)
- `PromoteRef`:
  [`promote_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_ref.rs)
- Re-exported through `cgp::extra::handler`.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
