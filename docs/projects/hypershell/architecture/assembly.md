---
sidebar_label: 'Assembly'
sidebar_position: 2
description: 'How one line of wiring gives a Hypershell context the whole language: providers gathered into bundles, and bundles into a namespace a context joins.'
---

# Assembly

How does one line of wiring give a context the whole Hypershell language? The language has dozens of
pieces of syntax, and each needs a provider, so something must gather them.
[Hypershell](../index.md) is a small shell-scripting language built with [CGP](/docs/). This page
explains the three layers that do the gathering, how a context can add to them, and how one piece of
syntax is traced through all three to its provider.

## Three layers, from providers to a context

The wiring is built up in three layers, each collecting the one below it:

- **A bundle** collects the providers of one backend. The Tokio bundle knows how to run commands and
  read files; the `reqwest` bundle knows how to send HTTP requests.
- **The namespace**, `HypershellNamespace`, collects the bundles into the whole language, and adds
  the few choices that are not tied to one backend, such as the error type.
- **A context** joins the namespace, and gets everything it collects.

The split lets each layer be reused. A bundle can serve any namespace that points at it, and a
namespace serves every context that joins it. The rest of this page takes the layers in turn.

## A bundle maps syntax to providers

A bundle is a table, written with CGP's
[`delegate_components!`](/docs/reference/macros/delegate_components), with one entry per piece of
syntax. These two entries are from the Tokio bundle, `HypershellTokioProvider`:

```rust
@HandlerComponent.<Path, Args> SimpleExec<Path, Args>:
    HandleSimpleExec,

@HandlerComponent.<Path, Args> StreamingExec<Path, Args>:
    PipeHandlers<Product![
        HandleToTokioAsyncRead,
        HandleStreamingExec,
        WrapTokioAsyncRead,
    ]>,
```

Each entry reads as "for this syntax, use this provider". `@HandlerComponent` at the start names the
part of CGP that runs stages, CGP's [`Handler`](/docs/reference/components/handler/handler)
interface, so these are the entries for running stages. The first says that `SimpleExec`, whatever
its command and arguments, is run by the provider `HandleSimpleExec`. The `<Path, Args>` in front of
it makes the entry cover every `SimpleExec`, the way a generic parameter does in a Rust impl. The
second entry chains three providers for `StreamingExec`, for reasons the
[streams](./streams-and-input-dispatch.md) page explains.

CGP calls a table like this an [aggregate provider](/docs/concepts/aggregate-providers): a bundle of
choices that other wiring can point at as one unit. Hypershell has one per backend crate: the base
bundle for the control syntax and arguments, and bundles for Tokio, `reqwest`, JSON, and WebSockets.

## The namespace gathers the bundles

A [namespace](/docs/concepts/namespaces) is a reusable wiring table that contexts can join.
`HypershellNamespace` routes each group of syntax to the bundle that handles it. This excerpt routes
the Tokio syntax to the Tokio bundle:

```rust
@cgp.extra.handler.HandlerComponent.[
    <Path, Args> SimpleExec<Path, Args>,
    <Path, Args> StreamingExec<Path, Args>,
    <Path, Args> CoreExec<Path, Args>,
    <Path> ReadFile<Path>,
    <Path> WriteFile<Path>,
    StreamToBytes,
    StreamToString,
    BytesToStream,
    StreamToStdout,
    ToTokioAsyncRead,
]:
    HypershellTokioProvider,
```

The bracketed list gives several pieces of syntax one destination. The long prefix,
`@cgp.extra.handler.HandlerComponent`, is the full name under which CGP registers the part that runs
stages, and a namespace's entries are written with such full names. `CoreExec` in the list is the
internal step both command stages start their process through.

The namespace also makes the choices that belong to the language as a whole. It sets the error type
to `anyhow::Error`, and it says where to find the HTTP client: in a field named `http_client`, which
is why every context that sends requests has one.

## A context joins the namespace

A context joins the namespace with one statement. Hypershell provides two ready-made contexts, which
differ only in their fields:

```rust
pub struct HypershellCli;

delegate_components! {
    HypershellCli {
        namespace HypershellNamespace;
    }
}

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

A context of your own follows the same pattern, adding a field for each value its programs read.

A context can also add entries of its own next to the `namespace` statement, to use syntax the
namespace does not include. [`bluesky_websocket`](../examples/bluesky-websocket.md) adds a WebSocket
stage this way. When several contexts need the same additions, a namespace that inherits
`HypershellNamespace` and adds them is the tidier choice, as
[`http_checksum_native`](../examples/http-checksum-native.md) shows.

## Following one piece of syntax

Running `SimpleExec` on `HypershellCli` passes through all three layers:

1. `HypershellCli` has no entry for `SimpleExec` itself, so the question goes to the namespace it
   joined.
2. `HypershellNamespace` routes `SimpleExec` to the Tokio bundle.
3. The Tokio bundle maps `SimpleExec` to its provider, `HandleSimpleExec`.
4. `HandleSimpleExec` needs things of its own, such as a way to start a process, and each is found
   by the same walk, starting again from the context.

The compiler does this walk while it type-checks the program, so none of it happens at run time: the
call becomes a direct call to `HandleSimpleExec`. When something along the way is missing,
[`cargo cgp check`](/docs/cargo-cgp/check) prints the same walk step by step, which is what the
[debugging guide](../guides/debugging.md) teaches a reader to follow.

## What it costs

The layers make the language easy to reuse and harder to trace. To answer "what runs for
`SimpleExec`?" a reader looks in the context, the namespace, and a bundle, where a plain Rust
program would have one function to read. The walk above is short once it is familiar, but it is
three tables rather than one.

Adding a piece of syntax to the language means registering it twice, once in a bundle and once in
the namespace, and nothing checks that the two agree until a program uses the new syntax. That is
the price of letting bundles be reused by other namespaces.

A namespace also fixes the choices it makes. A context that joins `HypershellNamespace` can add
syntax, but it cannot swap the provider for syntax the namespace already covers, such as
`SimpleExec`; CGP treats that as a conflict. The [namespaces](/docs/concepts/namespaces) page
explains the rule.

## Where to go next

- [Streams and input dispatch](./streams-and-input-dispatch.md): why some entries chain several
  providers.
- [Namespaces](/docs/concepts/namespaces) and [Aggregate
  providers](/docs/concepts/aggregate-providers): the CGP ideas behind the layers.
- [`hello`](../examples/hello.md): the one-line context, running a program.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
