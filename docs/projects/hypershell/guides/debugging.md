---
sidebar_label: 'Debugging'
sidebar_position: 2
description: 'The common mistakes in a Hypershell program or its wiring, each with the code that triggers it, the root cause cargo cgp check reports, and the fix.'
---

# Debugging a program

A mistake in a Hypershell program or its wiring is a compile error, and the raw compiler output for
one spans every layer of wiring between the context, the type the program runs on, and the part that
failed. [Hypershell](../index.md) is a shell-scripting language whose programs are Rust types, built
with [CGP](/docs/). This guide shows the common mistakes, each with the code that triggers it, the
root cause [`cargo cgp check`](/docs/cargo-cgp/check) reports, and the fix. Each output below was
produced by running the check and is trimmed to its headline and root cause, with long types
abridged. `cargo cgp check` leads with the root cause for the classes it recognizes, and the tool
does not yet reshape every class.

## Check first, then read the root cause

**Put the program in a `check_components!` before reading any error, and read the result with
`cargo cgp check`.** A failure reported at the `handle` call often carries no root cause at all: the
missing-field mistake below, reported at its call site, produces only headlines about the pipeline's
combinators. Through a check, the same mistake names its cause. The check is keyed on the program
and its input:

```rust
use cgp::extra::handler::HandlerComponent;

check_components! {
    #[check_trait(CheckHypershellCli)]
    HypershellCli {
        HandlerComponent: (Program, Vec<u8>),
    }
}
```

Below the root cause, `cargo cgp check` prints the chain of lookups that led to it, one line per
step, which follows the walk on the
[assembly](../architecture/assembly.md#following-one-piece-of-syntax) page. Read it from the bottom:
the last line is the part that failed. Each step names the table it looked in, so the chain shows
whether the failure is in the context, in a bundle, or in an input dispatcher. The codes in brackets
are listed on the [error codes](/docs/cargo-cgp/error-codes) page.

## A context lacks a field

A `FieldArg` names a field the context does not have. Here `HypershellCli`, which has no fields,
runs a program that reads `name`:

```rust
pub type Program = hypershell! {
        SimpleExec<StaticArg<"echo">, WithArgs[StaticArg<"Hello,">, FieldArg<"name">]>
    |   StreamToStdout
};
```

```text
error[E0277]: [CGP-E002] the provider trait `Handler<Pipe<…>, Vec<u8>>` with context `HypershellCli` is not implemented for provider `ComposeHandlers<…>`
   = note: root cause: [CGP-E106] missing field `name` on `HypershellCli`
```

The chain runs through the pipeline, `SimpleExec`, the command's argument list, and the extractor
for `FieldArg<"name">`. **Fix:** run the program on a context with the field, deriving `HasField`,
as [`hello_name`](../examples/hello-name.md) does. The same error names `http_client` when an HTTP
program runs on a context without a client.

## A stage cannot accept the previous stage's output

The input type of a stage is the previous stage's output, and the error takes one of two forms,
depending on whether the stage has an input dispatcher.

**A stage without a dispatcher reports the unmet bound.** `SimpleExec` needs bytes, and a streaming
stage produces a stream:

```rust
pub type Program = hypershell! {
        StreamingExec<StaticArg<"echo">, WithStaticArgs["hello"]>
    |   SimpleExec<StaticArg<"wc">, WithStaticArgs["-c"]>
    |   StreamToStdout
};
```

```text
   = note: root cause: [CGP-E201] the trait bound `TokioAsyncReadStream<ChildOutputStream>: AsRef<[u8]>` is not satisfied
```

**Fix:** insert `StreamToBytes` between the two stages. With it, the program compiles and prints
`6`, the byte count of `hello` and its newline.

**A stage with a dispatcher reports the entry it lacks.** Removing `BytesToHex` from
[`http_checksum_native`](../examples/http-checksum-native.md) leaves the raw digest flowing into
`StreamToStdout`:

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<GetMethod, FieldArg<"url">, WithHeaders[ ]>
    | Checksum<Sha256>
    | StreamToStdout
};
```

```text
   = note: root cause: [CGP-E110] provider `HandleToTokioAsyncRead` does not contain any delegate entry for `@HandlerComponent.StreamToStdout.GenericArray<u8, …>`
```

The path names the syntax and then the input type, which is exactly what the dispatcher could not
match; see [streams and input
dispatch](../architecture/streams-and-input-dispatch.md#a-stage-adapts-to-the-type-it-receives).
**Fix:** convert the value into one of the types the dispatcher accepts, here with `BytesToHex`. The
same form appears for an input the caller passes: `StreamingExec` given a `&'static str` reports a
missing `@HandlerComponent.StreamingExec<…>.&str` entry, and the fix is to pass a `String` or a
`Vec<u8>`.

## A syntax has no route

A syntax that no route reaches fails at the missing route, even when a provider for it exists.
`Checksum` has a provider in the hash crate, but `HypershellNamespace` does not route it, so a
program using it fails on `HypershellCli`:

```rust
pub type Program = hypershell! {
        StreamingExec<StaticArg<"cat">, WithStaticArgs["Cargo.toml"]>
    |   Checksum<Sha256>
};
```

```text
error[E0277]: [CGP-E001] the consumer trait `CanHandle<Pipe<…>, Vec<u8>>` is not implemented for context `HypershellCli`
  = note: root cause: [CGP-E107] context `HypershellCli` does not contain any delegate entry for `@cgp.extra.handler.HandlerComponent.Checksum<…>`
```

`WebSocket` fails the same way on such a context. **Fix:** add the route, either on a namespace that
inherits `HypershellNamespace`, as [`http_checksum_native`](../examples/http-checksum-native.md)
joins, or as an entry beside the context's `namespace` statement, as
[`bluesky_websocket`](../examples/bluesky-websocket.md) adds.

## A raised error type has no route

A provider raises an error type the namespace does not know. This custom stage raises `TooLong` when
its input is longer than 16 bytes:

```rust
use cgp::extra::handler::Handler;
use hypershell::namespaces::HypershellNamespace;
use hypershell::prelude::*;

pub struct CheckLength;

#[derive(Debug)]
pub struct TooLong;

#[cgp_impl(new HandleCheckLength)]
#[uses(CanRaiseError<TooLong>)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<CheckLength, Input>
where
    Input: Send + AsRef<[u8]>,
{
    type Output = Input;

    async fn handle(&self, _code: PhantomData<CheckLength>, input: Input) -> Result<Input, Error> {
        if input.as_ref().len() > 16 {
            return Err(Self::raise_error(TooLong));
        }
        Ok(input)
    }
}

pub struct App;

delegate_components! {
    App {
        namespace HypershellNamespace;
        @cgp.extra.handler.HandlerComponent.CheckLength: HandleCheckLength,
    }
}
```

```text
error[E0277]: [CGP-E001] the consumer trait `CanHandle<CheckLength, Vec<u8>>` is not implemented for context `App`
   = note: root cause: [CGP-E107] context `App` does not contain any delegate entry for `@cgp.core.error.ErrorRaiserComponent.TooLong`
```

The provider requires the context to raise `TooLong` into its error type, and `HypershellNamespace`
routes only the error types the base language raises. **Fix:** route the new type to a way of
converting it, beside the `namespace` statement:

```rust
@cgp.core.error.ErrorRaiserComponent.TooLong: DebugAnyhowError,
```

`DebugAnyhowError`, from `cgp_error_anyhow`, formats any `Debug` value into the `anyhow::Error` the
namespace uses, and `RaiseAnyhowError` converts a type that implements `std::error::Error`. The path
names `ErrorRaiserComponent`, so it must be imported from `cgp::core::error`. The idea behind the
routes is CGP's [modular error handling](/docs/concepts/modular-error-handling).

## A routed provider does not implement `Handler`

A route that exists can still lead to a provider of the wrong kind, as when a syntax is routed
straight to a synchronous [`Computer`](/docs/reference/components/handler/computer) provider. Here
`App` routes a custom `Shout` syntax to `ShoutText`:

```rust
use cgp::extra::handler::Computer;

pub struct Shout;

#[cgp_impl(new ShoutText)]
impl<Code> Computer<Code, String> {
    type Output = String;

    fn compute(&self, _code: PhantomData<Code>, input: String) -> String {
        input.to_uppercase()
    }
}

delegate_components! {
    App {
        namespace HypershellNamespace;
        @cgp.extra.handler.HandlerComponent.Shout: ShoutText,
    }
}
```

```text
error[E0277]: [CGP-E002] the provider trait `Handler<Shout, String>` with context `App` is not implemented for provider `ShoutText`
   = note: root cause: [CGP-E111] the provider trait `Handler` is not implemented for `ShoutText`
```

**Fix:** lift the provider into a handler, as `Promote<PromoteAsync<ShoutText>>`, with both imported
from `cgp::extra::handler`. [`PromoteAsync`](/docs/reference/providers/handler/promote_async) makes
the synchronous computer an asynchronous one, and
[`Promote`](/docs/reference/providers/handler/promote) makes that a `Handler`. Hypershell wires its
own `ConvertTo` syntax the same way.

## Rebinding a syntax conflicts

An entry for a syntax the namespace already routes conflicts with the namespace:

```rust
delegate_components! {
    App {
        namespace HypershellNamespace;
        @cgp.extra.handler.HandlerComponent.<Path, Args> SimpleExec<Path, Args>: FakeSimpleExec,
    }
}
```

```text
error[E0119]: [CGP-E005] `App` cannot wire `@cgp.extra.handler.HandlerComponent.SimpleExec.*` that is already set through `HypershellNamespace`
```

A namespace that inherits `HypershellNamespace` and binds the same path fails in the same way.
**Fix:** there is no wiring-only fix, because a namespace's bindings cannot be overridden. Choose
the provider in the program instead, with `Use<FakeSimpleExec, SimpleExec<…>>`, which runs the named
provider for that one stage, or write a namespace of your own that routes every syntax itself. The
[namespaces](/docs/concepts/namespaces) page explains why a shared namespace's choices cannot be
overridden.

## The macro cannot find its names

A program written with `hypershell!` can fail with "cannot find macro `Product` in this scope" or
"cannot find type `Pipe` in this scope". The message comes from the macro rather than from the
wiring, and does not mention CGP. It means the prelude is not imported: the macro writes `Pipe`,
`Product!`, and `Symbol!` without qualifying them, so they must be in scope. **Fix:** import
`hypershell::prelude::*`.

## Where to go next

- [Checking your wiring](/docs/concepts/check-traits): why CGP wiring is checked lazily, and what a
  check forces.
- [Compile errors](/docs/reference/errors): the classes of CGP compile error, each with the program
  behind it.
- [`cargo cgp check`](/docs/cargo-cgp/check): installing and running the checker.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
