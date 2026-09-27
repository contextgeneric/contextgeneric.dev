---
sidebar_label: 'rust_playground'
sidebar_position: 9
description: 'A Hypershell program whose input and output are Rust values: a struct sent as JSON to the Rust Playground, and the reply decoded into another.'
---

# Send a Rust value as JSON and decode the reply

This program takes a Rust struct holding some code, posts it as JSON to the Rust Playground's gist
API, and decodes the reply into another struct. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The page shows a
program whose input and output are both Rust values rather than bytes, run on a context, the type a
program runs on, that the library provides.

:::tip

### New to CGP?

This page builds on [`github_issues`](./github-issues.md), which introduces decoding JSON into a
Rust type. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is the quickest start,
and [Handlers](/docs/concepts/handlers) explains how a computation's output type follows from its
input and its code.

:::

## The problem

The task is to post a Rust value to the Rust Playground's gist API as JSON and decode the reply into
another Rust value. Unlike the earlier examples, the program's input and its output are both Rust
values, not bytes, so the pipeline has to start and end in Rust's types.

### Without CGP

With `reqwest`, the whole task is one chain: `.json(&request)` to encode and send, and
`.json::<Response>()` to decode, and for a single call that is simpler than a pipeline. This page
writes the same steps as three stages, encoding, the request, and decoding, so each is a stage the
language provides and the compiler checks that each stage's output is what the next one reads.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example rust_playground
```

It needs the network. **Running it publishes a public GitHub gist** containing a hello-world
program, through the Playground's API, so these pages do not record its output; the program compiles
against the current code, and its behavior follows from the stages described below.

## The program

From
[`rust_playground.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/rust_playground.rs):

```rust
pub type Program = hypershell! {
    EncodeJson
    |   SimpleHttpRequest<
            PostMethod,
            StaticArg<"https://play.rust-lang.org/meta/gist">,
            WithHeaders [
                Header<
                    StaticArg<"Content-Type">,
                    StaticArg<"application/json">,
                >
            ],
        >
    |   DecodeJson<Response>
};

#[derive(Serialize)]
pub struct Request {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: String,
    pub url: String,
    pub code: String,
}
```

## Rust values at both ends

The program starts with `EncodeJson`, so its input is any value that implements `Serialize`, encoded
into JSON bytes. Those bytes are the body of the `POST` request, since `SimpleHttpRequest` accepts
anything that `reqwest` can send as a body. The reply is decoded by `DecodeJson<Response>`, so the
program's output is a `Response`.

`main` passes a `Request` as the input and receives a `Response` as the output:

```rust
let app = HypershellHttp {
    http_client: Client::new(),
};

let input = Request {
    code: "fn main() { println!(\"Hello, world!\"); }".to_owned(),
};

let output = app.handle(PhantomData::<Program>, input).await?;
```

So a Hypershell program is not limited to bytes and streams. The first stage decides what the
program accepts, and the last decides what it returns, and both can be Rust types checked by the
compiler.

## A context the library provides

The program reads no field except the HTTP client, so it runs on `HypershellHttp`, the second
context the `hypershell` crate defines. It is `HypershellCli` with one field:

```rust
#[derive(HasField)]
pub struct HypershellHttp {
    pub http_client: Client,
}

delegate_components! {
    HypershellHttp {
        namespace HypershellNamespace;
    }
}
```

The two predefined contexts are wired identically. They differ only in their fields, which is all a
context needs to add for the programs it runs.

## The pattern

This program shows **a program's type fixing its interface**: the first stage's accepted input and
the last stage's output are the program's signature, worked out by the compiler from the wiring. The
same `handle` call serves a program of bytes, of streams, or of Rust structs, and a caller passing
the wrong input type gets a compile error rather than a runtime failure.

The cost is the one [`save_webpage`](./save-webpage.md) names: the signature is never written down.
It lives in the first and last stages' wiring, so a reader finds a program's input and output types
by following that wiring rather than by reading a declaration.

## Where to go next

- [`bluesky`](./bluesky.md): the next example, a pipeline that never ends on its own.
- [Assembly](../architecture/assembly.md#a-context-joins-the-namespace): the two predefined contexts
  and how a context joins the language.
- [Handlers](/docs/concepts/handlers): how a computation's output type follows from its code and
  input.
- [Hypershell: a type-level DSL for shell-scripting in
  Rust](/blog/hypershell-release#json-encoding): the post that announced Hypershell, which
  introduced this program. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
