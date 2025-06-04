+++

title = "Hypershell: A Compile-Time DSL for Shell-Scripting in Rust"

description = ""

authors = ["Soares Chen"]

+++

# Summary

I am thrilled to introduce [_Hypershell_](https://github.com/contextgeneric/hypershell), a compile-time domain-specific language (DSL) for writing shell-script-like programs in Rust. Hypershell is powered by [_context-generic programming_](/) (CGP), making it highly modular and extensible.

In this blog post, I will showcase some example Hypershell programs, and briefly explains how they are implemented using CGP. Towards the end, we will also discuss more about building DSLs in general, and how CGP can enable Rust to become a powerhouse for building new generation of domain specific languages.

## Disclaimer

Hypershell serves as an _experimental_, proof of concept, showcase of the capabilities of CGP. As such, its primary purpose is for demonstrating how CGP can be used to build highly modular DSLs in Rust.

The example use case of shell scripting is primarily chosen because it is fun and approachable to programmers of all background. In practice, we may or may not want to use Hypershell to write serious™ shell scripts. But regardless of the future outcome, I hope Hypershell can serve as a _fun_ programming example, and for you to becoming interested in learning CGP.

# An Overview of Hypershell

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

We can also see that the syntax `WithStaticArgs[...]` is desugared to `WithStaticArgs<Product![...]>`. With `hypershell!`, syntax that accept variable number of arguments can use the `[]` short hand to wrap the inner arguments around `Product!`. This leads to cleaner and more concise syntax and makes Hypershell programs more readable.

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

The macro argument is then followed by `:`, and then `HypershellPreset`, indicating that the `MyAppComponents` provider is _inherited_ from `HypershellPreset`, which is provided by Hypershell. The syntax looks similar to _supertraits_ in Rust, and works a bit like OOP prototypal inheritance, but operates only during compile-time at the _type-level_.

For the purpose of this example, the main takeaway is that the context `MyApp` implements all supported Hypershell components through the provider `MyAppComponents`, with the component wiring inherited from `HypershellPreset`. We will revisit later on how CGP presets can defined and customized towards the end of this blog post.

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

## Streaming Handlers

Now that we have gone through the basics of Hypershell, let's try defining more complex Hypershell programs. In the earlier example, we performed CLI execution using `SimpleExec`, which accepts inputs and produce outputs as raw bytes, i.e. `Vec<u8>`. This mode of execution gives simpler semantics to the execution, as we don't need to worry about cases when an `STDIN` or `STDOUT` stream being closed prematurely.

However, one of the appeal of shell scripting is the ability to _stream_ the `STDOUT` of one program into the `STDIN` of another program, with both programs running in parallel. To support this execution, Hypershell also provides `StreamingExec`, which spawns the child process in the background and handles inputs and outputs as _streams_. Hypershell currently supports 3 kinds of streams: [`futures::Stream`](https://docs.rs/futures/latest/futures/prelude/trait.Stream.html), [`futures::AsyncRead`](https://docs.rs/futures/latest/futures/io/trait.AsyncRead.html), and [`tokio::io::AsyncRead`](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html). As we will see later, the modular design of Hypershell also makes it easy to build extended implementations to support other kinds of streams.

To demonstrate streaming execution in action, we will define an example program that streams the HTTP response from a URL, and compute the SHA256 checksum of the web page. The program would be defined as follows:

```rust
pub type Program = hypershell! {
    StreamingExec<
        StaticArg<"curl">,
        WithArgs [
            FieldArg<"url">,
        ],
    >
    |   StreamingExec<
            StaticArg<"sha256sum">,
            WithStaticArgs [],
        >
    |   StreamingExec<
            StaticArg<"cut">,
            WithStaticArgs [
                "-d",
                " ",
                "-f",
                "1",
            ],
        >
    | StreamToStdout
};
```

To put it simply, the above Hypershell program is roughly equivalent to the following bash command:

```rust
curl $url | sha256sum | cut -d ' ' -f 1
```

The first handler uses `curl` to fetch the HTTP response from a `url` value provided by the context. The second handler uses `sha256sum` to perform streaming computation of the checksum. The third handler uses `cut` to get the checksum value produced by `sha256sum`, ignoring the file name output in the second column.

As with the earlier example, we define a new `MyApp` context to provide the `url` value:

```rust
#[cgp_context(MyAppComponents: HypershellPreset)]
#[derive(HasField)]
pub struct MyApp {
    pub url: String,
}
```

We can then call the program with `MyApp` in our main function:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = MyApp {
        url: "https://nixos.org/manual/nixpkgs/unstable/".to_owned(),
    };

    app.handle(PhantomData::<Program>, Vec::new()).await?;

    Ok(())
}
```

For our example run, we want to choose a public web page that is relatively large for the effect of streaming to be more noticeable. So we pick the [Nix manual](https://nixos.org/manual/nixpkgs/unstable/) as the `url` value to construct `MyApp`.

The example program is also available at the [Hypershell repository](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/http_checksum_cli.rs). If we run it, we would see the checksum produced such as follows:

```bash
$ cargo run --example http_checksum_cli
c5ce4ff8fb2d768d4cbba8f5bee3d910c527deedec063a0aa436f4ae7005c713
```

Feel free to tweak the example with different CLI commands, to better observe that Hypershell is indeed streaming the handler inputs/outputs in parallel.

## Native HTTP Request

In the earlier example, we performed HTTP requests `curl` before streaming it to the `sha256sum` command. But since we are running the program inside Rust, a natural evolution would be to use native HTTP clients in Rust to submit the HTTP request.

Hypershell provides native HTTP support with the implementation being a separate _extension_ on top of the base CLI implementation. The handlers `SimpleHttpRequest` and `StreamingHttpRequest` are provided as the HTTP equivalent of `SimpleExec` and `StreamingExec`.

We can modify our earlier example to use `StreamingHttpRequest` in place of `curl` as follows:

```rust
pub type Program = hypershell! {
    StreamingHttpRequest<
        GetMethod,
        FieldArg<"url">,
        WithHeaders[ ],
    >
    |   StreamingExec<
            StaticArg<"sha256sum">,
            WithStaticArgs [],
        >
    |   StreamingExec<
            StaticArg<"cut">,
            WithStaticArgs [
                "-d",
                " ",
                "-f",
                "1",
            ],
        >
    | StreamToStdout
};
```

The `StreamingHttpRequest` handler accepts 3 arguments. The first argument, `GetMethod`, is used to indicate that we want to send with the GET method. The second argument, `FieldArg<"url">`, indicates that we want to send the HTTP request to the URL specified in the `url` field of the context. The third argument, `WithHeaders[ ]`, allows us to specify the HTTP headers, which is left empty for this example.

As we can see, Hypershell allows streaming pipelines to be built seamlessly between native handlers and CLI handlers. In fact, all handlers are just CGP components that implement the `Handler` interface. So we can easily extend the DSL with new handler implementations that can interop with each others as long as the Rust types between the inputs and outputs matches.

Behind the scene, Hypershell implements the native HTTP client using [`reqwest`](https://docs.rs/reqwest/). To run the program, the context needs to provide a `http` field with a [`reqwest::Client`](https://docs.rs/reqwest/latest/reqwest/struct.Client.html). Together with the `url` field, we would define a `MyApp` context as follows:

```rust
#[cgp_context(MyAppComponents: HypershellPreset)]
#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub url: String,
}
```

We can then construct a `MyApp` context in our main function, and then call the Hypershell program:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = MyApp {
        http_client: Client::new(),
        url: "https://nixos.org/manual/nixpkgs/unstable/".to_owned(),
    };

    app.handle(PhantomData::<Program>, Vec::new()).await?;
    Ok(())
}
```

The same example is available at the [project repository](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/http_checksum_client.rs). Running it should produce the same HTTP checksum as before:

```bash
$ cargo run --example http_checksum_client
c5ce4ff8fb2d768d4cbba8f5bee3d910c527deedec063a0aa436f4ae7005c713
```

It is worth noting that aside from `reqwest`, it is possible to customize a context to use alternative HTTP client implementations to handle `SimpleHttpRequest` and `StreamingHttpRequest`. In such cases, we would be able to define contexts without the `http_client` field, if it is not required by the alternative implementation.

## JSON Encoding

As an embedded DSL, Hypershell programs have the ability to make full use of Rust to seamlessly integrate shell scripting with the remaining parts of the application. A good example to demonstrate this is to encode/decode native Rust types as part of the pipelines of a Hypershell program.

Following is an example Hypershell program that submits a Rust code snippet to [Rust Playground](https://play.rust-lang.org/), and publish it as a GitHub Gist:

```rust
pub type Program = hypershell! {
    EncodeJson
    |   SimpleHttpRequest<
            PostMethod,
            StaticArg<"https://play.rust-lang.org/meta/gist">,
            WithHeaders [
                Header<
                    StaticArg<"Content-Type">,
                    StaticArg<"application/json">,
                >
            ],
        >
    |   DecodeJson<Response>
};
```

The `EncodeJson` handler accepts any input that implements `Serialize`, and encode them into JSON bytes as the output. Next, we use `SimpleHttpRequest` to submit the HTTP request, as there is no need of streaming with small payloads. Inside `WithHeaders`, we also use `Header` to set the `Content-Type` header to `application/json`. Finally, the `DecodeJson` handler decodes its input bytes into the specified Rust type, which is expected to implement `Deserialize`.

We would then define the input and output types as follows:

```rust
#[derive(Serialize)]
pub struct Request {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub id: String,
    pub url: String,
    pub code: String,
}
```

The `Request` and `Response` types are defined with the respective `Serialize` and `Deserialize` implementations, following the formats expected by Rust playground.

With the program defined, we can now programmatically submit a code snippet to Rust Playground in our `main` function:

```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = HypershellHttp {
        http_client: Client::new(),
    };

    let input = Request {
        code: "fn main() { println!(\"Hello, world!\"); }".to_owned(),
    };

    let output = app.handle(PhantomData::<Program>, input).await?;

    println!("Created new Rust playground gist with response: {output:#?}");

    Ok(())
}
```

When no additional field is required, Hypershell provides the pre-defined `HypershellHttp` context that can be used to run Hypershell programs with HTTP capabilities. The example code is also available at the [project repository](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/rust_playground.rs). Running it should produce an output such as follows:

```bash
$ cargo run --example rust_playground
Created new Rust playground gist with response: Response {
    id: "ec90cbb6b3e797b15dd1eacbd51ffa8b",
    url: "https://gist.github.com/rust-play/ec90cbb6b3e797b15dd1eacbd51ffa8b",
    code: "fn main() { println!(\"Hello, world!\"); }",
}
```

# Implementation Details

By now, hopefully the earlier examples are enough to convince you that the base implementation of Hypershell is pretty powerful, and can be potentially useful to build real-world™ applications.

Now that I have pique your interest, hopefully it would also gave you sufficient motivation to learn about _how_ Hypershell is implemented, and how you can make use of CGP to build other domain-specific languages in similar ways as Hypershell.

## Context-Generic Programming

At its core, the modular implementation of Hypershell is made possible through _context-generic programming_ (CGP), a modular programming paradigm for Rust. A full introduction for CGP can be found on the [current website](/) that hosts this blog post. But for readers who are new here, I will try to give a short introduction to CGP in this section.

As its name implies, CGP allows Hypershell to implement its core logic to be generic over any context, e.g. `HypershellCli`, `HypershellHttp`, `MyApp`. This means that every time we define a new concrete context, we can choose reuse _all_, or more importantly, _some_ of Hypershell's core implementation based on the needs of our application. Furthermore, external developers can also write their own context-generic implementations in the same way, which can be used to _replace_ or _extend_ the existing core implementation.

At a high level, CGP makes it possible to bypass the _coherence_ restrictions in Rust traits, allowing us to define overlapping or orphan trait implementations. Everything else in CGP is built on the foundation of asking: what would Rust programs look like if there were no coherence restrictions? CGP works on _safe_, _stable_ version of Rust today, and all you have to do is include the [`cgp`](https://crates.io/crates/cgp) crate as a dependency.

The basic idea of how CGP workaround coherence is pretty simple. We first start with an example `CanGreet` trait, that is implemented with CGP as follows:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}
```

The `CanGreet` trait that we defined is a classical Rust trait, which we now call a _consumer trait_ in CGP. With the `#[cgp_component]` macro, a _provider trait_ and a _name_ struct are also generated as follows:

```rust
pub trait Greeter<Context> {
    fn greet(context: &Context);
}

pub struct GreeterComponent;
```

Compared to the consumer trait `CanGreet`, the provider trait `Greeter` has an additional generic `Context` parameter to refer to the original `Self` type in `CanGreet`. Similarly, all occurances of `Self`, i.e. `&self`, are replaced with the explicit `Context`, i.e. `context: &Context`.

In CGP, each implementation of the provider trait `Greeter` would choose a _unique type_ for `Self`, i.e. by defining a dummy struct like `struct Provider;`, and then implementing the provider trait for that struct. The dummy struct that implements the provider trait is called a _provider_. Because the coherence restriction only applies mainly to the `Self` type, by choosing a unique `Self` type for each implementation, we essentially bypass the coherence restrictions, and can define multiple generic implementations that are overlapping over the `Context` type.

The macro also generates a `GreeterComponent` struct, which is used as a _name_, or a _key_, for the underlying implementation to perform compile-time _look up_ when instantiating the implementation of the consumer trait from a provider trait implementation. We will revisit this in a moment.

To demonstrate, we show two example provider implementations for `Greeter` as follows:

```rust
#[cgp_new_provider]
impl<Context> Greeter<Context> for GreetHello {
    fn greet(context: &Context) {
        println!("Hello!");
    }
}

#[cgp_new_provider]
impl<Context> Greeter<Context> for GreetBonjour {
    fn greet(context: &Context) {
        println!("Bonjour!");
    }
}
```

The `#[cgp_new_provider]` macro automatically define new structs for `GreetHello` and `GreetBonjour`. As we can see, both implementations are generic over the `Context` type, but there is no error arise from overlapping instances.

### Components Wiring

Although multiple overlapping provider trait implementations can co-exist, they do not implement the original consumer trait `CanGreet`. To implement the consumer trait `CanGreet` for a specific concrete context, additional _wiring steps_ are needed to select _which_ provider implementation should be used for that concrete context.

To demonstrate how the wiring works, we will define an example `MyApp` context as follows:

```rust
#[cgp_context(MyAppComponents)]
pub struct MyApp;

delegate_components! {
    MyAppComponents {
        GreeterComponent: GreetHello,
    }
}
```

In the example above, we define a concrete `MyApp` context using the `#[cgp_context]` macro, which generates a new `MyAppComponents` struct and wire it with the context. Following that, we use `delegate_components!` to essentially use `MyAppComponents` as a _type-level lookup table_, containing one entry with `GreeterComponent` as the "key", and `GreetHello` as the "value".

With the wiring in place, the concrete context `MyApp` now automatically implements `CanGreet`, and we can now call `MyApp.greet()`. To understand how the magic works, we can visualize the underlying implementation with the following diagram:

![Diagram](/blog/images/cgp-wiring.png)

We start from the lower left corner, with the goal to implement `CanGreet` for `MyApp`. First of all, Rust trait system would notice that `MyApp` does not have an explicit implementation for `CanGreet`, but has a `HasProvider` implementation generated by `#[cgp_context]`, which points to `MyAppComponents`.

Next, the trait system sees that `MyAppComponents` does not directly implement `Greeter<MyApp>`. So the system performs a type-level lookup on the `GreeterComponent` key stored in `MyAppComponents`. The lookup is implemented through the trait `DelegateComponent<GreeterComponent>`, which were generated by the `delegate_components!` macro. There, it sees that an entry for `GreeterComponent` is found, which points to `GreetHello`.

Following that, the trait system sees that `GreetHello` has a valid implementation of `Greeter<MyApp>`. Through that, it generates a blanket implementation of `Greeter<MyApp>` for MyAppComponents, which simply forwards the call to `GreetHello`.

Similarly, now that `Greeter<MyApp>` is implemented for `MyAppComponents`, the trait system generates a blanket implementation of `CanGreet` for `MyApp`. The blanket implementation forwards the call to the `Greeter<MyApp>` implementation of `MyAppComponents`, which is then forwarded to `GreetHello`.

The blanket implementations for `CanGreet` and `Greeter` were generated by `#[cgp_components]` when the consumer trait is defined. What we have described earlier are the high level visualizations on how these blanket implementations work under the hood.

### Prototypal Inheritance

For readers who are familiar with JavaScript, you might notice that the wiring mechanics for CGP is very similar to how _prototypal inheritance_ works in JavaScript. Conceptually, the greet example earlier works similar to the following JavaScript code:

```javascript
// provider
function greet_hello() {
    console.log("Hello!")
}

// lookup table
var MyAppComponents = {
    greet: greet_hello,
}

// concrete context
var MyApp = function() {}
MyApp.prototype = MyAppComponents

// context value
var app = new MyApp()
app.greet()
```

Since JavaScript is dynamic-typed, the concept of a trait or interface cannot be specified in the code. However, we can still conceptually think that a `CanGreet` interface exist with certain requirements on the method. The function `greet_hello` is the equivalent of a _provider_ that implements the `Greeter` interface.

Similarly, `MyAppComponents` serves as a lookup table that maps the `greet` method to the `greet_hello` provider. We then define the `MyApp` context class, and set `MyAppComponents` as the _prototype_ of `MyApp`. This works similar to the `HasProvider` trait from CGP to link the consumer trait implementation the provider trait.

Finally, we can instantiate an instance of `MyApp` using the `new` keyword, and there we can also find that the `app.greet()` method can be called.

If we try to visualize the wiring of prototypes in our JavaScript example, we would get a similar diagram as what we have for CGP:

![Diagram](/blog/images/prototypal-inheritance.png)

We would navigate the implementation diagram from the top-left corner. In order for `app.greet()` to be implemented, its class `MyApp` needs to have a `prototype` field, which points to `MyAppComponents`. We then perform lookup on the `greet` key, and find the provider function `greet_hello` to be called.

During runtime, the prototype `MyAppComponents` is attached to `app.__proto__`, which in turn enables `app.greet()` to be called.

### Comparison to OOP

Although CGP shares similarity with OOP, particularly with prototype-based programming, there are major differences with how it is implemented that makes CGP far better than existing systems.

In particular, the strong type system provided by Rust, in addition to advanced features such as generics and traits, makes it possible to write highly advanced constructs that are otherwise not possible with OOP alone. Furthermore, the prototype-like lookup is performed by CGP at _compile-time_, and thus eliminates runtime overheads such as virtual tables and JIT compilation.

In a way, we can think of CGP only taking the good parts from OOP, and enhance it by making it work seamlessly with the advanced type system in Rust.

## Abstract Syntax