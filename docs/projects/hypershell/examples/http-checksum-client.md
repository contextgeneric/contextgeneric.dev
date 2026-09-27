---
sidebar_label: 'http_checksum_client'
sidebar_position: 4
description: 'A native Hypershell HTTP request feeding external commands, with input dispatch converting one kind of stream into another.'
---

# Replace a command with a native HTTP request

This program computes the same digest as [`http_checksum_cli`](./http-checksum-cli.md), with the
`curl` command replaced by a native HTTP request. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The page shows how
a native stage and an external command share one stream, even though they produce different kinds of
stream, and where the HTTP client comes from.

:::tip

### New to CGP?

This page builds on [`http_checksum_cli`](./http-checksum-cli.md). For CGP itself, the [Hello World
tutorial](/docs/tutorials/hello) introduces contexts and wiring. The dispatch this page describes is
the `open` statement of [`delegate_components!`](/docs/reference/macros/delegate_components), and
the page explains what it does here without assuming that reference.

:::

## The problem

The task is the same digest with `curl` replaced by an HTTP request made from Rust, whose response
streams into the `sha256sum` and `cut` commands. The difficulty is at the join: the HTTP library
produces its body as one kind of stream, and a command's input is fed from another kind, so the two
do not connect directly.

### Without CGP

In plain Rust, the join is written by hand: the response body is adapted into the reader type the
command's input needs, with a compatibility wrapper, and the adapting code is tied to the two stages
it sits between. Swap the HTTP stage for a file, or the command for another native stage, and that
code changes too. This page has each stage choose how to convert its input by the input's type, so
any stage can follow any other that produces something it can read.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example http_checksum_client
```

It needs the network and the `sha256sum` and `cut` commands, and prints the same digest of the
Nixpkgs manual page as `http_checksum_cli`.

## A native first stage

Only the first stage changes, from
[`http_checksum_client.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/http_checksum_client.rs):

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<
        GetMethod,
        FieldArg<"url">,
        WithHeaders[ ],
    >
    |   StreamingExec<
            StaticArg<"sha256sum">,
            WithStaticArgs [],
        >
    |   StreamingExec<
            StaticArg<"cut">,
            WithStaticArgs [
                "-d",
                " ",
                "-f",
                "1",
            ],
        >
    | StreamToStdout
};
```

`StreamingHttpRequest` sends a request with the `reqwest` crate and produces the response body as a
stream. Its parameters are the method, the URL, and a list of headers, here empty. The URL is a
`FieldArg`, read from the context, the type the program runs on, as before.

## The client lives in a field

The context gains a second field:

```rust
#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub url: String,
}

delegate_components! {
    MyApp {
        namespace HypershellNamespace;
    }
}
```

The HTTP stage needs a `reqwest::Client`, and it gets one from the context rather than creating its
own. `HypershellNamespace` wires the client lookup to a field by name:

```rust
@hypershell.reqwest.ReqwestClientGetterComponent:
    UseField<Symbol!("http_client")>,
```

[`UseField`](/docs/reference/providers/use_field) is the CGP provider that implements a getter by
reading a named field. So any context that joins the namespace and has a `Client` field named
exactly `http_client` can run HTTP programs, and a context without one fails to compile with a root
cause naming the missing field. `main` builds the context with `Client::new()`.

## The stream kinds are converted by dispatch

The request produces a different kind of stream from the command stages. `reqwest` returns a reader
from the `futures` crate, which Hypershell wraps as `FuturesAsyncReadStream`, while a command reads
from a Tokio reader. The program never converts one into the other, and it does not need to.

The conversion happens in the input dispatcher that `StreamingExec` starts with,
`HandleToTokioAsyncRead`. It is a small table from the Tokio backend crate, with one entry per input
type it accepts:

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

Each key is a path with two segments after `@HandlerComponent`: the syntax, then the input. The
`open` statement makes a lookup for `CanHandle<Code, Input>` follow that path, so an entry can
select a provider by the input type alone. The first segment of every entry here is the generic
`<Code> Code`, which matches any syntax. The second picks the conversion: a futures reader is
converted, a Tokio reader passes through unchanged with `ReturnInput`, and bytes are wrapped as a
reader.

This is why Hypershell wraps each stream kind in its own type. Entries keyed on
`FuturesAsyncReadStream<S>` and `TokioAsyncReadStream<S>` never overlap, where a single entry for
"anything that is a reader" would overlap every other entry and conflict. An input type the table
does not list fails to compile, with a root cause naming the missing entry.

## The pattern

This program shows **a stage choosing its implementation by the type of its input**. The same stage
works after a command, after an HTTP request, or at the start of a program, because a small
converter in front of it picks the right conversion for whatever arrives. The choice is ordinary CGP
wiring, made by the compiler, and it is why stages written by different people chain without the
program naming the conversions. The wiring form it uses is the `open` statement of
[`delegate_components!`](/docs/reference/macros/delegate_components).

The cost is that each stage accepts a fixed list of input types. A new kind of stream works only
once the converters have an entry for it, which is a change to the language, not to the program.

## Where to go next

- [`http_checksum_native`](./http-checksum-native.md): the next example, with the two commands
  replaced by native stages from an extension.
- [Streams and input dispatch](../architecture/streams-and-input-dispatch.md): every dispatcher, and
  what each stage accepts.
- [`UseField`](/docs/reference/providers/use_field): the provider behind the client field.
- [Handlers](/docs/concepts/handlers): the computation interface whose second parameter, the input,
  the dispatch keys on.
- [Hypershell: a type-level DSL for shell-scripting in
  Rust](/blog/hypershell-release#native-http-request): the post that announced Hypershell, which
  introduced this program. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
