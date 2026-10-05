---
sidebar_label: 'bluesky_websocket'
sidebar_position: 11
description: 'A Hypershell program that adds a WebSocket stage and its error type to one context with two wiring entries, instead of joining an extension namespace.'
---

# Add a WebSocket stage to one context

This program reads the same Bluesky firehose as [`bluesky`](./bluesky.md), with `websocat` replaced
by a native WebSocket stage. It is an example from [Hypershell](../index.md), a shell-scripting
language whose programs are Rust types, built with [CGP](/docs/). The WebSocket stage is an
extension that the base language does not include, and this page shows how one context, the type a
program runs on, adds it with two wiring entries, where
[`http_checksum_native`](./http-checksum-native.md) joined an extended namespace.

:::tip

### New to CGP?

This page builds on [`http_checksum_native`](./http-checksum-native.md), which introduces extending
the language. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is the quickest
start. The page uses CGP's [modular error handling](/docs/concepts/modular-error-handling), in which
the context chooses the error type, and explains what it needs of it.

:::

## The problem

The task is the same firehose, read with a native WebSocket stage instead of the `websocat` command.
The WebSocket stage is an extension the base language does not include, and it can fail in ways the
base language's error handling does not know about, so the task is to add it, and its errors, to a
single program without changing the language for everyone else.

### Without CGP

In a language whose program is an enum interpreted by a `match`, a new stage is an edit to the
language's crate, and its errors are a new case in the language's error type, so the change reaches
every user. Without a language at all, the WebSocket client is a loop written by hand that reads
messages and writes them to the next command's input. This page adds the stage to one context with
two wiring entries, one for the stage and one for its error, and leaves the language and every other
context unchanged.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example bluesky_websocket
```

It needs the network and `grep`. It prints firehose events that contain `love` as they arrive, and
runs until it is stopped with Ctrl-C.

## The program

From
[`bluesky_websocket.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/bluesky_websocket.rs):

```rust
use cgp::core::error::ErrorRaiserComponent;
use cgp_error_anyhow::RaiseAnyhowError;
use hypershell::namespaces::HypershellNamespace;
use hypershell::prelude::*;
use hypershell_tokio_components::types::TokioAsyncReadStream;
use hypershell_tungstenite_components::providers::HypershellTungsteniteProvider;
use tokio::io::simplex;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

pub type Program = hypershell! {
        WebSocket<
            StaticArg<"wss://jetstream1.us-west.bsky.network/subscribe">,
            (),
        >
    |   StreamingExec<
            StaticArg<"grep">,
            WithArgs [ FieldArg<"keyword"> ],
        >
    |   StreamToStdout
};
```

`WebSocket` connects to the URL and produces the messages it receives as a stream. Its second
parameter is `()` in this program.

## Two entries beside the namespace

The context joins `HypershellNamespace` and adds two entries of its own:

```rust
#[derive(HasField)]
pub struct MyApp {
    pub keyword: String,
}

delegate_components! {
    MyApp {
        namespace HypershellNamespace;

        @cgp.core.error.ErrorRaiserComponent.TungsteniteError:
            RaiseAnyhowError,

        @cgp.extra.handler.HandlerComponent.<Url, Params> WebSocket<Url, Params>:
            HypershellTungsteniteProvider,
    }
}
```

The second entry routes the `WebSocket` syntax to `HypershellTungsteniteProvider`, the bundle of
providers that the `hypershell-tungstenite-components` crate publishes. It uses the same full path
the namespace uses for its own routes, starting from `@cgp.extra.handler`, the prefix under which
CGP's handler component is registered.

The first entry is needed because the WebSocket provider can fail with an error type the base
language has never seen. Every provider reports failure through the context's error type, which
`HypershellNamespace` sets to `anyhow::Error`, and it converts each kind of source error through a
route listed for that type. The namespace lists the errors the base language raises, so an extension
that raises a new one must add its route: here `tungstenite`'s error, converted by
`RaiseAnyhowError` from the `cgp-error-anyhow` crate, which accepts any type implementing
`std::error::Error`. The entry names `ErrorRaiserComponent` in its path, so that component is
imported even though the code never mentions it otherwise.

Entries like these work for syntax the namespace does not already route. A context cannot use the
same form to give `SimpleExec` a different provider, because the namespace already binds that path;
see the [limitations](../limitations.md#extensions-add-to-the-language-rather-than-replace-it).

## An input that stays open

`main` passes an input that never ends:

```rust
let (read, _write) = simplex(102400);
let input = TokioAsyncReadStream::from(read);

app.handle(PhantomData::<Program>, input).await?;
```

The WebSocket stage forwards its input to the socket and closes the connection when the input ends.
An empty `Vec<u8>` would therefore end the stream at once. The program instead passes the reading
half of an in-memory pipe whose writing half is kept alive and never written, so the connection
stays open.

## Try a change

Remove the error route, keeping the `WebSocket` entry. Checked with `check_components!` as on the
[`hello_name`](./hello-name.md#try-a-change) page, keyed on the program and the pipe's reader type,
[`cargo cgp check`](/docs/cargo-cgp/check) reports the missing route, shown here with the long types
abridged:

```text
error[E0277]: [CGP-E002] the provider trait `Handler<Pipe<…>, TokioAsyncReadStream<ReadHalf<SimplexStream>>>` with context `MyApp` is not implemented for provider `ComposeHandlers<…>`
   = note: root cause: [CGP-E107] context `MyApp` does not contain any delegate entry for `@cgp.core.error.ErrorRaiserComponent.Error`
```

The path's last segment is the error type the provider raises, written by its own name. That type is
`tungstenite::Error`, which the program imports as `TungsteniteError`, so the `Error` in the message
is `tungstenite`'s error rather than the context's own. Adding the route back fixes it.
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class.

## The pattern

This program shows **extending one context rather than the whole language**. Entries next to the
`namespace` statement add syntax for that context alone, with no new type to define. A namespace
that inherits the standard one, as in [`http_checksum_native`](./http-checksum-native.md), does the
same for every context that joins it. The first suits a single application, and the second an
extension that several applications share.

It also shows how errors work in CGP: code reports its own kinds of error, and the context decides
how each one becomes the application's error type. See [modular error
handling](/docs/concepts/modular-error-handling).

The cost is bookkeeping. Every new kind of error an extension can produce needs its own entry, and a
context that forgets one finds out from a compile error, which names the missing error type as shown
above.

## Where to go next

- [Examples](./index.md): every example, in teaching order.
- [Writing a program](../guides/writing-a-program.md): the steps every example shares, as a guide.
- [Error handling in CGP](/docs/concepts/modular-error-handling): how a context chooses its error
  type and how source errors reach it.
- [Assembly](../architecture/assembly.md#a-context-joins-the-namespace): context entries and
  extension namespaces.
- [Hypershell: a type-level DSL for shell-scripting in
  Rust](/blog/hypershell-release#extending-hypershell): the post that announced Hypershell, whose
  last part covers extending the language. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
