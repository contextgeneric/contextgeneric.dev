---
sidebar_label: 'ReturnInput'
sidebar_position: 3
---

# `ReturnInput`

The identity handler: return the input unchanged as the output.

## Overview

`ReturnInput` ignores the **context**, the type a capability runs against, and the `Code` tag, and
returns its input as its output. It is the neutral element of handler composition: putting it before or
after any other handler leaves that handler's behavior unchanged. It fills a handler slot where no
transformation is wanted, serves as the base case of a conditionally-built pipeline, and stands in as a
placeholder stage. Like every CGP provider, it carries no runtime value; it is a plain unit struct.

## Usage

Import it from `cgp::extra::handler`. It takes no type parameter:

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

`ReturnInput` is most useful as a pass-through stage. A pipeline that conditionally transforms its input
can fall back to `ReturnInput` when there is nothing to do, so the slot is always filled:

```rust
use cgp::extra::handler::{PipeHandlers, ReturnInput};

// A pipeline whose middle stage is a no-op for this context.
delegate_components! {
    App {
        ComputerComponent:
            PipeHandlers<Product![ParseInput, ReturnInput, Format]>,
    }
}
```

The `ReturnInput` stage passes its input straight to `Format`, so the pipeline behaves as
`PipeHandlers<Product![ParseInput, Format]>`.

## When to use it

**Reach for `ReturnInput` when a handler slot must be filled but no transformation is wanted**, such as
the base case of a pipeline built up conditionally, or a stage that is a no-op for one context. Reach
for a producer written with [`#[cgp_producer]`](../../macros/cgp_producer.md) instead when the slot
should compute a fixed value rather than echo its input.

## Under the hood

`ReturnInput` is a plain unit struct:

```rust
pub struct ReturnInput;
```

It implements `Computer`, `TryComputer`, `AsyncComputer`, and `Handler`, in each case setting
`Output = Input` and returning the input directly. The fallible variants wrap the input in `Ok`, so they
require the context to have an error type.

## Related constructs

- [`ComposeHandlers`](compose_handlers.md), [`PipeHandlers`](pipe_handlers.md) — the composition
  combinators `ReturnInput` is the neutral element of.
- [`Producer`](../../components/handler/producer.md) — for a stage that computes a value rather than echoing
  the input.
- [`Computer`](../../components/handler/computer.md), [`Handler`](../../components/handler/handler.md) — the family it
  implements.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family and its composition.

## Source

- [`providers/return_input.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/return_input.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
