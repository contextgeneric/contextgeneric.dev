---
sidebar_label: 'save_webpage'
sidebar_position: 7
description: 'A Hypershell program that streams an HTTP download straight into a file whose path comes from a field of the context.'
---

# Download a web page to a file

This program downloads the Nixpkgs manual and writes it to a file as it arrives, without holding the
whole page in memory. It is an example from [Hypershell](../index.md), a shell-scripting language
whose programs are Rust types, built with [CGP](/docs/). The page shows a pipeline whose last stage
produces nothing, and a file path read from the context, the type the program runs on.

:::tip

### New to CGP?

This page builds on [`http_checksum_client`](./http-checksum-client.md), which introduces the HTTP
stage and its input dispatch. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is
the quickest start, and [Handlers](/docs/concepts/handlers) explains the computations whose output
types a pipeline chains.

:::

## The problem

The task is to download a web page into a file, writing it as it arrives rather than holding the
whole page in memory, with the URL and the file path both chosen when the program runs.

### Without CGP

`curl -o manual.html <url>` does this in a shell, and it is the simpler tool for a download run by
hand. In Rust, streaming a response to disk means a loop that reads the body chunk by chunk and
writes each chunk to a file, with the URL and the path passed in as arguments. This page expresses
the download as a two-stage pipeline whose last stage writes to a file and produces nothing, so a
download is one more program in the language rather than a loop written for the occasion.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example save_webpage
```

It needs the network. It writes the manual page to `nix_manual.html` in the directory the command is
run from, and then prints:

```text
Webpage saved to nix_manual.html
```

Run from the repository root, it writes the file into the checkout.

## The program

From
[`save_webpage.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/save_webpage.rs):

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<
        GetMethod,
        FieldArg<"url">,
        WithHeaders[ ],
    >
    |   WriteFile<FieldArg<"file_path">>
};

#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub url: String,
    pub file_path: String,
}
```

The context joins `HypershellNamespace`, and `main` sets `file_path` to the relative path
`nix_manual.html`.

## A stage that ends the pipeline

`WriteFile` takes the path to write as its parameter, and the stage's input as the content. Its path
is a `FieldArg`, so the program reads it from the context like any other runtime value. The path
position asks for a file path rather than a string, so the same argument syntax that produced a URL
in [`nix_manual`](./nix-manual.md) produces a path here.

The request's output is a futures reader, and `WriteFile` accepts it because its wiring starts with
the same input dispatcher as `StreamingExec`, described in
[`http_checksum_client`](./http-checksum-client.md#the-stream-kinds-are-converted-by-dispatch). The
file is written as the response streams in.

`WriteFile` produces `()`, so the program's output is `()` and `handle` returns `Result<(), Error>`.
The compiler computes that from the last stage; `main` only prints its message once the call
returns.

## The pattern

This program shows a **pipeline whose output type follows from its last stage**. A caller of
`handle` chooses only the input; what comes back is decided by the stages the program names, through
their providers. A program ending in `WriteFile` returns nothing, and one ending in a decoding stage
returns the decoded value, as [`github_issues`](./github-issues.md) shows.

The cost is that the signature is implicit. Nothing in `Program` names its output type; a reader
works it out from the last stage's provider, and a caller who expects the wrong type learns so from
a compile error at the call rather than from a declaration.

## Where to go next

- [`github_issues`](./github-issues.md): the next example, which calls a JSON API.
- [Streams and input
  dispatch](../architecture/streams-and-input-dispatch.md#which-stages-accept-which-inputs): what
  each stage accepts and produces.
- [Handlers](/docs/concepts/handlers): the computation interface whose output type the program's
  signature follows from.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
