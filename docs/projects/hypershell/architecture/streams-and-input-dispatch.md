---
sidebar_label: 'Streams and input dispatch'
sidebar_position: 3
description: 'How Hypershell stages of different kinds fit together: bytes and three kinds of stream, stages that adapt to their input, and what each accepts.'
---

# Streams and input dispatch

In a shell, every program in a pipeline reads bytes and writes bytes, so any two can be connected.
In Hypershell, stages pass typed Rust values, so how do stages of different kinds fit together?
[Hypershell](../index.md) is a small shell-scripting language built with [CGP](/docs/). This page
explains what stages pass to each other, how a stage adapts to the kind of input it receives, and
which stages accept which inputs.

## What stages pass to each other

A stage passes either all of its output at once, as bytes, or a stream that the next stage reads as
it arrives. The simple stages, such as `SimpleExec` and `SimpleHttpRequest`, collect their whole
output and pass a `Vec<u8>`. The streaming stages, such as `StreamingExec` and
`StreamingHttpRequest`, pass a stream as soon as they start, which is what lets a pipeline run all
its stages at once.

Streams come in more than one kind, because Rust's async libraries define more than one reader
trait. A process started with Tokio produces a Tokio reader, while `reqwest` produces a reader from
the `futures` crate. Hypershell wraps each kind in its own type, so the next stage can tell which
one it received:

| Wrapper type | What it holds | Produced by |
|---|---|---|
| `TokioAsyncReadStream` | a Tokio reader | `StreamingExec`, `ReadFile`, `BytesToStream` |
| `FuturesAsyncReadStream` | a `futures` reader | `StreamingHttpRequest`, `WebSocket` |
| `FuturesStream` | a `futures` stream of chunks | the adapter in front of `Checksum` |

## A stage adapts to the type it receives

A stage that runs a command needs a Tokio reader to feed the command's input. To accept bytes or a
`futures` reader as well, it starts with a small converter, called an **input dispatcher**, that
looks at the type it receives and converts it. This is `HandleToTokioAsyncRead`, the dispatcher in
front of the Tokio stages:

```rust
delegate_components! {
    new HandleToTokioAsyncRead {
        open HandlerComponent;

        @HandlerComponent
            .<Code> Code
            .<S> FuturesAsyncReadStream<S>:
                FuturesToTokioAsyncRead,
        @HandlerComponent
            .<Code> Code
            .<S> TokioAsyncReadStream<S>:
                ReturnInput,
        @HandlerComponent
            .<Code> Code.[
                Vec<u8>,
                String,
            ]:
                HandleBytesToTokioAsyncRead,
    }
}
```

It is an ordinary CGP wiring table, where each entry names a provider for one kind of input. After
`@HandlerComponent`, each key has two parts: the syntax, here the generic `<Code> Code` that matches
any syntax, and then the input type. So the table reads: a `futures` reader is converted, a Tokio
reader is passed through unchanged, and bytes or a `String` are wrapped as a reader. Any other input
type has no entry, and the program fails to compile. The `open` statement at the top is what lets a
table choose by these parameters; the reference for
[`delegate_components!`](/docs/reference/macros/delegate_components) describes it.

This is also why the stream kinds each have a wrapper type. The table can only tell two kinds of
reader apart if they are different types; an entry for "any reader" would match every input and
conflict with the others.

## Stages are built from small steps

A streaming stage is usually three providers run in order. Under Hypershell's wiring,
`StreamingExec` is:

```rust
@HandlerComponent.<Path, Args> StreamingExec<Path, Args>:
    PipeHandlers<Product![
        HandleToTokioAsyncRead,
        HandleStreamingExec,
        WrapTokioAsyncRead,
    ]>,
```

The first step converts the input, the second runs the command, and the third wraps the command's
output as a `TokioAsyncReadStream`, so the *next* stage can tell what it received.
[`PipeHandlers`](/docs/reference/providers/handler/pipe_handlers) is CGP's way of running providers
in order, each one's output feeding the next. `WriteFile`, `StreamToStdout`, and `WebSocket` are
built the same way.

The HTTP request stage also chooses by input type. Given bytes or a `String`, it sends them as an
ordinary request body; given a stream, it streams the body as it reads it.

## Which stages accept which inputs

This table is the practical summary. It lists what each stage accepts and what it passes on. Here
*bytes* means a `Vec<u8>` or a `String`, and *either reader* means either reader type above. The
stages that accept only bytes, such as `SimpleExec`, take anything else that can be read as bytes
too:

| Stage | Accepts | Passes on |
|---|---|---|
| `SimpleExec` | bytes | `Vec<u8>` |
| `StreamingExec` | bytes or either reader | `TokioAsyncReadStream` |
| `SimpleHttpRequest` | a request body, such as bytes | `Vec<u8>` |
| `StreamingHttpRequest` | bytes or either reader | `FuturesAsyncReadStream` |
| `ReadFile` | `()`, since it has no input | `TokioAsyncReadStream` |
| `WriteFile` | bytes or either reader | `()` |
| `StreamToStdout` | bytes or either reader | `()` |
| `StreamToBytes`, `StreamToString` | a Tokio reader only | `Vec<u8>`, `String` |
| `BytesToStream` | bytes | `TokioAsyncReadStream` |
| `BytesToString` | bytes | `String` |
| `EncodeJson` | any value that implements `Serialize` | `Vec<u8>` |
| `DecodeJson<T>` | bytes | `T` |
| `Checksum<Hasher>` | bytes or either reader | the digest, as bytes |
| `BytesToHex` | bytes | `String` |
| `WebSocket` | a `Vec<u8>` or either reader | `FuturesAsyncReadStream` |

Two pairings come up often enough to remember:

- **A streaming stage cannot feed a simple one directly**, since `SimpleExec` takes bytes and a
  streaming stage passes a stream. Put `StreamToBytes` between them.
- **`StreamToBytes` and `StreamToString` accept only a Tokio reader**, so they cannot follow an HTTP
  or WebSocket stage directly. Put `ToTokioAsyncRead`, from `hypershell_tokio_components::dsl`, in
  front of them.

## How a streaming stage runs

A streaming command stage starts its process and passes on the process's output straight away, while
a background task feeds the process its input. So consecutive streaming stages run at the same time,
as they do in a shell. A failed command is reported when its output stream ends: the stage reading
the stream fails the program, with the command's error output in the message. These stages start
background tasks on Tokio, so a program that uses them must run inside a Tokio runtime, as every
example does with `#[tokio::main]`.

## What it costs

Typed stages catch mismatches that a shell would only reveal while running, and sometimes never. The
price is that a Hypershell pipeline is less forgiving: two stages connect only if the second accepts
what the first produces, and when they do not, the program has to name an adapter such as
`StreamToBytes`. A shell never asks for one.

The set of inputs each stage accepts is also fixed by its wiring. A stage accepts a new kind of
stream only once its dispatcher gains an entry for it, which is a change to the language rather than
to the program.

## Where to go next

- [`http_checksum_client`](../examples/http-checksum-client.md): an HTTP stream converted for a
  command, in a running program.
- [Crate layout](./crate-layout.md): which crate each stage lives in.
- [Handlers](/docs/concepts/handlers): the CGP computation interface each stage implements, and the
  combinators that chain them.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
