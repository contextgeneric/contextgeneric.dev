---
sidebar_label: 'Writing a program'
sidebar_position: 1
description: 'How to write a Hypershell program: the prelude and a context, values from fields, simple and streaming stages, adapters, the input, and checks.'
---

# Writing a program

This guide is for writing Hypershell programs rather than extending the language: how to choose the
context, which is the type a program runs on, and then how to feed the program its values, make its
stages agree, and check it before running it. [Hypershell](../index.md) is a shell-scripting
language whose programs are Rust types, built with [CGP](/docs/). The
[examples](../examples/index.md) show each step in a running program.

## Start from the prelude and a context

**Import `hypershell::prelude::*`, and run the program on `HypershellCli` if it reads no runtime
values.** The prelude brings the syntax, the `hypershell!` macro and the names its output uses,
`CanHandle`, `PhantomData`, and the error type. Run the program with `handle`, inside a Tokio
runtime:

```rust
use hypershell::prelude::*;

pub type Program = hypershell! {
        SimpleExec<StaticArg<"echo">, WithStaticArgs["hello", "world!"]>
    |   StreamToStdout
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    HypershellCli.handle(PhantomData::<Program>, Vec::new()).await?;
    Ok(())
}
```

Import the prelude rather than the macro alone. The macro's output names `Pipe`, `Product!`, and
`Symbol!` without qualifying them, so a file without them in scope fails with "cannot find macro
`Product` in this scope". Use `HypershellHttp` instead of `HypershellCli` when the program makes
HTTP requests and reads no other value; it carries the `http_client` field those requests need.

## Read runtime values from a custom context

**A value a program needs at run time lives in a field of the context, and the program names the
field.** A program is a type, so it cannot hold a URL or a name. Define a struct with the fields,
derive [`HasField`](/docs/reference/derives/derive_has_field), and join `HypershellNamespace`:

```rust
use hypershell::namespaces::HypershellNamespace;
use hypershell::prelude::*;

#[derive(HasField)]
pub struct MyApp {
    pub http_client: reqwest::Client,
    pub url: String,
}

delegate_components! {
    MyApp {
        namespace HypershellNamespace;
    }
}
```

Name the client field `http_client`, since the namespace reads the HTTP client from a field of
exactly that name. Then reach the fields from the program with the argument syntax:

- **`FieldArg<"name">`** — one value, formatted with `Display`, wherever an argument is expected.
- **`FieldArgs<"args">`** — every item of an iterable field, as a command's whole argument list.
- **`JoinArgs[…]`** — several arguments joined into one. For a URL or a header it concatenates them,
  and for a command path or a file path it joins them as path segments.
- **`UrlEncodeArg<…>`** — a value percent-encoded for a URL. Use it inside the `JoinArgs` that
  builds the URL.

## Choose simple or streaming stages

**Use a simple stage when the data is small or a command must finish before the next one starts, and
a streaming stage for large data or concurrent processes.** `SimpleExec` and `SimpleHttpRequest`
collect the whole output and produce bytes. `StreamingExec` and `StreamingHttpRequest` produce a
stream at once, and consecutive streaming stages run at the same time, as in a shell.

The choice changes when a failure is reported. `SimpleExec` fails as soon as its command exits with
a failure status, reporting its standard error. `StreamingExec` reports the same failure when its
output stream ends, so the stage reading the stream fails with it, after any output the command
wrote has flowed through the stages in between.

## Make adjacent stages agree

**Each stage's output is the next stage's input, and a stage accepts only the input types its wiring
lists.** Most stages accept bytes or any kind of stream, so they chain freely. Three boundaries need
an adapter:

- **A streaming stage before a simple stage** — insert `StreamToBytes`, as in
  `StreamingExec<…> | StreamToBytes | SimpleExec<…>`.
- **An HTTP or WebSocket stream before `StreamToBytes` or `StreamToString`** — insert
  `ToTokioAsyncRead`, imported from `hypershell_tokio_components::dsl`.
- **Raw bytes to print as text** — `BytesToString` decodes UTF-8, and `BytesToHex` encodes bytes
  such as a digest.

The full table of what each stage accepts and produces is on the [streams and input
dispatch](../architecture/streams-and-input-dispatch.md#which-stages-accept-which-inputs) page.

## Pass the right input

**The input argument is the first stage's input, and its type must be one the first stage accepts.**
The common cases are:

- **`Vec::<u8>::new()`** for a program whose first stage runs a command or sends a request with an
  empty body.
- **`()`** for a program starting with `ReadFile`, which accepts nothing else.
- **A Rust value** for a program starting with `EncodeJson`.

When a program fails to compile, the errors reported at the `handle` call show some of their types
as unresolved `_`, whether the input is written `Vec::new()` or `Vec::<u8>::new()`. Checking the
program, as the last section shows, avoids them.

The program's output type is computed from its last stage, so `handle` on a program ending in
`DecodeJson<Vec<Issue>>` returns a `Vec<Issue>` with no conversion.

## Check the program before running it

**Assert that a context can run a program with
[`check_components!`](/docs/reference/macros/check_components), keyed on the program and its
input.** The check forces the whole resolution where it is written, and a failure there carries a
root cause:

```rust
use cgp::extra::handler::HandlerComponent;

check_components! {
    #[check_trait(CheckMyApp)]
    MyApp {
        HandlerComponent: (Program, Vec<u8>),
    }
}
```

One check can list several programs, or several inputs, as an array:
`HandlerComponent: [(Hello, Vec<u8>), (Cat, ())]`. Read failures with
[`cargo cgp check`](/docs/cargo-cgp/check); the [debugging guide](./debugging.md) shows the common
ones. A failure reported at the `handle` call itself often carries no root cause, which is the
practical reason to check. `cargo cgp check` leads with the root cause for the classes it
recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class.

## Build with nightly and the new trait solver

The Hypershell repository pins nightly Rust and passes `-Z next-solver=globally` to the compiler,
and a crate of your own whose programs nest sub-programs should do the same. On stable Rust the
simpler programs type-check, but the largest examples exhausted memory while type-checking; see
[crate layout](../architecture/crate-layout.md#what-the-workspace-needs-to-build).

## Where to go next

- [Debugging](./debugging.md): the common mistakes, with the error each one produces and its fix.
- [`hello_name`](../examples/hello-name.md): a custom context and a check, in a running program.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
