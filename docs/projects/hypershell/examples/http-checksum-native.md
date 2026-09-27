---
sidebar_label: 'http_checksum_native'
sidebar_position: 5
description: 'A fully native Hypershell checksum pipeline, run on a context that joins a language extended with new syntax through an inheriting namespace.'
---

# Extend the language with native checksums

This program computes the same digest as the two before it with every stage native: an HTTP request,
a SHA-256 checksum, and hexadecimal encoding. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The checksum
stages are not part of the base language. The page shows how a language extension defined in other
crates is joined by changing one namespace name.

:::tip

### New to CGP?

This page builds on [`http_checksum_client`](./http-checksum-client.md). For CGP itself, the [Hello
World tutorial](/docs/tutorials/hello) is the quickest start. The page uses CGP's
[namespaces](/docs/concepts/namespaces), reusable wiring tables that a namespace or a context can
inherit, and explains what it needs of them.

:::

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example http_checksum_native
```

It needs the network and no other tools, and prints the same digest of the Nixpkgs manual page as
[`http_checksum_cli`](./http-checksum-cli.md).

## Two stages from an extension crate

The program replaces `sha256sum` and `cut` with `Checksum` and `BytesToHex`, from
[`http_checksum_native.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/http_checksum_native.rs):

```rust
use hypershell::prelude::*;
use hypershell_examples::namespaces::HypershellChecksumNamespace;
use hypershell_hash_components::dsl::{BytesToHex, Checksum};
use reqwest::Client;
use sha2::Sha256;

pub type Program = hypershell! {
    StreamingHttpRequest<
        GetMethod,
        FieldArg<"url">,
        WithHeaders[ ],
    >
    | Checksum<Sha256>
    | BytesToHex
    | StreamToStdout
};
```

The two new stages come from `hypershell-hash-components`, a crate that depends on neither the
Hypershell core nor Tokio. `Checksum<Sha256>` names its hash algorithm as a type from the `sha2`
crate, and produces the digest as raw bytes, which `BytesToHex` encodes as text for printing.

## Joining the extended language

The context, the type the program runs on, differs from the previous example's in one name:

```rust
#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub url: String,
}

delegate_components! {
    MyApp {
        namespace HypershellChecksumNamespace;
    }
}
```

`HypershellNamespace` does not route `Checksum` or `BytesToHex`, so a context joined to it cannot
run this program. `HypershellChecksumNamespace` is a namespace defined in the examples crate that
inherits `HypershellNamespace` and adds routes for the two new stages:

```rust
cgp_namespace! {
    new HypershellChecksumNamespace: HypershellNamespace {
        @cgp.extra.handler.HandlerComponent.[
            <Hasher> Checksum<Hasher>,
            BytesToHex,
        ]:
            HypershellChecksumProvider,
    }
}
```

[`cgp_namespace!`](/docs/reference/macros/cgp_namespace) defines the namespace, and the part after
the colon names the namespace it inherits. Its one entry routes both syntaxes to
`HypershellChecksumProvider`, a bundle of providers, or **aggregate provider**, also defined in the
examples crate:

```rust
delegate_components! {
    new HypershellChecksumProvider {
        open {HandlerComponent};

        @HandlerComponent.<Hasher> Checksum<Hasher>:
            PipeHandlers<Product![
                HandleToFuturesStream,
                HandleStreamChecksum,
            ]>,

        @HandlerComponent.BytesToHex:
            HandleBytesToHex,
    }
}
```

`HandleStreamChecksum` and `HandleBytesToHex` are the providers from the hash crate that do the
work. `HandleToFuturesStream` in front of the checksum is an input dispatcher, the counterpart of
the one in [`http_checksum_client`](./http-checksum-client.md), so `Checksum` accepts bytes or
either kind of stream. The same two-layer shape, a bundle that maps syntax to providers and a
namespace route that points at the bundle, is how the whole base language is assembled; see
[assembly](../architecture/assembly.md).

## Try a change

Remove the `BytesToHex` stage, so that the raw digest flows straight into `StreamToStdout`:

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<
        GetMethod,
        FieldArg<"url">,
        WithHeaders[ ],
    >
    | Checksum<Sha256>
    | StreamToStdout
};
```

Checked with `check_components!`, as on the [`hello_name`](./hello-name.md#try-a-change) page,
[`cargo cgp check`](/docs/cargo-cgp/check) names the missing entry, shown here with the long types
abridged:

```text
error[E0277]: [CGP-E002] the provider trait `Handler<Pipe<…>, GenericArray<u8, …>>` with context `MyApp` is not implemented for provider `Call<StreamToStdout>`
   = note: root cause: [CGP-E110] provider `HandleToTokioAsyncRead` does not contain any delegate entry for `@HandlerComponent.StreamToStdout.GenericArray<u8, …>`
```

The digest is a `GenericArray<u8, …>`, and `StreamToStdout`'s input dispatcher has no entry for that
type. The path in the message is the syntax followed by the input type, which is exactly the key the
dispatcher failed to match. The fix is to convert the value into a type the dispatcher lists, which
is what `BytesToHex` does. `cargo cgp check` leads with the root cause for the classes it
recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class.

## The pattern

This program shows **extending a language through a namespace that inherits it**. A library
publishes its language as a namespace, and anyone can publish a larger language that inherits it and
adds new syntax, from their own crates and without editing the library. A context chooses which
language it runs by the namespace it joins, and extensions can build on each other: the examples
crate has a second extension built on this one. The ideas are explained in
[Namespaces](/docs/concepts/namespaces) and [Aggregate
providers](/docs/concepts/aggregate-providers).

The limit is that an extension can only add. It cannot change what the syntax it inherits does,
because CGP does not let an inheriting namespace override a choice its parent has made. The
[limitations](../limitations.md#extensions-add-to-the-language-rather-than-replace-it) page says
more about what this means.

## Where to go next

- [`nix_manual`](./nix-manual.md): the next example, with a static URL and a mixed argument list.
- [`bluesky_websocket`](./bluesky-websocket.md): an extension wired onto one context instead of a
  namespace.
- [Assembly](../architecture/assembly.md): the bundles and routes the base language is built from.
- [Namespaces](/docs/concepts/namespaces): inheriting a namespace, and why an inherited binding
  cannot be overridden.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
