---
title: 'ReturnInput — the identity handler'
description: 'The handler that returns its input unchanged, the neutral stage of a pipeline, as a Computer, TryComputer, AsyncComputer, or Handler.'
sidebar_label: 'ReturnInput'
sidebar_position: 3
---

# `ReturnInput`

The identity handler: return the input unchanged as the output.

## Overview

`ReturnInput` ignores the [**context**](/docs/reference/glossary#context), the type the
implementation runs against, and the `Code` tag, and returns its input as its output. It is the
neutral element of handler composition: putting it before or after any other handler leaves that
handler's behavior unchanged. It fills a handler slot where no transformation is wanted and stands
in as a placeholder stage. Like every CGP provider, it carries no runtime value; it is a plain unit
struct.

## Usage

Import it from `cgp::extra::handler`; it is not in the prelude. It takes no type parameter:

```rust
use cgp::extra::handler::ReturnInput;

delegate_components! {
    App {
        ComputerComponent: ReturnInput,
    }
}
```

Wired this way, computing over any input returns that input unchanged.

## Examples

`ReturnInput` as the middle stage of a pipeline, where it changes nothing:

```rust
use cgp::prelude::*;
use cgp::extra::handler::{CanCompute, PipeHandlers, ReturnInput};

#[cgp_computer]
pub fn add_one(value: u64) -> u64 {
    value + 1
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: PipeHandlers<Product![AddOne, ReturnInput, AddOne]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), u64),
    }
}

pub fn demo() {
    assert_eq!(App.compute(PhantomData::<()>, 5), 7); // 5 -> 6 -> 6 -> 7
}
```

The pipeline behaves as `PipeHandlers<Product![AddOne, AddOne]>`, since the `ReturnInput` stage
passes its input straight on.

## When to use it

**Reach for `ReturnInput` when a handler slot must be filled but no transformation is wanted**, such
as a stage that is a no-op for one context, or a pipeline slot that a generic wiring always expects.
Reach for a producer written with [`#[cgp_producer]`](../../macros/cgp_producer.md) instead when the
slot should yield a fixed value rather than echo its input.

## Under the hood

`ReturnInput` is a plain unit struct:

```rust
pub struct ReturnInput;
```

It implements `Computer`, `TryComputer`, `AsyncComputer`, and `Handler`, in each case setting
`Output = Input` and returning the input directly. The fallible variants wrap the input in `Ok`, so
they require the context to have an error type. It has no `…Ref` impl.

## Related constructs

- [`ComposeHandlers`](compose_handlers.md), [`PipeHandlers`](pipe_handlers.md) — the composition
  combinators `ReturnInput` is the neutral element of.
- [`Producer`](../../components/handler/producer.md) — for a stage that yields a value rather than
  echoing the input.
- [`Computer`](../../components/handler/computer.md),
  [`Handler`](../../components/handler/handler.md) — the family it implements.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its composition.

## Source

- [`providers/return_input.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/return_input.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
