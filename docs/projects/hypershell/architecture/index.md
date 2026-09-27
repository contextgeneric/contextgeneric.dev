---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How Hypershell works, in outline: a program written as a Rust type, a context that decides what each part does, and a language built from reusable parts.'
---

# How Hypershell works

A Hypershell program looks like a shell pipeline, but it is a Rust type, and the compiler turns it
into ordinary function calls. [Hypershell](../index.md) is a small shell-scripting language built
with [CGP](/docs/), a language extension for Rust with pluggable trait implementations at
compile-time. This page explains, in outline, how a type can be a program, what decides what that
program does, and how the language is put together. Each idea has a page of its own with the
details.

## The program only says what to do

In a shell script, a pipeline is text. The shell reads it, works out which programs to start, and
connects them. A Hypershell pipeline is written in Rust instead, as a type:

```rust
pub type Program = hypershell! {
        SimpleExec<StaticArg<"echo">, WithStaticArgs["hello", "world!"]>
    |   StreamToStdout
};
```

Each name in it, such as `SimpleExec` or `StreamToStdout`, is an empty struct. It records *what* the
program asks for, "run `echo`" and "print the result", and contains no code for doing either. The
`hypershell!` macro only turns the shell-like notation into that type. The [abstract
syntax](./abstract-syntax.md) page describes every kind of piece a program can contain.

## The context decides how it is done

Because the program contains no code, something else must supply it. That is the **context**: the
type the program is run on, as in `HypershellCli.handle(PhantomData::<Program>, input)`. For each
piece of the program, the context's wiring picks an implementation, called a **provider**. One
provider knows how to run a command, another how to print a stream, another how to send an HTTP
request.

The compiler makes these choices while it type-checks the program. By the time the program runs,
each piece has become a direct call into its provider, with no interpreter and no lookup at run
time. This is what CGP is for: letting a type choose among implementations of the same interface,
and resolving the choice at compile time.

The context makes one more choice: the error type. Providers report failures without naming an error
type of their own, and the context decides what those failures become. Hypershell's standard wiring
turns them all into `anyhow::Error`, using CGP's [modular error
handling](/docs/concepts/modular-error-handling).

Two consequences follow, and they are why the design is interesting. The same program can behave
differently on two contexts that choose different providers. And a new piece of syntax, with a
provider for it, can be added from any crate, without touching the crates that define the language.

## The language is assembled from reusable parts

A context does not list a provider for every piece of the language itself. Hypershell supplies its
choices in layers, so that a user's context needs one line:

```rust
pub struct HypershellCli;

delegate_components! {
    HypershellCli {
        namespace HypershellNamespace;
    }
}
```

Each backend, such as the one that runs processes with Tokio or the one that sends HTTP requests,
publishes its providers as a bundle. `HypershellNamespace` gathers the bundles into the whole
language, and a context that joins it gets everything in it. The [assembly](./assembly.md) page
explains the layers, and how a context can add to them.

## Stages are typed, and fit together like functions

Each stage of a pipeline receives the previous stage's output, as in a shell, but here the output
has a Rust type. One stage produces bytes, another a stream, another a decoded JSON value. The
compiler checks that every stage can accept what the one before it produces, and it chooses a
stage's provider by that type as well as by the syntax. Most stages accept both bytes and streams,
so they chain freely. The [streams and input dispatch](./streams-and-input-dispatch.md) page
explains how, and where they do not.

## Each backend lives in its own crate

The syntax lives in a crate that depends only on CGP, and each backend (Tokio, `reqwest`,
`serde_json`, and the rest) lives in a crate of its own. A program's crate compiles only the
backends it uses, and no provider names the context or the error type it will run with; those are
chosen last. The [crate layout](./crate-layout.md) page lists the crates and what the workspace
needs to build.

## What the design costs

The flexibility is paid for in indirection. To find out what a piece of syntax does, a reader
follows it from the program to the context, to the namespace, to a bundle, and finally to a
provider. The compiler does that in an instant, but a person reading the code, or reading a long
compile error, has to do it too. The [debugging guide](../guides/debugging.md) shows how to read
such an error.

It also moves work to compile time. A program is fixed when it is compiled, so it cannot be loaded
or changed while an application runs, and large programs make the compiler work hard: the biggest
examples need nightly Rust's newer trait solver to type-check in reasonable memory.

For a pipeline written once and run by hand, a shell script is simpler, and for a single command a
few lines of `std::process::Command` are. The design earns its cost when a pipeline's parts should
be reusable, checked by the compiler, and replaceable without editing the program.

## Where to go next

- [Abstract syntax](./abstract-syntax.md): what a program is made of.
- [`hello`](../examples/hello.md): the design at its smallest, in a running program.
- [Type-level DSLs](/docs/concepts/type-level-dsls): the CGP technique behind Hypershell, explained
  with a smaller language.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
