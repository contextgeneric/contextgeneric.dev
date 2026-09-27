---
sidebar_label: 'nix_manual'
sidebar_position: 6
description: 'A Hypershell program that filters a web page through tr and grep, with a static URL and an argument list mixing literals and fields.'
---

# Filter a web page through two commands

This program downloads the Nixpkgs manual, converts it to upper case with `tr`, and prints the lines
that mention a keyword, found with `grep`. It is an example from [Hypershell](../index.md), a
shell-scripting language whose programs are Rust types, built with [CGP](/docs/). The page looks at
Hypershell's argument syntax: a URL written as a literal, and an argument list that mixes a literal
with a field.

:::tip

### New to CGP?

This page builds on [`hello_name`](./hello-name.md), which introduces `FieldArg`, and
[`http_checksum_client`](./http-checksum-client.md), which introduces the HTTP stage. For CGP
itself, the [Hello World tutorial](/docs/tutorials/hello) is the quickest start, and [Impl-side
dependencies](/docs/concepts/impl-side-dependencies) explains how an interpreter asks its context
for what it needs.

:::

## The problem

The task is to download a web page, convert it to upper case, and print the lines that mention a
keyword chosen at run time. The interesting part is the command lines: `tr` takes fixed arguments,
while `grep` takes a mix of a fixed flag and a value from the running application.

### Without CGP

In a shell, this is `curl … | tr '[:lower:]' '[:upper:]' | grep -i "$keyword"`, one line that
mixes literals and a variable freely, and for a one-off search it is the simpler tool. In Rust, each
command's argument list is built as a vector of strings before the command is spawned, so the mix of
fixed and runtime values is assembled by hand, in code that sits apart from the pipeline it feeds.
This page writes each argument list inside the program, with each argument saying where its value
comes from.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example nix_manual
```

It needs the network and the `tr` and `grep` commands. It prints, in upper case, the lines of the
manual page that contain `Nix`.

## The program

From
[`nix_manual.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/nix_manual.rs):

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<
            GetMethod,
            StaticArg<"https://nixos.org/manual/nixpkgs/unstable/">,
            WithHeaders<Nil>,
        >
    |   StreamingExec<
            StaticArg<"tr">,
            WithStaticArgs [
                "[:lower:]",
                "[:upper:]",
            ],
        >
    |   StreamingExec<
            StaticArg<"grep">,
            WithArgs [
                StaticArg<"-i">,
                FieldArg<"keyword">,
            ],
        >
    | StreamToStdout
};

#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub keyword: String,
}
```

The context, the type the program runs on, joins `HypershellNamespace` as in the earlier examples,
and `main` sets `keyword` to `"Nix"`.

## Arguments are a language of their own

Every argument in a Hypershell program is an expression that the context interprets, and the kind of
value it must produce depends on where it appears.

**A URL can be a literal.** The request's URL is a `StaticArg`, where the earlier examples read it
from a field. It is still interpreted when the program runs: the URL position asks for a parsed
`url::Url`, so the literal string is parsed, and a malformed one is an error raised through the
context.

**Headers can be written without the macro's brackets.** `WithHeaders<Nil>` is the empty header list
written as the plain type. It is exactly what `WithHeaders[]` expands to, since an empty
[`Product!`](/docs/reference/macros/product) list is `Nil`. Either form may be used; the macro only
rewrites the bracketed form into this one.

**One argument list can mix kinds.** `grep`'s arguments are `WithArgs`, so the literal flag `-i` and
the `keyword` field sit in one list, as in [`hello_name`](./hello-name.md). `tr`'s arguments are all
literals, so they use `WithStaticArgs`.

Each kind of argument, a literal, a field, a joined value, an encoded value, has its own
interpreter, chosen by the context's wiring in the same way as the pipeline's stages are. Which
interpreter runs depends on the position as well: command arguments and file paths come from one
extractor and URLs from another, which is why the same `StaticArg` becomes a path in one place and a
parsed URL in another. The [abstract
syntax](../architecture/abstract-syntax.md#four-kinds-of-syntax) page lists every kind.

## The pattern

This program shows **one piece of syntax meaning different things in different places**. `StaticArg`
is always a literal, but whether it becomes a string, a file path, or a parsed URL depends on where
it appears, because each place asks the context for a different kind of value. In CGP terms, each
position requires a different trait, and the context's wiring implements each one for the same
syntax.

The cost is that a piece of syntax has no single definition to read. To know what `StaticArg` does
in a URL, a reader looks at how URLs are produced, rather than at one definition of `StaticArg`.

## Where to go next

- [`save_webpage`](./save-webpage.md): the next example, a download written to a file.
- [Writing a program](../guides/writing-a-program.md#read-runtime-values-from-a-custom-context):
  every kind of argument and where it may appear.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): how each interpreter asks the
  context for what it needs.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
