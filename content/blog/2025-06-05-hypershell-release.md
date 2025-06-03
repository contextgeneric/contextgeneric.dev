+++

title = "Hypershell: A Compile-Time DSL for Shell-Scripting in Rust"

description = ""

authors = ["Soares Chen"]

+++

## Summary

I am thrilled to introduce [_Hypershell_](https://github.com/contextgeneric/hypershell), a compile-time domain-specific language (DSL) for writing shell-script-like programs in Rust. Hypershell is powered by [_context-generic programming_](/) (CGP), making it highly modular and extensible.

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

## Macro Desugaring

The `hypershell!` macro is implemented as a simple proc macro the performs some basic syntax transformation to make Hypershell programs look slightly similar to shell scripts. However, it is completely optional, and we can also define Hypershell programs without using the `hypershell!` macro.

For instance, the earlier hello world program can be rewritten as follows:

```rust
pub type Program = Pipe<Product![
    SimpleExec<
        StaticArg<symbol!("echo")>,
        WithStaticArgs<Product![
            symbol!("hello"),
            symbol!("world!"),
        ]>,
    >,
    StreamToStdout,
]>;
```

Compared to the prettified version, the raw syntax for Hypershell is slightly more verbose, but is still relatively readable. The first thing to notice is that the handlers that are chained together with `|` are now placed inside a `Pipe` wrapper. Furthermore, the `Product!` macro from CGP is used to construct a variable-length list at the _type-level_, so that `Pipe` can accept arbitrary number of handlers.

We can also see that the syntax to `WithStaticArgs[...]` is desugared to `WithStaticArgs<Product![...]>`. With `hypershell!`, syntax that accept variable number of arguments can use the `[]` short hand to wrap the inner arguments around `Product!`. This leads to cleaner and more concise syntax and makes Hypershell programs more readable.

Finally, you may notice that all occurances of strings are wrapped inside the `symbol!` macro from CGP. This is because Hypershell programs are types, but string literals are value-level expressions. The `symbol!` macro can be used to turn string literals into _types_, so that we can now use them within type expressions.

Behind the scene, `symbol!` works similar to _const-generics_ in Rust. However, since Rust do not yet support the use of `String` or `&str` as const-generic arguments, the macro desugars the string literal into a type-level list of `char`, which can be used with const-generic.

With the three syntax transformations described, we can now better understand how the `hypershell!` macro works. In the DSL architecture for Hypershell, the `hypershell!` macro provides the _surface syntax_ of the DSL, which is desugared to Rust types that serve as the _abstract syntax_.

## Variable Parameters

Now that we have a better understanding of Hypershell, let's move on to a slightly more complex hello world example. Supposed that we want to run `echo` with a _variable_ argument `name`, so that the program would print "Hello", followed by the value stored in `name`. To do that, we would re-define our program as follows:

```rust
pub type Program = hypershell! {
        SimpleExec<
            StaticArg<"echo">,
            WithArgs [
                StaticArg<"Hello">,
                FieldArg<"name">,
            ],
        >
    |   StreamToStdout
};
```

In our new program, the second argument to `SimpleExec` is changed from `WithStaticArgs` to `WithArgs`. The main difference is that `WithStaticArg` accepts a list of static arguments, while `WithArgs` accepts a list of arguments with explicit specifiers.

In the first argument for `WithArgs`, we specify `StaticArg<"Hello">` to indicate that we always want to print the string `"Hello"` in the first argument. Following that, we specify `FieldArg<"name">` to indicate that we want to use the value of the `name` field from the context as the second argument.

Now that we have defined our program, a question that arise next is how can we "pass" in the `name` value to the program? Given that the program itself is only present at the type level, there is no place to hold the `name` value within the program. If we try to run the program using `HypershellCli`, we would encounter errors indicating that no `name` field is present inside the `HypershellCli` context.

### Custom Context

What we would do instead is to define a _new_ context type, so that we can use it to run our program. We would define a `MyApp` context with a `name` field as follows:

```rust
#[cgp_context(MyAppComponents: HypershellPreset)]
#[derive(HasField)]
pub struct MyApp {
    pub name: String,
}
```

The `MyApp` context is defined as a simple struct with a `name` field of type `String`. But it also comes with two attribute macros that are used to automatically derive capabilities for running Hypershell programs.

The first macro, `#[cgp_context]`, is used to enable wiring of CGP components to be used by the context. The first argument to the macro, `MyAppComponents`, is the name given to the _provider_ of the `MyApp` context. But since we don't include any additional wiring in this example, we can ignore it for now.

The macro argument is then followed by `:`, and then `HypershellPreset`, indicating that the `MyAppComponents` provider is _inherited_ from `HypershellPreset`, which is provided by Hypershell. The syntax looks similar to _supertraits_ in Rust, and works a bit like OOP inheritance, but operates at the _type-level_.

For the purpose of this example, the main takeaway is that the context `MyApp` implements all supported Hypershell traits through the provider `MyAppComponents`, with the component wiring inherited from `HypershellPreset`. We will revisit later on how CGP presets can defined and customized towards the end of this blog post.

The second macro, `#[derive(HasField)]`, is used to implement the `HasField` trait for `MyContext`. With this, the `name` field in `MyContext` is exposed as a `HasField` implementation, which can then be accessed by the implementation of `FieldArg<"name">`.

Now that we have defined our custom context, we can construct a value for it inside our `main` function, and call it with our program:

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

The full example is available in the [Hypershell repository](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/hello_name.rs). Since we initialize the `MyApp` context with the value `"Alice"` set in the `name` field, we should see `"Hello, Alice"` printed out when running the program:

```bash
$ cargo run --example hello_name
Hello, Alice
```

## Context-Generic Implementation

The example code earlier shows that our custom `MyApp` context implements everything that `HypershellCli` has implemented, with only two lines of macro code added to it. This is all made possible, because the core implementation of Hypershell is fully _context-generic_. That is, _none_ of the implementation code for Hypershell had direct access to `HypershellCli`, or `MyApp` for that matter.

Since the Hypershell core implementation do not have access to the concrete contexts, they are implemented to be _generic_ over any context type that satisfies given conditions. This makes the Hypershell implementation highly customizable and extensible. CGP makes it very _cheap_ to define custom contexts like `MyApp`, and eliminates tight coupling between implementations and concrete types.

As a side note, in case if you are wondering, the macro `#[cgp_context]` does _not_ generate Hypershell implementation code that work specifically with `MyApp`. Instead, if you expand the macro, you will only see a few lines of trait implementations that link to the `HypershellPreset` provider.

This puts CGP in stark contrast with alternative modular programming libraries in Rust, which tend to rely on heavy macro expansion to copy "template" code implementations to work with concrete types. Instead, CGP leverages traits, generics, and the Rust type system to ensure that all abstract implementations would always work reliability regardless of which concrete types they are instantiated with.

## Dependency Injection

With CGP, a key feature that we get for free from Rust is the ability to perform _dependency injection_ inside context-generic implementations. Even though Hypershell core implementation is generic over the context type, we can introduce additional trait bounds in the `impl` block to impose additional constraints on the context.

The implementation for `FieldArg<"name">` makes use of this feature, to require the generic context to contain a `name` field. With this, we can automatically use `FieldArg<"name">` with `MyApp`, since it is able to get the `name` value via the `HasField` instance. On the other hand, if we try to use it with `HypershellCli`, the implementation would fail as the `name` field could not be found there.

From this, we can also learn that the implementation wiring for CGP is done _lazily_. That is, both `HypershellCli` and `MyApp` are wired with the same abstract implementations from `HypershellPreset`, but only _some_ of the wirings are valid, depending on the additional capabilities provided by the concrete context.

It is worth noting that it has always been possible to use dependency injection through Rust trait impls, even in vanilla Rust. But with CGP, we bring the use of dependency injection to the next level, enabling use cases such as the implementation of `FieldArg` in this example.