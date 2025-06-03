+++

title = "Hypershell: A Compile-Time DSL for Shell-Scripting in Rust"

description = ""

authors = ["Soares Chen"]

+++

## Summary

I am excited to introduce [_Hypershell_](https://github.com/contextgeneric/hypershell), a compile-time domain-specific language (DSL) for writing shell-script-like programs in Rust. Hypershell is powered by [_context-generic programming_ (CGP)](/), which enables it to be highly modular and extensible.

In this blog post, I will showcase some example Hypershell programs, and briefly explains how they are implemented using CGP. Towards the end, we will also discuss more about building DSLs in general, and how CGP can enable Rust to become a powerhouse for building new generation of domain specific languages.

## Disclaimer

Hypershell serves as an _experimental_, proof of concept, showcase of the capabilities of CGP. As such, its primary purpose is for demonstrating how CGP can be used to build highly modular DSLs in Rust.

The example use case of shell scripting is primarily chosen because it is fun and approachable to programmers of all background. In practice, we may or may not want to use Hypershell to write serious™ shell scripts. But regardless of the future outcome, I hope Hypershell can serve as a _fun_ programming example, and for you to becoming interested in learning CGP.

## Getting Started

You can use Hypershell today by simply adding the `hypershell` crate to your `Cargo.toml` dependencies. Since we will also cover the direct use of CGP, you should also add the `cgp` crate to your dependencies.

```toml
[dependencies]
cgp         = { version = "0.4.1" }
hypershell  = { version = "0.1.0" }
```

## Hello World

We will begin learning Hypershell with a simple hello world example. Our hello world program would run the CLI command `echo hello world!`, and then stream the output to STDOUT. With Hypershell, our program would be written as follows:

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

We first import everything from `hypershell::prelude` to use common Hypershell constructs. Our hello program is then defnined as a Rust _type_ named `Program`. In the body, we use the `hypershell!` macro to define our program with shell-like syntactic sugar, such as the use of pipe operator (`|`). At a high level, a Hypershell program is consist of one or more _handlers_ that forms a connected _pipeline_.

In the first part of the program, we use the `SimpleExec` handler to perform a simplified execution of a CLI command. The first argument to `SimpleExec` is `StaticArg<"echo">`, meaning that the program would always execute the hardcoded `echo` command. The second argument to `SimpleExec` is `WithStaticArgs`, which accepts a _variable_ list of static arguments that would be passed to the `echo` command.

In the second part of the program, we use the `|` operator to indicate that we want to pipe the result from `SimpleExec` to the next handler, `StreamToStdout`. The `StreamToStdout` handler would then stream the output to the STDOUT of the main Rust program, so that we can see the output when running the program.

Now that our program is defined, we can define a `main` function to call the Hypershell program inside our Rust program:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    HypershellCli
        .handle(PhantomData::<Program>, Vec::new())
        .await?;

    Ok(())
}
```

We use `#[tokio::main]` to define an async main function. Inside the function body, we make use of `HypershellCli`, which is a pre-defined context that can be used for running simple CLI-only Hypershell programs. The `HypershellCli` context is an empty struct, hence we are able to directly construct a value and call the `handle` method on it.

The `handle` method comes from the `CanHandle` trait from `cgp`, and is automatically implemented by `HypershellCli` for any suppported program. We pass our program to the first argument of `handle` as `PhantomData::<Program>`, that is, the `Program` we defined earlier is purely a _type-level_ construct and has no meaningful representation at the value-level. Nevertheless, we use `PhantomData` to "pass" the type as a value parameter, as it leads to cleaner syntax as compared to passing it as a generic argument.

We then pass an empty `Vec<u8>` to the second argument to `handle`, which would be used as the input of `SimpleExec` to be passed to the `STDIN` of the `echo` command.

The full example program shown here is also available at our [GitHub repository](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/hello.rs). If you clone the repository, you can run the example program with `cargo run`, and we should see the familiar `hello world!` printed out:

```bash
$ cargo run --example hello
hello world!
```