---
sidebar_label: 'hello_name'
sidebar_position: 2
description: 'A Hypershell program that reads a runtime value from a field of its context with FieldArg, and the error cargo cgp check reports when the field is missing.'
---

# Read a runtime value from the context

This program runs `echo` with a greeting whose name comes from a field of the context, the type the
program runs on, at run time. It is an example from [Hypershell](../index.md), a shell-scripting
language whose programs are Rust types, built with [CGP](/docs/). This page shows where such runtime
values live, and what the compiler reports when one is missing.

:::tip

### New to CGP?

[`hello`](./hello.md) introduces the program-as-a-type idea this page builds on. For CGP itself, the
[Hello World tutorial](/docs/tutorials/hello) shows a context supplying a value from its fields, the
same idea in its simplest form, and [Implicit arguments](/docs/concepts/implicit-arguments) explains
reading fields in ordinary CGP code.

:::

## The problem

The task is to greet a name that is known only when the program runs. A Hypershell program is a
type, so it can name a value it needs but cannot contain one, and the value has to come from
somewhere the program's interpreter can reach while it runs.

### Without CGP

A shell script reads the name from a variable, `echo "Hello, $name"`, and a Rust function takes it
as an argument. In a program that is interpreted, the usual answer is an environment passed to every
step of the interpreter, which each step reads from, so every step's signature carries the whole
environment whether it uses it or not. This page puts the value in a field of the context that runs
the program, and only the step that needs the field asks for it.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example hello_name
```

It needs only the `echo` command, and prints:

```text
Hello, Alice
```

## An argument list with a field in it

The program is `echo` with two arguments, from
[`hello_name.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/hello_name.rs):

```rust
use hypershell::namespaces::HypershellNamespace;
use hypershell::prelude::*;

pub type Program = hypershell! {
        SimpleExec<
            StaticArg<"echo">,
            WithArgs[
                StaticArg<"Hello,">,
                FieldArg<"name">,
            ],
        >
    |   StreamToStdout
};
```

[`hello`](./hello.md) used `WithStaticArgs`, which takes a list of literals. `WithArgs` takes a list
of argument *expressions*, so literals and other kinds of argument can be mixed.
`StaticArg<"Hello,">` is a literal, and `FieldArg<"name">` is the value of the context's field named
`name`, formatted with `Display`.

The arguments form a small language of their own inside the program. Each kind of argument is
interpreted through the context, so the same program reads its `name` from whichever context runs
it.

## A context of its own

`HypershellCli` has no fields, so this program runs on a context defined next to it:

```rust
#[derive(HasField)]
pub struct MyApp {
    pub name: String,
}

delegate_components! {
    MyApp {
        namespace HypershellNamespace;
    }
}
```

`MyApp` is a type that stands for this application, which is where its choices and its data live. It
has two parts. The wiring is the same one line `HypershellCli` has, so `MyApp` runs the same
language. The field is the value the program needs, and
[`#[derive(HasField)]`](/docs/reference/derives/derive_has_field) exposes it to generic code by
name, which is how `FieldArg<"name">` reaches it.

`main` builds the context with a value and runs the program on it:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = MyApp {
        name: "Alice".to_owned(),
    };

    app.handle(PhantomData::<Program>, Vec::new()).await?;

    Ok(())
}
```

The program reads the field through CGP's field access rather than through an ordinary method, and
it names the field itself. That is why a context needs no code beyond the derive: the program
decides which field it reads, and the context only has to have it.

## Try a change

Run the program on `HypershellCli` instead, which has no `name` field. The quickest way to see what
the compiler says is to check the pairing on its own, with
[`check_components!`](/docs/reference/macros/check_components), keyed on the program and its input:

```rust
use cgp::extra::handler::HandlerComponent;

check_components! {
    #[check_trait(CheckHypershellCli)]
    HypershellCli {
        HandlerComponent: (Program, Vec<u8>),
    }
}
```

[`cargo cgp check`](/docs/cargo-cgp/check) reports the failure with its root cause, shown here with
the long types abridged:

```text
error[E0277]: [CGP-E002] the provider trait `Handler<Pipe<…>, Vec<u8>>` with context `HypershellCli` is not implemented for provider `ComposeHandlers<…>`
   = note: root cause: [CGP-E106] missing field `name` on `HypershellCli`
```

The full output continues with the dependency chain that leads there, from the pipeline through
`SimpleExec`, the command's argument list, and the `FieldArg` that asked for the field. The fix is
the one `MyApp` already applies: run the program on a context that derives `HasField` and has the
field. `cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does
not yet reshape every class.

## The pattern

This program shows a **context supplying the values its implementations need**. The program and the
providers that interpret it state what they require, a field named `name`, without that requirement
appearing in how the program is run. Any context that joins the namespace and has the field can run
it. The idea is CGP's [impl-side dependencies](/docs/concepts/impl-side-dependencies), and reading a
context's fields in ordinary CGP code is covered by [implicit
arguments](/docs/concepts/implicit-arguments).

Hypershell uses `FieldArg` rather than an implicit argument because the field name is part of the
program: two programs run on one context can read different fields. The cost is that a missing field
is found at compile time only where the program is checked or run, which is why checking a program
against its context, as above, is worth doing.

## Where to go next

- [`http_checksum_cli`](./http-checksum-cli.md): the next example, a pipeline of three streaming
  commands.
- [Writing a program](../guides/writing-a-program.md): every kind of argument, and how to check a
  program.
- [Checking your wiring](/docs/concepts/check-traits): why CGP wiring is checked lazily, and what
  `check_components!` forces.
- [Hypershell: a type-level DSL for shell-scripting in
  Rust](/blog/hypershell-release#variable-parameters): the post that announced Hypershell, which
  introduced this program. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
