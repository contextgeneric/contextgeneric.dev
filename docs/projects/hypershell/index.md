---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'Hypershell is a Rust DSL for shell-script-like programs, where a pipeline is a Rust type and the context that runs it is its interpreter, built with CGP.'
---

# Hypershell

Hypershell is a small language for writing shell-script-like programs in Rust, built with
[CGP](/docs/), a language extension for Rust with pluggable trait implementations at compile-time. A
Hypershell program is a Rust *type*, and the context that runs it, a type whose wiring chooses how
each piece behaves, is its interpreter. So both the language's syntax and its meaning can be
extended without editing the crates that define it.

Hypershell is an experimental proof of concept. It is not a replacement for a shell, and it builds
only on nightly Rust. It is also the most complete example of a type-level language built with CGP,
and these pages show what its design makes possible while being candid about its gaps.

## A program that is a type

This is the smallest Hypershell program. It runs `echo hello world!` and prints the output:

```rust
use hypershell::prelude::*;

pub type Program = hypershell! {
        SimpleExec<
            StaticArg<"echo">,
            WithStaticArgs["hello", "world!"],
        >
    |   StreamToStdout
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    HypershellCli
        .handle(PhantomData::<Program>, Vec::new())
        .await?;

    Ok(())
}
```

```text
hello world!
```

`Program` is a type alias, not a value. Each piece of it, such as `SimpleExec` or `StreamToStdout`,
is an empty struct that says nothing about how it runs. The `hypershell!` macro only rewrites the
shell-like surface into that type: the `|` becomes a pipeline, the brackets a type-level list, and
each string a type-level string.

Running the program is a single `handle` call on a context. `HypershellCli` is the context here, and
this is its entire definition:

```rust
pub struct HypershellCli;

delegate_components! {
    HypershellCli {
        namespace HypershellNamespace;
    }
}
```

The one line joins `HypershellNamespace`, which routes each piece of syntax to the implementation,
or **provider**, that interprets it. The compiler resolves every choice while it type-checks the
program, so the call compiles to direct calls to those providers, with no interpreter running and no
lookup at run time.

## What the design shows

Hypershell exists to show that CGP's wiring is a general mechanism, and it shows three things a
reader can reuse in their own code.

**A language's meaning can live entirely in its wiring.** Because a program only names syntax, the
same program can run differently under two contexts that wire different providers. The
[architecture](./architecture/index.md) explains how one provider interprets each piece of syntax
and how the language is assembled from bundles of providers.

**A library can hand its users a whole language in one line.** `HypershellNamespace` collects every
route the language needs, so a context that joins it can run any program in the base language. A
context that adds fields for its programs to read is still a few lines long; see
[`hello_name`](./examples/hello-name.md).

**Anyone can extend the language.** New syntax, a new provider, and a new namespace that inherits
the base can all live in another crate. The
[`http_checksum_native`](./examples/http-checksum-native.md) example joins a language extended with
native checksums by changing one namespace name.

Writing a program uses the language's syntax, and running one uses a context, so a user who only
writes programs meets little of CGP. Extending the language is where CGP's providers and wiring
become the subject, and that is where the learning cost falls.

## Status and costs

Hypershell is a demonstration, and these are the costs a reader evaluating it should know first:

- **Nightly Rust.** The repository pins nightly and enables the next-generation trait solver with
  `-Z next-solver=globally`. The largest examples, which nest whole programs inside others,
  exhausted memory while type-checking on stable Rust without it, growing `rustc` to about 7 GB. On
  nightly with the new solver, the same check took about 9 seconds and 460 MB. See [crate
  layout](./architecture/crate-layout.md#what-the-workspace-needs-to-build).
- **Programs are fixed at compile time.** A program is a type, so it cannot be loaded or changed
  while the application runs. Values a program needs at run time, such as a URL, come from fields of
  its context.
- **Error messages are long.** A mistake in a program is a compile error, and the compiler's raw
  message lists every step it took before it failed. Checking the program with `check_components!`
  and reading the error with [`cargo cgp check`](/docs/cargo-cgp/check) helps: `cargo cgp check`
  leads with the root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does
  not yet reshape every class. The [debugging guide](./guides/debugging.md) shows the common
  failures.
- **It is a demonstration.** Hypershell is lightly tested and not meant for production use; its
  value is as a worked example. The [limitations page](./limitations.md) describes the limits of its
  design.

## The examples

The examples are the best way into Hypershell. Each is a runnable program from the repository,
walked through as a short tutorial, in the order they teach:

- [`hello`](./examples/hello.md) — a program is a type, and a context runs the whole language from
  one line of wiring.
- [`hello_name`](./examples/hello-name.md) — a context supplies the runtime values a program reads
  from its fields.
- [`http_checksum_cli`](./examples/http-checksum-cli.md) — three streaming commands running
  concurrently, as in a shell pipeline.
- [`http_checksum_client`](./examples/http-checksum-client.md) — a native HTTP request feeding
  external commands, with the stream types converted by dispatch on the input.
- [`http_checksum_native`](./examples/http-checksum-native.md) — the same pipeline in native stages,
  from a language extended through an inheriting namespace.
- [`nix_manual`](./examples/nix-manual.md) — a static URL and an argument list mixing literals and
  fields.
- [`save_webpage`](./examples/save-webpage.md) — a streaming download written to a file.
- [`github_issues`](./examples/github-issues.md) — a URL built from fields, a header, and JSON
  decoded into a Rust type.
- [`rust_playground`](./examples/rust-playground.md) — a program whose input and output are Rust
  values encoded as JSON.
- [`bluesky`](./examples/bluesky.md) — a pipeline that streams indefinitely.
- [`bluesky_websocket`](./examples/bluesky-websocket.md) — a language extension wired onto a single
  context rather than a namespace.

## The rest of the section

The [architecture](./architecture/index.md) pages explain the design one idea at a time: programs as
types, how the language is assembled, how stages dispatch on their input, and how the crates split.
The guides cover [writing a program](./guides/writing-a-program.md) and [debugging
one](./guides/debugging.md), and the [limitations page](./limitations.md) describes the limits of
its design.

## Running it

Clone the [repository](https://github.com/contextgeneric/hypershell) and run an example from its
root, such as `cargo run --example hello`. The repository's toolchain file selects nightly Rust, and
its `.cargo/config.toml` enables the new trait solver, so no setup is needed beyond `rustup`. Most
examples need the network and a few common command-line tools; each example page says which.

To use Hypershell from your own crate, depend on the repository by git, and copy its
`rust-toolchain.toml` and `.cargo/config.toml` so that your crate builds with the same toolchain and
solver. The `hypershell` crates published on crates.io are an earlier release, built on an older
version of CGP with a different design, and the code on these pages does not compile against them.

## Where to go next

- [`hello`](./examples/hello.md): the first example, and the one to read first.
- [Type-level DSLs](/docs/concepts/type-level-dsls): the CGP technique behind Hypershell, explained
  with a smaller language.
- [Hello World](/docs/tutorials/hello): a first CGP program, for a reader new to CGP.
- [Hypershell: a type-level DSL for shell-scripting](/blog/hypershell-release): the post that
  announced the project, with its history and motivation. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
