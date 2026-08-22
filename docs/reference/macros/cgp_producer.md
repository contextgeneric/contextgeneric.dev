---
sidebar_label: '#[cgp_producer]'
---

# `#[cgp_producer]`

Define a `Producer` provider from an input-free function.

## Overview

`#[cgp_producer]` is for the degenerate member of the handler family: a computation that takes no input and
yields a value from nothing. A constant, a default, a seed for a pipeline. It is the input-free sibling of
[`#[cgp_computer]`](./cgp_computer.md), and like that macro it lets you write a plain function and have the
provider synthesized for you:

```rust
#[cgp_producer]
fn magic_number() -> u64 {
    42
}
```

That produces the provider struct `MagicNumber` and an impl of [`Producer`](../components/producer.md), the
family member whose method takes only a **context** — the type the capability runs against — and a phantom
`Code` tag, with no input value at all.

The reason it is worth a macro of its own is what happens next. **A producer can stand in for any handler**,
because a handler that ignores its input is just a producer with an unused parameter. So the macro wires the
generated provider into every member of the family, and one function definition answers `produce`,
`compute`, `try_compute`, `compute_async`, `handle`, and their by-reference forms — every one of them
yielding the same value regardless of what it is handed.

## Using it

Apply the attribute to a free function. It takes an optional provider name:

```rust
#[cgp_producer]
fn magic_number() -> u64 {
    42
}

#[cgp_producer(TheAnswer)]
fn magic_number() -> u64 {
    42
}
```

Omitted, the provider struct takes the function name in PascalCase — `magic_number` becomes `MagicNumber`.
Given, the argument is used verbatim. The function's return type becomes the producer's output.

**The function is constrained tightly, to exactly what a producer can be.** All three restrictions are
enforced at expansion time with their own messages:

| Restriction | Why |
|---|---|
| No parameters, and no `self` | A producer takes no input and has no receiver. |
| Not `async` | The producer trait is synchronous. |
| No generic parameters | There is nothing to infer them from. |

That third restriction is the one most likely to bite, and it is the sharpest difference from
[`#[cgp_computer]`](./cgp_computer.md), which carries generics through happily. A producer has no input, so a
type parameter would be determined by nothing at all.

## Examples

One function, read back through every shape the family offers:

```rust
use cgp::prelude::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{Producer, Computer, TryComputer, Handler};

#[cgp_producer]
pub fn magic_number() -> u64 {
    42
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

// The single `magic_number` definition answers every shape, all yielding 42:
// MagicNumber::produce(&App, PhantomData::<()>)           == 42
// MagicNumber::compute(&App, PhantomData::<()>, &())      == 42
// MagicNumber::try_compute(&App, PhantomData::<()>, &())  == Ok(42)
// MagicNumber::handle(&App, PhantomData::<()>, &())       resolves to Ok(42)
```

The computer and handler forms accept an input argument and discard it, since the underlying producer takes
none. The error type wired into `App` is what lets the fallible forms build their `Result`; the produced
value is always `Ok`.

The typical use is as the first step of a pipeline, where a producer seeds the value the later steps
transform:

```rust
delegate_components! {
    App {
        ComputerComponent:
            PipeHandlers<Product![
                MagicNumber,
                Double,
            ]>,
    }
}
```

## When to reach for it, and when not

**Reach for `#[cgp_producer]` when a pipeline step needs no input.** That is the whole of its remit, and
within it there is nothing simpler.

- **Use [`#[cgp_computer]`](./cgp_computer.md) as soon as there is an input**, even a trivial one. It is the
  same macro with the restrictions lifted, and it handles generics, `async`, and `Result` returns.
- **Write the provider by hand with [`#[cgp_impl]`](./cgp_impl.md) when the value comes from the context.**
  This is the boundary that matters. A `#[cgp_producer]` function has no receiver, so it cannot read a field,
  name an abstract type, or call a capability — it can only return something it computes from nothing. A
  producer that draws on its context is an impl of `Producer` written with `#[cgp_impl]`, where `self` is the
  context and [`#[implicit]`](../attributes/implicit.md) works normally. **In practice that covers most
  producers**, which makes this macro narrower than it first looks: it is for constants and pure seeds.
- **Use [`ReturnInput`](../providers/handler_combinators.md) rather than a producer that ignores its input.**
  If the goal is to pass a value through a pipeline unchanged, that combinator says so directly.
- **Do not reach for the handler family at all for a plain constant.** A `const` or a function is clearer
  unless the value is genuinely being composed into a pipeline or dispatched on a `Code` tag.

## Under the hood

:::note

### Advanced

This section shows the three items the macro emits. You do not need them to use `#[cgp_producer]`, but the
delegation block is what makes one function answer eight components, and it differs from
[`#[cgp_computer]`](./cgp_computer.md)'s in a way worth noticing. `cargo cgp expand` prints the same thing for
your own code.

:::

The macro emits the function unchanged, a provider impl of [`Producer`](../components/producer.md), and a
[`delegate_components!`](./delegate_components.md) block wiring the whole family. From this input:

```rust
#[cgp_producer]
pub fn magic_number() -> u64 {
    42
}
```

it produces the base impl, whose method ignores both of its parameters and simply calls the function:

```rust
#[cgp_new_provider]
impl<__Context__, __Code__> Producer<__Context__, __Code__> for MagicNumber {
    type Output = u64;

    fn produce(_context: &__Context__, _code: PhantomData<__Code__>) -> Self::Output {
        magic_number()
    }
}
```

[`#[cgp_new_provider]`](./cgp_provider.md) declares `pub struct MagicNumber;` and derives the
[`IsProviderFor`](../traits/is_provider_for.md) impl, whose parameter tuple here holds just the code tag. The
context and code parameters carry the reserved names `__Context__` and `__Code__`.

Then the wiring, which reaches **all eight** other components:

```rust
delegate_components! {
    MagicNumber {
        [
            ComputerComponent,
            ComputerRefComponent,
            TryComputerComponent,
            TryComputerRefComponent,
            AsyncComputerComponent,
            AsyncComputerRefComponent,
            HandlerComponent,
            HandlerRefComponent,
        ]:
            PromoteProducer<Self>,
    }
}
```

Two things differ from `#[cgp_computer]`'s block. `ComputerComponent` **is** in the list, because a producer
does not implement it directly — a computer takes an input and the producer has none, so the promotion is
what discards it. And the operator is **`:`** rather than `->`, delegating each component straight to
[`PromoteProducer<Self>`](../providers/handler_combinators.md) rather than to that bundle's own entry for the
key. `PromoteProducer` then wires `ComputerComponent` to a promoter that drops the input and calls `produce`,
and derives the remaining members from there.

**Unlike `#[cgp_computer]`, the expansion has no variation.** Because the function cannot be async, cannot be
generic, and is not inspected for a `Result` return, there is exactly one base trait and one bundle for every
`#[cgp_producer]` — the output type is taken as written, whether or not it happens to be a `Result`.

<details>
<summary>Formal grammar</summary>

The attribute argument is a single optional provider name, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
CgpProducerArgs -> ProviderName?

ProviderName    -> IDENTIFIER
```

Omitted, the provider struct takes the function name converted to PascalCase; a given `IDENTIFIER` is used
verbatim. The annotated function is plain Rust, constrained to a producer's shape — no parameters, no
`async`, no generics — as described in [Using it](#using-it).

</details>

## Gotchas

**All three shape restrictions have their own message**, so the macro tells you which one you broke:

```text
error: Producer functions cannot have parameters
error: Producer functions cannot be async
error: Producer functions must have empty generic parameters
```

For the first two, [`#[cgp_computer]`](./cgp_computer.md) is the macro that accepts them. For the third there
is no macro alternative — write the provider by hand.

**The function cannot reach the context**, so a producer that needs a field or an abstract type is not
expressible this way at all. That rules out most real producers, which is worth knowing before reaching for
the macro: write an impl of `Producer` with [`#[cgp_impl]`](./cgp_impl.md) instead.

**A `Result` return is not interpreted**, and this is the sharpest trap on the page. Unlike
`#[cgp_computer]`, the macro does not inspect the output for fallibility, so a `#[cgp_producer]` returning
`Result<T, E>` produces a `Producer` whose `Output` *is* that `Result` — and the fallible members wrap it
again rather than treating it as failure:

```text
producer produce     = Err("nope")
producer try_compute = Ok(Err("nope"))
```

That `Ok(Err(..))` is a success carrying an error, so it short-circuits nothing and a pipeline downstream
keeps going. If the production can genuinely fail, write a `TryComputer` or `Handler` provider by hand
instead.

**The fallible forms still need an error type on the context.** `try_compute` and `handle` name the context's
abstract error, so a context lacking an
[`ErrorTypeProviderComponent`](../components/has_error_type.md) fails on those members while `produce` and
`compute` work. Note also that the wiring key is **not in the prelude** — it has to be imported from
`cgp::core::error`, and forgetting that reports the component as an unresolved type rather than as a missing
import.

## Related constructs

- [`#[cgp_computer]`](./cgp_computer.md) — the sibling for a computation that takes an input.
- [`Producer`](../components/producer.md) — the base component this implements.
- [Handler combinators](../providers/handler_combinators.md) — `PromoteProducer`, which this wires, plus
  `ReturnInput` and `PipeHandlers`.
- [`Computer`](../components/computer.md) and [`Handler`](../components/handler.md) — the members the
  promotion reaches.
- [`#[cgp_impl]`](./cgp_impl.md) — for a producer that needs its context, which is most of them.
- [`#[cgp_new_provider]`](./cgp_provider.md) — what the generated impl is emitted through.
- [`HasErrorType`](../components/has_error_type.md) — what the fallible forms require of a context.

The ideas behind it:

- [Handlers](/docs/concepts/handlers) — the computation family, and where the input-free member
  fits.

## Source

- Entry point: [`entrypoints/cgp_producer.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-extra-macro-lib/src/entrypoints/cgp_producer.rs)
- The `Producer` trait: [`components/produce.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/components/produce.rs)
- The `PromoteProducer` bundle: [`providers/promote_all.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-handler/src/providers/promote_all.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
