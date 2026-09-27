---
sidebar_label: 'bluesky'
sidebar_position: 10
description: 'A Hypershell pipeline that streams the Bluesky firehose through grep until it is stopped, with a tool provisioned on the fly by nix-shell.'
---

# Stream a live feed through `grep`

This program reads the Bluesky firehose, a live stream of every public post, and prints the events
that mention a keyword, until it is stopped. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The page shows a
pipeline that never ends on its own, and a stage that passes a whole shell command line to another
program.

:::tip

### New to CGP?

This page builds on [`http_checksum_cli`](./http-checksum-cli.md), which introduces streaming
stages. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is the quickest start, and
[Type-level DSLs](/docs/concepts/type-level-dsls) explains how a program written as a type still
processes data at run time.

:::

## The problem

The task is to read the Bluesky firehose, a live stream of every public post, and print the events
that mention a keyword, for as long as the program runs. The pipeline never finishes on its own, so
every stage has to process data as it arrives and keep running.

### Without CGP

In a shell, `websocat … | grep "$keyword"` does exactly this, and it is the right tool for watching
a feed by hand. In Rust, the same pipeline means spawning both commands and connecting them by hand,
as in [`http_checksum_cli`](./http-checksum-cli.md), with the added need that nothing waits for a
stage to finish before the next one starts. This page shows the same streaming stages running a
pipeline that has no end.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example bluesky
```

It needs the network, `grep`, and `nix-shell`, which fetches the `websocat` WebSocket client the
first time it runs. It prints firehose events that contain `love` as they arrive, and runs until it
is stopped with Ctrl-C. In one two-minute run it printed about 45 KB.

## The program

From
[`bluesky.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/bluesky.rs):

```rust
pub type Program = hypershell! {
        StreamingExec<
            StaticArg<"nix-shell">,
            WithStaticArgs [
                "-p",
                "websocat",
                "--run",
                "websocat -nU wss://jetstream1.us-west.bsky.network/subscribe",
            ],
        >
    |   StreamingExec<
            StaticArg<"grep">,
            WithArgs [ FieldArg<"keyword"> ],
        >
    |   StreamToStdout
};

#[derive(HasField)]
pub struct MyApp {
    pub keyword: String,
}
```

The context, the type the program runs on, joins `HypershellNamespace`, and `main` sets `keyword` to
`"love"`.

## A pipeline that does not end

Both stages are `StreamingExec`, which returns its command's output as a stream at once and keeps it
flowing while the command runs; see
[`http_checksum_cli`](./http-checksum-cli.md#each-stage-streams). `websocat` never closes the
connection, so the first stage's output never ends, and neither does the program. Each event reaches
`grep`, and each matching line reaches standard output, as soon as it arrives.

The first stage's last argument is a whole command line, passed as one literal to `nix-shell --run`.
Hypershell passes arguments to a process without a shell, so the string is not split; `nix-shell`
runs it in its own shell with `websocat` available.

## The pattern

This program shows that a Hypershell pipeline has the **streaming behavior of a shell pipeline**:
stages run concurrently and data flows through them as it is produced, with no stage waiting for the
one before it to finish. It is what makes the language suitable for long-running feeds and large
downloads rather than only for commands that finish quickly.

The cost is in how failures surface. A streaming stage has returned before its command finishes, so
a failing command is reported only when its output stream ends, and a stream that never ends never
reports; a stalled feed here looks the same as a quiet one.

## Where to go next

- [`bluesky_websocket`](./bluesky-websocket.md): the next example, which reads the same feed with a
  native WebSocket stage instead of `websocat`.
- [Streams and input
  dispatch](../architecture/streams-and-input-dispatch.md#how-a-streaming-stage-runs): how a
  streaming stage runs and reports failure.
- [Type-level DSLs](/docs/concepts/type-level-dsls): how a program fixed at compile time still
  processes data as it arrives.
- [Hypershell: a type-level DSL for shell-scripting in Rust](/blog/hypershell-release): the post
  that announced Hypershell, with the project's motivation. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
