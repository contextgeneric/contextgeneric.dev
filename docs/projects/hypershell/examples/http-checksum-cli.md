---
sidebar_label: 'http_checksum_cli'
sidebar_position: 3
description: 'The shell pipeline curl | sha256sum | cut written as three streaming Hypershell stages that run concurrently, with typed stages that must line up.'
---

# Pipe three commands together, streaming

This program is the shell pipeline `curl $url | sha256sum | cut -d ' ' -f 1`: it downloads a web
page and prints its SHA-256 digest. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The page shows how
streaming stages run concurrently, and why each stage's input type decides what may come before it.

:::tip

### New to CGP?

Read [`hello`](./hello.md) and [`hello_name`](./hello-name.md) first, which introduce programs as
types and contexts with fields. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is
the quickest start, and [Handlers](/docs/concepts/handlers) explains the family of computations each
stage belongs to.

:::

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example http_checksum_cli
```

It needs the network and the `curl`, `sha256sum`, and `cut` commands. It prints the SHA-256 digest
of the Nixpkgs manual page as a line of hexadecimal, the same digest the next two examples print by
other means.

## Three streaming stages

The program is three `StreamingExec` stages and a final `StreamToStdout`, from
[`http_checksum_cli.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/http_checksum_cli.rs):

```rust
pub type Program = hypershell! {
    StreamingExec<
        StaticArg<"curl">,
        WithArgs [
            FieldArg<"url">,
        ],
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

#[derive(HasField)]
pub struct MyApp {
    pub url: String,
}

delegate_components! {
    MyApp {
        namespace HypershellNamespace;
    }
}
```

`curl` reads its URL from the `url` field of the context, the type the program runs on, as
[`hello_name`](./hello-name.md) read its name. `WithStaticArgs []` is an empty argument list. `main`
builds `MyApp` with the manual's URL and calls `handle` with an empty `Vec<u8>` as the input.

## Each stage streams

`StreamingExec` differs from the `SimpleExec` the earlier examples used in when it returns.
`SimpleExec` waits for its command to finish and produces the whole output as bytes. `StreamingExec`
spawns its command and returns the command's standard output as a stream at once, while a background
task copies the stage's input into the command's standard input. So the three commands run at the
same time, each reading the previous one's output as it arrives, as they would in a shell.

That changes when a failure is reported. A streaming stage has already returned by the time its
command fails, so a non-zero exit is reported when its output stream ends, and the stage reading
that stream fails the program. The command's standard error becomes part of the message.

## The stages must line up

Every stage's output is the next stage's input, and the compiler checks that each pair agrees. A
`StreamingExec` stage produces a Tokio reader, wrapped in Hypershell's `TokioAsyncReadStream` type,
and the next `StreamingExec` accepts it. The first stage accepts the `Vec<u8>` that `main` passes,
and `StreamToStdout` at the end accepts the stream.

A stage accepts several input types because its wiring puts a small **input dispatcher** in front of
the provider that does the work. Under `HypershellNamespace`, `StreamingExec` is interpreted by
three providers run in sequence:

```rust
@HandlerComponent.<Path, Args> StreamingExec<Path, Args>:
    PipeHandlers<Product![
        HandleToTokioAsyncRead,
        HandleStreamingExec,
        WrapTokioAsyncRead,
    ]>,
```

The first converts whatever input arrives, bytes or either kind of stream, into a Tokio reader. The
second runs the command. The third wraps its output in `TokioAsyncReadStream`, so that the next
stage can tell what kind of stream it received.
[`PipeHandlers`](/docs/reference/providers/handler/pipe_handlers) is CGP's combinator for running
providers in sequence, each one's output feeding the next.

The simpler stages have no dispatcher. `SimpleExec` accepts only bytes, so a `StreamingExec` stage
cannot feed it directly: the compiler rejects the pair until a `StreamToBytes` stage collects the
stream in between. The [streams and input dispatch](../architecture/streams-and-input-dispatch.md)
page has the table of what every stage accepts and produces.

## The pattern

This program shows a **typed pipeline**: stages connected in order, with the compiler checking that
each stage can accept what the one before it produces. The check happens when the program is
compiled, so a pipeline whose stages do not fit never runs. The next example,
[`http_checksum_client`](./http-checksum-client.md), shows how a stage accepts more than one kind of
input.

The cost is the strictness that types bring. A shell passes bytes between any two commands, while a
Hypershell pipeline rejects a pair of stages that do not fit, and the program has to add an adapter
stage between them.

## Where to go next

- [`http_checksum_client`](./http-checksum-client.md): the next example, with `curl` replaced by a
  native HTTP request.
- [Streams and input dispatch](../architecture/streams-and-input-dispatch.md): the stream types and
  the dispatchers.
- [Handlers](/docs/concepts/handlers): CGP's family of computation components, including the
  combinators that compose them.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
