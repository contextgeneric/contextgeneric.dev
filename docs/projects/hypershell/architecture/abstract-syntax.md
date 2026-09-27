---
sidebar_label: 'Abstract syntax'
sidebar_position: 1
description: 'What a Hypershell program is made of: empty Rust structs that name what to do, the four kinds of syntax, and the macro that gives them a shell-like look.'
---

# Abstract syntax

Why would anyone write a program as a Rust type made of empty structs? Because a program that
contains no code can be given its meaning by something else, and changed without being edited.
[Hypershell](../index.md) is a small shell-scripting language built with [CGP](/docs/). This page
shows what a Hypershell program is made of, the kinds of pieces it can contain, and the macro that
lets it be written like a shell pipeline.

## A piece of syntax is an empty struct

Every piece of a Hypershell program is a struct with no fields and no methods. Its type parameters
hold the parts nested inside it, stored in `PhantomData` so the struct stays empty:

```rust
pub struct SimpleExec<Path, Args>(pub PhantomData<(Path, Args)>);

pub struct StreamToStdout;
```

`SimpleExec<Path, Args>` means "run the command `Path` with the arguments `Args`", and `Path` and
`Args` are themselves pieces of syntax. So a whole program is one large nested type, and nothing in
it says *how* to run a command or print a stream. That is left to the providers the running context
chooses, as the [overview](./index.md) explains. Replace the provider for `SimpleExec`, and every
program that uses `SimpleExec` behaves differently, without any program being edited.

A program is also never created as a value. It is passed to a context as `PhantomData::<Program>`, a
zero-sized marker that carries only the type. The only data that flows at run time is the program's
input, such as the bytes fed to the first command.

## Four kinds of syntax

Hypershell's syntax falls into four kinds. They nest inside each other, and each kind is interpreted
by its own kind of provider:

| Kind | What it does | Examples |
|---|---|---|
| **Stages** | take an input and produce an output, like the commands in a shell pipeline | `SimpleExec`, `StreamingExec`, `SimpleHttpRequest`, `ReadFile`, `WriteFile`, `EncodeJson`, `StreamToStdout` |
| **Arguments** | produce one value for a stage: a string, a path, a URL, or an HTTP method | `StaticArg<"echo">`, `FieldArg<"url">`, `JoinArgs[…]`, `UrlEncodeArg<…>`, `GetMethod` |
| **Argument lists** | configure a command or a request | `WithArgs[…]`, `WithStaticArgs[…]`, `WithHeaders[…]` |
| **Control** | combine other programs | `Pipe`, which the `\|` builds, and `Use`, `ConvertTo`, `Box` |

The arguments are worth a closer look, because they are how a program gets values it cannot contain.
A literal such as a command name is a `StaticArg`. A value known only at run time, such as a URL,
lives in a field of the context, and `FieldArg<"url">` reads it; see
[`hello_name`](../examples/hello-name.md). Arguments nest, so `UrlEncodeArg<FieldArg<"org">>` reads
a field and encodes it for use in a URL.

Some syntax exists for the language's own use rather than for programs. Both command stages, for
instance, start their process through one shared internal step, which is written as syntax too, so
that a single provider decides how every command is started.

## The macro only changes the notation

Writing nested types by hand is tiring, so the `hypershell!` macro accepts a shell-like notation and
rewrites it into the type. It follows three rules: `|` joins stages into a pipeline, a bracketed
list after a name becomes a type-level list, and a string literal becomes a type-level string. So
this program

```rust
pub type Program = hypershell! {
        SimpleExec<
            StaticArg<"echo">,
            WithStaticArgs["hello", "world!"],
        >
    |   StreamToStdout
};
```

is exactly this type, which can be written without the macro:

```rust
pub type Program = Pipe<Product![
    SimpleExec<
        StaticArg<Symbol!("echo")>,
        WithStaticArgs<Product![Symbol!("hello"), Symbol!("world!")]>,
    >,
    StreamToStdout,
]>;
```

[`Product!`](/docs/reference/macros/product) and [`Symbol!`](/docs/reference/macros/symbol) are
CGP's type-level list and string. The macro knows nothing about what `SimpleExec` means, which is
deliberate: the language works without the macro, and new syntax can be added without changing it.

## What it costs

A program that is a type is checked and fixed at compile time. That is the benefit, since a
mismatched pipeline is a compile error rather than a failure halfway through a run, and it is also
the limit, since a program cannot be read from a file or changed while an application runs. Programs
that must change at run time are better expressed as data, run by a shell or by a plain Rust
function.

Large nested types also cost compile time, and when something is wrong the compiler reports the
whole type, so error messages are long. The macro's output also needs `hypershell::prelude::*` in
scope, which every example imports.

## Where to go next

- [Assembly](./assembly.md): how a context gets a provider for every piece of syntax.
- [`hello`](../examples/hello.md): the program above, run.
- [Type-level DSLs](/docs/concepts/type-level-dsls): the same technique with a smaller language.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
