---
sidebar_label: 'hello'
sidebar_position: 1
description: 'The smallest Hypershell program: echo hello world! written as a Rust type, run on a context whose whole definition is one line of wiring.'
---

# Run a shell command from a program written as a type

This program runs `echo hello world!` and prints the result. It is the smallest program in
[Hypershell](../index.md), a shell-scripting language whose programs are Rust types, built with
[CGP](/docs/), a language extension for Rust with pluggable trait implementations at compile-time.
The page shows the two ideas every other Hypershell example builds on: a program is a type, and the
context it runs on, itself a type, gets the whole language from one line of wiring.

:::tip

### New to CGP?

This page can be read without knowing CGP. For background, the [Hello World
tutorial](/docs/tutorials/hello) introduces CGP's contexts and wiring, and [Type-level
DSLs](/docs/concepts/type-level-dsls) explains how a program can be a type, using a smaller language
than this one.

:::

## The problem

The task itself is trivial on purpose: run `echo hello world!` and print what it prints. The problem
this page is about is the shape of the program, not its output. Hypershell's aim is a shell pipeline
written as a value that the compiler checks, that different interpreters can run differently, and
that anyone can extend with new kinds of stage.

### Without CGP

For this task alone, a shell script, `echo hello world!`, or a few lines of Rust with
`std::process::Command` are simpler, and they are what a reader should use to run one command.
Neither makes the program a thing of its own that can be inspected or reinterpreted: the shell reads
text, and the Rust code runs the command where it is written. The usual way to make a program a
value in Rust is an enum of its instructions and an interpreter that matches on it, and that design
is closed: a new kind of instruction means editing the enum and the interpreter, in the crate that
owns them. This page shows the smallest program in a language that avoids both limits.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example hello
```

It needs only the `echo` command, and prints:

```text
hello world!
```

## The program is a type

The whole program is one type alias, from
[`hello.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/hello.rs):

```rust
use hypershell::prelude::*;

pub type Program = hypershell! {
        SimpleExec<
            StaticArg<"echo">,
            WithStaticArgs["hello", "world!"],
        >
    |   StreamToStdout
};
```

`SimpleExec` runs a command and collects its output, `StaticArg` is a literal argument, and
`StreamToStdout` writes whatever it receives to standard output. The `|` connects them as a
pipeline, as in a shell.

Each of these names is an empty struct. `SimpleExec` holds its two parameters in `PhantomData` and
has no methods, so nothing in the program says how a command is run. That choice is made by the
context that runs the program, which is the reason the language can be reinterpreted or extended
without editing it. The [abstract syntax](../architecture/abstract-syntax.md) page describes every
kind of syntax the language has.

## The macro only rewrites tokens

`hypershell!` gives the program its shell-like look, and it does nothing else. It turns the `|` into
a `Pipe` over a type-level list, each bracketed list into a
[`Product!`](/docs/reference/macros/product), and each string literal into a
[`Symbol!`](/docs/reference/macros/symbol), a string encoded as a type. The program above is exactly
this type, which can be written without the macro:

```rust
pub type Program = Pipe<Product![
    SimpleExec<
        StaticArg<Symbol!("echo")>,
        WithStaticArgs<Product![Symbol!("hello"), Symbol!("world!")]>,
    >,
    StreamToStdout,
]>;
```

The macro emits `Pipe`, `Product!`, and `Symbol!` without qualifying them, which is why the program
imports `hypershell::prelude::*` rather than the macro alone.

## One call runs the program

The program runs with one call to `handle`:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    HypershellCli
        .handle(PhantomData::<Program>, Vec::new())
        .await?;

    Ok(())
}
```

The program is passed as `PhantomData::<Program>`, since it is a type and has no value. The second
argument is the pipeline's input, here an empty byte buffer that becomes `echo`'s standard input.
`handle` is the method of CGP's [`Handler`](/docs/reference/components/handler/handler) component,
the asynchronous and fallible member of CGP's [handler family](/docs/concepts/handlers), which
Hypershell uses as its interpreter interface. The compiler works out the output type from the
program: `StreamToStdout` produces `()`, so the call returns `Result<(), Error>`.

## A context that is one line of wiring

`HypershellCli` is the **context**: the type the program runs on. It is a type that stands for one
set of language choices, and it has no fields, because this program reads no runtime values. Its
entire definition, in the `hypershell` crate, is this:

```rust
pub struct HypershellCli;

delegate_components! {
    HypershellCli {
        namespace HypershellNamespace;
    }
}
```

[`delegate_components!`](/docs/reference/macros/delegate_components) records a context's wiring,
which is the choice of implementation, or **provider**, for each piece of behavior. The one
`namespace` statement joins `HypershellNamespace`, a reusable wiring table that routes every piece
of the base language to the provider that interprets it. So `SimpleExec` reaches a provider that
spawns a Tokio process, and `StreamToStdout` one that copies bytes to standard output.

The compiler resolves all of these choices while it type-checks `main`. The call to `handle`
compiles to direct calls into those providers, with no interpreter running at run time and no lookup
table. How the namespace is built from the language's parts is on the
[assembly](../architecture/assembly.md) page.

## The pattern

This program shows a **type-level DSL**: a language whose programs are types made of empty syntax
structs, with the meaning of each piece chosen by the wiring of the context that runs it. The
technique is explained with a small arithmetic language in [Type-level
DSLs](/docs/concepts/type-level-dsls). The other half of the pattern is a library shipping its whole
language as a [namespace](/docs/concepts/namespaces), so that a user's context needs one line to run
it.

The costs are the ones that come with the technique. A program is fixed at compile time, the
compiler does real work to resolve a large program, and a mistake in one is a compile error that can
be long to read. Hypershell pays off when a program's parts should be reusable and its meaning
replaceable.

## Where to go next

- [`hello_name`](./hello-name.md): the next example, which reads a runtime value from the context.
- [Abstract syntax](../architecture/abstract-syntax.md): every kind of syntax, and the macro's
  rules.
- [Assembly](../architecture/assembly.md): how `HypershellNamespace` routes each piece of syntax.
- [Type-level DSLs](/docs/concepts/type-level-dsls): the CGP technique behind the language.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
