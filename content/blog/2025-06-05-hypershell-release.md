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
    |   StreamToStdout
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

## Conclusion

By now, hopefully the earlier examples are enough to convince you that the base implementation of Hypershell is pretty powerful, and can be potentially useful to build real-world™ applications.

Now that I have pique your interest, hopefully it would also gave you sufficient motivation to learn about _how_ Hypershell is implemented, and how you can make use of CGP to build other domain-specific languages in similar ways as Hypershell.

# Context-Generic Programming

At its core, the modular implementation of Hypershell is made possible through _context-generic programming_ (CGP), a modular programming paradigm for Rust. A full introduction for CGP can be found on the [current website](/) that hosts this blog post. But for readers who are new here, I will try to give a short introduction to CGP in this section.

As its name implies, CGP allows Hypershell to implement its core logic to be generic over any context, e.g. `HypershellCli`, `HypershellHttp`, `MyApp`. This means that every time we define a new concrete context, we can choose reuse _all_, or more importantly, _some_ of Hypershell's core implementation based on the needs of our application. Furthermore, external developers can also write their own context-generic implementations in the same way, which can be used to _replace_ or _extend_ the existing core implementation.

At a high level, CGP makes it possible to bypass the _coherence_ restrictions in Rust traits, allowing us to define overlapping or orphan trait implementations. Everything else in CGP is built on the foundation of asking: what would Rust programs look like if there were no coherence restrictions? CGP works on _safe_, _stable_ version of Rust today, and all you have to do is include the [`cgp`](https://crates.io/crates/cgp) crate as a dependency.

## Consumer and Provider Traits

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

## Components Wiring

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

## Prototypal Inheritance

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

## Comparison to OOP

Although CGP shares similarity with OOP, particularly with prototype-based programming, there are major differences with how it is implemented that makes CGP far better than existing systems.

In particular, the strong type system provided by Rust, in addition to advanced features such as generics and traits, makes it possible to write highly advanced constructs that are otherwise not possible with OOP alone. Furthermore, the prototype-like lookup is performed by CGP at _compile-time_, and thus eliminates runtime overheads such as virtual tables and JIT compilation.

Through this comparison, I also hope to highlight that CGP is _not_ a totally novel concept that is impossible to understand. There are many more articles available that explains in depth how prototypal inheritance works, but there are virtually no third party articles that explain how CGP works. My hope is that the similarity comparison can help readers coming from OOP background to better understand how CGP works, by reusing existing concepts that you may have learned before.

## Learn More

We have now finished our quick introduction to CGP. So far, we have only explored CGP at very high level, with little technical details or code exploration to dive into how CGP actually works behind the scene.

If you would like to learn more about how CGP works, I recommend reading the in-progress book [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/), which walks through all the programming techniques used to build CGP from the ground up. Note, however, that if you _don't_ care about the internal details, and just want to quickly get started with programming _with_ CGP, then you might want to skip reading the book, at least for now.

Unfortunately, we don't yet have good tutorials available for readers to quickly get started on writing toy programs with CGP. This is partly because there is currently not sufficient motivation for readers to go through such tutorials. CGP is designed to solve the problems of highly complex applications with many cross-cutting concerns, and the benefits would only become apparent after writing at least 5,000 to 10,000 lines of code. As a result, if a tutorial shows only few hundreds lines of example code, some readers would inevitably feel confused of _why_ they should learn writing the example code with CGP, instead of using vanilla Rust patterns that they are already familiar with.

Instead, the current priority for the CGP project is to make use of the _full power_ of CGP to build powerful DSL frameworks, such as Hypershell, that demonstrate the full potential of CGP. The purpose is to show readers that CGP is demonstrably useful, at least in the showcased domains, and give them a reason to start _learning_ CGP.

A consequence for this strategy is that many advanced CGP patterns are introduced _all at once_, as will be covered in the next section. At lot of the advanced CGP patterns are not yet covered in the book, and there is currently no other documentation other than this blog post that talks about these patterns. As a result, if you are totally new to CGP, or if you are just starting to learn the basic concepts, it can potentially be overwhelming and confusing to continue reading to the next section.

Nevertheless, I will try to stay as high level as possible when explaining the advanced CGP concepts, and omit the internal details similar to how the earlier explanation for CGP wiring is done. So I hope you would bear with me for now, and let's walk through together on how Hypershell is implemented with CGP.

# Implementation of Hypershell

Now that we have equiped with some brief understanding of CGP, let's take a look at how the Hypershell DSL is implemented using CGP. The programming techniques that we are going to cover would not only work for Hypershell, but also more generally any kind of DSL.

The main idea is that the programs for this family of DSLs would be written as _types_ that will be "interpreted" at compile time. The main strength of this approach is that the DSL can leverage the Rust compiler and zero-cost abstraction and be very performant. The main disadvantage is that the program for the DSL must be available at the same time as the Rust program is built. In other words, it is less suitable in scripting applications that require dynamic loading of programs, such as web browsers or plugin systems, unless the system also bundles the full Rust compiler to compile the DSL program.

Nevertheless, this section will be especially useful for readers who are interested in building DSLs similar to Hypershell. For the remaining readers, I hope that the section would still be useful for you to understand more about CGP, and consider using it to build other kinds of modular applications.

## Handler Component

The core component behind Hypershell is the `Handler` component, which is implemented by the handlers in a Hypershell pipeline. The consumer trait for the component, `CanHandle` is defined as follows:

```rust
#[cgp_component(Handler)]
pub trait CanHandle<Code: Send, Input: Send>: HasAsyncErrorType {
    type Output: Send;

    async fn handle(
        &self,
        _tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Self::Output, Self::Error>;
}
```

The `CanHandle` trait is parameterized by two generic parameters, `Code` and `Input`. The `Code` type represents the DSL program that we want to "run" or "interpret". The `Input` is the main input data to be passed to the program, such as for the STDIN or the HTTP request body. The generic types have additional `Send` bound with them, as CGP requires async functions to implement `Send` by dfault, to allow them to be used in spawned tasks such as `tokio::spawn`.

The trait also has an associated type `Output`, which represents the output type produced by the program, such as STDOUT or the HTTP response body. Being associated type, it means that for each combination of `Code` and `Input` parameters, there is a _unique_ `Output` type that is associated to that.

The `handle` method is an async function with `&self` as the first argument, which means in addition to the `Input`, the handler also has access to the context that contains additional dependencies and the surrounding environment. The second parameter, `_tag`, has the type `PhantomData<Code>`, and is used to pass the `Code` program as a value to assist in type inference. Other than that, the value is expected to be ignored by the method body, since there is no runtime information carried by `PhantomData`.

The `handle` method returns a `Result`, with `Self::Output` being the success result, and `Self::Error` if there is any error. `Self::Error` is an _abstract type_ defined by the `ErrorTypeProvider` component, which is defined in CGP as follows:

```rust
#[cgp_type]
pub trait HasErrorType {
    type Error: Debug;
}
```

First, `HasErrorType` is a consumer trait that contains an associated `Error` type, which is required to always implement `Debug`. The macro `#[cgp_type]` is an extension to `#[cgp_component]`, and is used to define abstract type component with some additional derivations. The macro also generates an `ErrorTypeProvider` provider trait.

To support the async method in `CanHandler`, the context and the `Error` type also needs to implement `Send`, which is provided by `HasAsyncErrorType` as a _trait alias_:

```rust
#[blanket_trait]
pub trait HasAsyncErrorType:
    Send + Sync + HasErrorType<Error: Send + Sync>
{}
```

The `HasAsyncErrorType` trait is automatically implemented for all `Context` type that implements `HasErrorType`, with the additional constraints that `Context: Send + Sync` and `Context::Error: Send + Sync`. This would ensure that the `Future` returned from async functions that capture `Context` or `Context::Error` will always implement `Send`.

The `#[blanket_trait]` macro is provided by CGP to help with shortening definitions of trait aliases. Behind the scene, it generates a trivial blanket implementation for `HasAsyncErrorType` that is implemented if all supertrait constraints are satisfied.

Going back to `CanHandle`, the `#[cgp_component]` macro also generates the provider trait `Handler` as follows:

```rust
pub trait Handler<Context, Code: Send, Input: Send>
where
    Context: HasAsyncErrorType,
{
    type Output: Send;

    async fn handle(
        context: &Context,
        _tag: PhantomData<Code>,
        input: Input,
    ) -> Result<Self::Output, Context::Error>;
}
```

As we can see, the main difference between `Handler` and `CanHandle` is that the `Self` type is replaced with an explicit `Context` parameter. The supertrait `HasAsyncErrorType` now becomes a trait bound for `Context`.

## Abstract Syntax

Now that we know about the interface for the handler component, let's take a look at how the `Handler` trait is implemented for a basic Hypershell command, `SimpleExec`. Recall from earlier that `SimpleExec` allows execution of shell command, with plain raw bytes as the input and output.

If we try to navigate to the place that defines `SimpleExec`, all we see is the following definition:

```rust
pub struct SimpleExec<CommandPath, Args>(pub PhantomData<(CommandPath, Args)>);
```

_Wait, what?_ That's it? Yes, you see it right. There is no extra trait implementation that is directly tied to `SimpleExec`. In fact, all types that are used to "write" a Hypershell program are just dummy structs like the one defined for `SimpleExec`.

**This implies that how a Hypershell program is "written" is completely _decoupled_ from how the program is "interpreted" or "executed" by the concrete context.**

In other words, when we walked through our examples earlier, the use of `HypershellCli`, `HypershellHttp`, or `MyApp` are only few of the possible _choices_ that we can choose to run our Hypershell programs. More generally, since all the contexts so far only inherit from `HypershellPreset`, it implies that one can also build fully customized presets with different ways to run the programs, such as changing the way how `SimpleExec` should run.

More formally, we can say that a type like `SimpleExec` is used to represent the _abstract syntax_ of the Hypershell DSL. We then make use of CGP and Rust's trait system to act as the "interpreter" for the DSL, to "dispatch" the handling of a program fragment to a specific CGP provider. When we define custom contexts, we are essentially building custom "interpreters" that are used for "executing" the Hypershell program at compile-time.

It is also worth noting that the pattern introduced here is a highly advanced CGP programming technique. There are also simpler versions of the pattern, such as _higher order providers_, where traits like `Handler` would not contain the `Code` parameter, and types like `SimpleExec` would directly implement the `Handler` trait. In this simplified pattern, the execution of the program would be tightly coupled with a specific implementation, making it less modular.

Both the higher order provider and DSL patterns are advanced CGP patterns that are not yet covered by the CGP patterns book. Such advanced techniques can sometimes be overkill for building simple applications, especially for beginners who just want to try out CGP to make their applications _slightly_ more modular. However, they are perfect for building DSLs, as it is a good practice to separate the _syntax_ from the _semantics_ of proramming languages.

## Handler Implementation for `SimpleExec`

For most of you who are new to CGP, at this point you would probably be completely loss on how to navigate your way to figure out _where_ is the actual implementation for `SimpleExec`. We will revisit the topic of how the wiring is actually done slightly later on. For now, let's go straight to the default provider used by Hypershell to implemented `SimpleExec`:

```rust
#[cgp_new_provider]
impl<Context, CommandPath, Args, Input>
    Handler<Context, SimpleExec<CommandPath, Args>, Input>
    for HandleSimpleExec
where
    Context: CanExtractCommandArg<CommandPath>
        + CanUpdateCommand<Args>
        + CanRaiseAsyncError<std::io::Error>
        + for<'a> CanWrapAsyncError<CommandNotFound<'a>>
        + ...,
    Context::CommandArg: AsRef<OsStr> + Send,
    CommandPath: Send,
    Args: Send,
    Input: Send + AsRef<[u8]>,
{
    type Output = Vec<u8>;

    async fn handle(
        context: &Context,
        _tag: PhantomData<SimpleExec<CommandPath, Args>>,
        input: Input,
    ) -> Result<Vec<u8>, Context::Error> {
        ...
    }
}
```

If you search for the occurance of `SimpleExec` in the Hypershell code base, you will find `HandleSimpleExec`, which is a provider that implements `Handler` specifically to handle `SimpleExec`.

The main method body for `HandleSimpleExec` is not very interesting, and looks mostly similar to regular Rust code. Essentially, it makes use of Tokio's [`Command`](https://docs.rs/tokio/latest/tokio/process/struct.Command.html) to spawn a new child process with the specified arguments. It then make use of the returned [`Child`](https://docs.rs/tokio/latest/tokio/process/struct.Child.html) to write the input to the `STDIN` of the process, and then calls [`wait_with_output`](https://docs.rs/tokio/latest/tokio/process/struct.Child.html#method.wait_with_output) to get the result from `STDOUT`.

Hence, to keep this blog post simple, we omit the method body, and instead focus on the trait signature to see how it is integrated with the rest of Hypershell.

Looking at the generic parameters, you may notice that `SimpleExec<CommandPath, Args>` is used in place of where `Code` was. In other words, `HandleSimpleExec` implements `Handler` if `Code` is in the form `SimpleExec<CommandPath, Args>`. Essentially, we are using Rust generics to "pattern match" on a DSL code fragment, and extract the inner `CommandPath` and `Args` parameters.

### Command Arg Extractor

Inside the `where` clause, we make use of dependency injection to require other dependencies to be provided by the generic `Context`. The first trait, `CanExtractCommandArg`, is defined as follows:

```rust
#[cgp_component {
    provider: CommandArgExtractor,
}]
pub trait CanExtractCommandArg<Arg>: HasCommandArgType {
    fn extract_command_arg(&self, _phantom: PhantomData<Arg>) -> Self::CommandArg;
}

#[cgp_type]
pub trait HasCommandArgType {
    type CommandArg;
}
```

The `CommandArgExtractor` component provides an `extract_command_arg` method to extract a command line argument from an `Arg` code. The method returns an abstract `CommandArg` type, which may be instantiated with types such as `PathBuf` or `String`.

As an example, given a code like `SimpleExec<StaticArg<symbol!("echo")>, ...>`, the `Arg` type that is passed to `CanExtractCommandArg` would be `StaticArg<symbol!("echo")>`. In other words, for `HandleSimpleExec` to implement `Handler<Context, SimpleExec<StaticArg<symbol!("echo")>, ...>, Input>`, it requires `Context` to implement `CanExtractCommandArg<StaticArg<symbol!("echo")>>`.

Since `extract_command_arg` returns an abstract `CommandArg` type, `HandleSimpleExec` also has an additional constraint `Context::CommandArg: AsRef<OsStr> + Send`. This means that the context may choose to instantiate `CommandArg` with any concrete type that implements `AsRef<OsStr> + Send`, such as `PathBuf` or `OsString`.

This also shows that the dependency injection in CGP is more powerful than typical dependency injection frameworks in OOP, as we canuse it with not only the main `Context` type, but also all associated types provided by the context.

### Command Updater

Aside from `CanExtractCommandArg`, we can see that `HandleSimpleExec` also requires `Context: CanUpdateCommand<Args>` to handle the CLI arguments passed to the command. Let's take a look at how the trait is defined:

```rust
#[cgp_component(CommandUpdater)]
pub trait CanUpdateCommand<Args> {
    fn update_command(&self, _phantom: PhantomData<Args>, command: &mut Command);
}
```

Similar to `CanExtractCommandArg`, `CanUpdateCommand` has a generic `Args` parameter to process the CLI arguments specified in the Hypershell program. But instead of returning something, the `update_command` method takes in a `mut` reference to a Tokio [`Command`](https://docs.rs/tokio/latest/tokio/process/struct.Command.html) value.

By passing a `&mut Command` directly, this allows the DSL to provide different kinds of argument syntaxes to configure the CLI excution in different ways. For example, the `WithArgs` syntax allows one to specify a list of CLI arguments, but we can also define new syntaxes such as `WithEnvsAndArgs` to allow specification of _both_ CLI arguments and environment variables for the process.

To see how it works in action, consider the example code:

```rust
SimpleExec<
    StaticArg<symbol!("echo")>,
    WithStaticArgs<Product![
        symbol!("hello"),
        symbol!("world!"),
    ]>,
>
```

The `Args` type given to `HandleSimpleExec` would be `WithStaticArgs<Product![symbol!("hello"), symbol!("world!")]>`, which means the following constraint need to be satisfied:

```rust
Context: CanUpdateCommand<WithStaticArgs<Product![symbol!("hello"), symbol!("world!")]>>
```

To keep focus on the main implementation of `HandleSimpleExec`, we'll omit the details on how the argument updates work. At a high level, the main idea for the implementation is to perform a _type-level iteration_ on the list passed to `WithStaticArgs`. So the implementation would be broken down into two smaller constraints:

```rust
Context: CanUpdateCommand<StaticArg<symbol!("hello")>>
    + CanUpdateCommand<StaticArg<symbol!("world!")>>
```

Once we reach each individual argument, we then make use of `CanExtractCommandArg` to extract the argument, and then call [`Command::arg`](https://docs.rs/tokio/latest/tokio/process/struct.Command.html#method.arg) to add the argument to the `Command`.

It is also worth noting that the `CanUpdateCommand` trait is tightly coupled with the Tokio `Command` type. This would mean that the trait cannot be reused if there are alternative implementations that implement CLI execution without using Tokio. However, this is totally fine, and there is nothing in CGP that prevent us from defining less-abstract interfaces.

Instead, the main advantage that CGP provides is that a trait like `CanUpdateCommand` can be included by specific providers that need it via dependency injection. This means that if the involved providers are not wired with the concrete context, then the context also don't need to implement a trait like `CanUpdateCommand`.

In other words, a CGP trait like `CanUpdateCommand` may be tightly coupled with Tokio, but the trait itself is still fully decoupled from the remaining parts of Hypershell. And so, it would not prevent Hypershell from having alternative implementations that do not use Tokio at all.

### Error Handling

Inside the `where` clause for `HandleSimpleExec`, we can see that it also requires `Context` to implement `CanRaiseAsyncError<std::io::Error>`. Here we will briefly explore how CGP offers a different and more modular approach on error handler.

When calling upstream Tokio methods such as [`Command::spawn`](https://docs.rs/tokio/latest/tokio/process/struct.Command.html#method.spawn), the error `std::io::Error` is returned by the method. However, since the method signature requires an abstract `Context::Error` to be returned in case of errors, we need ways to convert, or "upcast", the `std::io::Error` into `Context::Error`.

A naive approach to handle the error would be to require a _concrete_ error type to be used with the implementation. For example, we could modify the method signature of the `CanHandle` trait to return `anyhow::Error` instead of `Context::Error`. Or, we can add a constraint `Context: HasErrorType<Error = anyhow::Error>` to _force_ the context to use a specific error type, such as `anyhow::Error`. However, doing so would introduce unnecessary coupling between the provider implementation with the concrete error type, and make it impossible for the context to reuse the provider if it wanted to choose another error type for the application.

### Error Raisers

Instead, CGP provides the `ErrorRaiser` component as a way for context-generic implementations to handle errors without requiring access to the concrete error type. The trait is defined as follows:

```rust
#[cgp_component(ErrorRaiser)]
pub trait CanRaiseError<SourceError>: HasErrorType {
    fn raise_error(error: SourceError) -> Self::Error;
}
```

We can think of `CanRaiseError` as a more flexible form of the `From` trait for error handling. In fact, if there exist a `From` instance for all `SourceError`s used by an application, then the provider can be trivially implemented as:

```rust
#[cgp_new_provider]
impl<Context, SourceError> ErrorRaiser<Context, SourceError> for RaiseFrom
where
    Context: HasErrorType,
    Context::Error: From<SourceError>,
{
    fn raise_error(e: SourceError) -> Context::Error {
        e.into()
    }
}
```

When programming with CGP, it is preferred to use `CanRaiseError` instead of directly using `From` to convert a source error to the abstract `Context::Error`. This is because `From` is a plain Rust trait that is subject to the coherence rules in Rust, making it challenging to customize in case if no `From` instance is not implemented by a third party error type like `anyhow::Error`.

On the other hand, using `CanRaiseError` gives us much more freedom to use anything as `SourceError` without worrying about compatibility. For instance, it is common for context-generic implementations to use `CanRaiseError<String>` or even `CanRaiseError<&'static str>`, at least during early prototyping phases. This would have caused issue if we instead require `Context::Error: From<String>`, as types like `anyhow::Error` do not implement `From<String>`.

Going back to the example, with the constraint `CanRaiseError<std::io::Error>` in place, we can now call `Command::spawn()` inside `HandleSimpleExec`, and handle the error with `.map_err(Context::raise_error)`:

```rust
let child = command.spawn().map_err(Context::raise_error)?;
```

### Default Error Type

In the default Hypershell contexts, such as `HypershellCli`, we use [`anyhow::Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html) together with the providers from the [`cgp-error-anyhow`](https://docs.rs/cgp-error-anyhow/) crate to handle errors from different parts of the application.

However, just as everything else, an application can choose different error providers, such as using [`eyre::Report`](https://docs.rs/eyre/latest/eyre/struct.Report.html) together with [`cgp-error-eyre`](https://docs.rs/cgp-error-eyre), to handle the errors from Hypershell programs. This would especially be useful, if users want to embed Hypershell programs within larger applications that use their own structured error types that are defined using [`thiserror`](https://docs.rs/thiserror).

### Error Wrappers

In the `where` clause for `HandleSimpleExec`, we also sees a constraint `Context: for<'a> CanWrapAsyncError<CommandNotFound<'a>>`. Here we will walk through what that entails.

CGP also provides a supplementary error wrapper component, which provides similar functionality as the [`anyhow::Error::context`](https://docs.rs/anyhow/1.0.98/anyhow/struct.Error.html#method.context) method to add additional details about an error. The trait is defined as follows:

```rust
#[cgp_component(ErrorWrapper)]
pub trait CanWrapError<Detail>: HasErrorType {
    fn wrap_error(error: Self::Error, detail: Detail) -> Self::Error;
}
```

Using `CanWrapError`, we can for example add additional details on top of `std::io::Error` to explain that the error happened when we try to spawn the child process. A common frustration with the base error is that when an executable is not found at the specified command path, only a basic `NotFound` error is returned without additional details of _what_ is not found. Using `CanWrapAsyncError`, we can now add additional details to the error about the command that is not found:

```rust
let child = command.spawn().map_err(|e| {
    let is_not_found = e.kind() == ErrorKind::NotFound;
    let e = Context::raise_error(e);

    if is_not_found {
        Context::wrap_error(e, CommandNotFound { command: &command })
    } else {
        e
    }
})?;
```

In the example approach, we first check whether the error kind returned from `command.spawn()` is `ErrorKind::NotFound`. We then use `raise_error` to convert the error into `Context::Error`. After that, if the error kind was `NotFound`, we call `wrap_error` to wrap the error with a custom `CommandNotFound` detail, which is defined as follows:

```rust
pub struct CommandNotFound<'a> {
    pub command: &'a Command,
}

impl<'a> Debug for CommandNotFound<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "command not found: {}",
            self.command.as_std().get_program().to_string_lossy(),
        )
    }
}
```

The `CommandNotFound` struct contains a reference to the `Command` that we are running. We pass the full `Command` struct here, so that it allows potential `ErrorWrapper` implementation to show customized error about the failing command. We also provide a default `Debug` implementation for `CommandNotFound`, which prints out only the program path without additional details about the full command.

Similar to `ErrorRaiser`, CGP allows the `ErrorWrapper` implementation to be chosen by the context to handle the error differently. For instance, the `HypershellCli` context uses the `DebugAnyhowError` provider from `cgp-error-anyhow`, which builds a string using the `Debug` implementation, and then call `anyhow::Error::context` with the formatted string. But if desired, a user of Hypershell is free to override this behavior, and make it print out the full command, or wrap the error in other ways.

Since the `CommandNotFound` contains a lifetime, when we specify the constraint, we need to add a [higher-ranked trait bound](https://doc.rust-lang.org/nomicon/hrtb.html) (HRTB) `for<'a>` to the constraint, so that we can always wrap the error for all lifetime. While it is also possible to pass an owned `Command` value without a lifetime here, it may not be possible in general cases when the detail comes from argument references. Furthermore, having a reference encourages the wrapper handler to only extract essential details, and avoid bloating the error value with large values wrapped within it.

### Input Type

The `Handler` implementation for `HandleSimpleExec` shows that it can work with any generic `Input` type, as long as the constraint `Input: Send + AsRef<[u8]>` is satisfied. This means that aside from `Vec<u8>`, we can also pass to it other compatible types such as `String`, `Bytes`, or `&'a [u8]`.

On the other hand, the constraint shows that `HandleSimpleExec` cannot accept inputs from stream types that implement traits like `AsyncRead`, at least not directly. Since Hypershell is stringly typed, if we want to form a pipeline such as `StreamingExec<...> | SimpleExec<...>`, it would result in a compile-time error.

One way to workaround this is that we can make use of explicit _adapters_ provided by Hypershell, such as `StreamToBytes`, to be part of the pipeline to transform the output before passing as the next input:

```
StreamingExec<...> | StreamToBytes | SimpleExec<...>
```

The main takeaway here is that the supported `Input` and `Output` types in a Hypershell program is determined based on the chosen concrete provider, and _not_ based on the abstract syntax. A concrete context may choose to wire a different provider to handle `SimpleExec`, in which case the supported input/output types for `SimpleExec` may change.

Nevertheless, just as with standard programming languages, it is possible to define a _standard_ around the language syntax to impose the expectations and requirements on how the program should behave. For example, the language specification may state that it should always be possible to pipe the output from `StreamingExec` to `SimpleExec`, and vice versa. In such cases, it would imply that `HandleSimpleExec` alone may not be sufficient to handle all valid Hypershell programs.

But as we will learn later, it is also possible to use the _generic dispatcher_ pattern in CGP to perform _ad hoc dispatch_ to different handlers, based on the `Input` type. In such case, `HandleSimpleExec` would become part of a larger implementation that can be used to handle all possible `Input` types that will be encountered in a Hypershell program.

### Modularity of `HandleSimpleExec`

If we inspect the entire implementation of `HandleSimpleExec`, we would find that other than Tokio, CGP, and the Hypershell core traits, the implementation is fully _decoupled_ from the remaining parts of the application. In fact, we can move the implementation code to an entirely new crate and only include the 3 dependencies, and everything will still work.

This shows that code written with CGP typically have _inverted_ structure in their dependency graphs. Instead of focusing on _concrete types_, CGP starts with _abstract implementations_ and only define the concrete types at the last stage of the process. This significantly reduce bloats in the dependency graph, as each sub-crate can be compiled with only the exact dependencies they need.

To demonstrate the benefit in action, We can look at how Hypershell structures its crate dependencies:

- `hypershell-components` - Defines DSL types and CGP component interfaces, and only depends on `cgp`.
- `hypershell-tokio-components` - Implements Tokio-specific CLI providers and component interfaces. Depends on `cgp`, `hypershell-components`, and `tokio`.
- `hypershell-reqwest-components` - Implements Reqwest-specific HTTP providers and component interfaces. Depends on `cgp` and `hypershell-components`, and `reqwest`.
- `hypershell` - Defines concrete contexts and wiring. Depends on all other Hypershell crates.

As we can see, even though the full Hypershell application uses both Tokio and Reqwest, the crate `hypershell-tokio-components` can be built without `reqwest` being in any part of its dependencies. This may look signficant given that there are only 2 crates. But consider when there is a large Rust application where there are hundreds of dependencies, CGP will make it much easier for the application to break down the dependencies, so that every part of the implementation only needs to be compiled with the exact dependencies they need.

With this level of modularity, it also means that it is possible to build an alternative Hypershell implementations that fully remove `tokio` from its dependencies, and use a different crate to spawn CLI process, such as using [`async-process`](https://github.com/smol-rs/async-process) with [`smol`](https://docs.rs/smol/latest/smol/) as the runtime. Granted, since `reqwest` also depends on `tokio`, if we want to fully remove `tokio`, we would also have to do the same for `hypershell-reqwest-components` and use an alternative HTTP library like [`isahc`](https://docs.rs/isahc).

It is also worth highlighting that with CGP, there would be no need to use _feature flags_ to switch between underlying implementations. Because CGP providers can be implemented fully isolated from each others, we could just create new crates that do not depend on the original providers, and define new contexts that are wired with the alternative providers.

The generic approach is also less error prone than feature flags, as _all_ alternative implementations can co-exist and be tested at the time, as compared to having multiple _variants_ of the code that must be tested separately for each combination of feature flags.

## Wiring for `SimpleExec`

At this point, we have learned how `HandleSimpleExec` is implemented to handle the `SimpleExec` syntax. Next, we will look into how the `HandleSimpleExec` provider is wired up, so that it is accessible from concrete contexts like `HypershellCli`.

### Generic Dispatcher

As we know, aside from `SimpleExec`, there are also other Hypershell syntaxes such as `StreamingExec` and `SimpleHttpRequest`. But since `HandleSimpleExec` only implements `Handler` for `SimpleExec`, we cannot wire it directly as the provider for _all_ generic parameters of `Handler`. Instead, we need an intermediary provider, known as a _generic dispatcher_, to dispatch the handling logic to different providers based on the generic `Code` parameter.

The pattern for provider dispatching based on generic parameters is common enough that CGP provides options to automatically derive them inside the `#[cgp_component]` macro. For the `Handler` component, a dispatcher called `UseDelegate` is provided to handle provider dispatching based on the `Code` parameter.

In CGP, we can declare the dispatching logic in similar ways as normal provider delegation using the `delegate_components!` macro. Following shows a simplified wiring of the providers for `HypershellCli`:

```rust
#[cgp_context(HypershellCliComponents)]
pub struct HypershellCli;

delegate_components! {
    HypershellCliComponents {
        HandlerComponent:
            UseDelegate<HypershellHandlerComponents>,
        ...
    }
}

pub struct HypershellHandlerComponents;

delegate_components! {
    HypershellHandlerComponents {
        <CommandPath, Args> SimpleExec<CommandPath, Args>:
            HandleSimpleExec,
        <CommandPath, Args> StreamingExec<CommandPath, Args>:
            HandleStreamingExec,
        ...
    }
}
```

The first part of the wiring declaration is the same as the hello world example we had earlier. We define a `HypershellCli` struct, with `#[cgp_context]` to make it into a CGP context with `HypershellCliComponents` being the provider. We then use `delegate_components!` on `HypershellCliComponents` to set up the wiring for all providers used by the context. But for the wiring of `HandlerComponent`, we map it to `UseDelegate<HypershellHandlerComponents>` instead of directly to `HandleSimpleExec`.

Following that, we define a new struct `HypershellHandlerComponents`, and then use `delegate_components!` to define some mappings on it. But this time, instead of mapping CGP component names, we map the Hypershell syntax types to their respective providers. In the first entry, we map `SimpleExec` to `HandleSimpleExec`, and then map `StreamingExec` to a `HandleStreamingExec` provider that is implemented separately in Hypershell.

In the mappings for `HypershellHandlerComponents`, we can also see the key for `SimpleExec` being specified as `<CommandPath, Args> SimpleExec<CommandPath, Args>`. The first part, `<CommandPath, Args>` is used as additional _generic parameters_ for the mapping, since we want to map _all_ possible uses of `SimpleExec` to `HandleSimpleExec`. If they were not specified, Rust would instead try to find _specific_ concrete Rust types called `CommandPath` and `Args` imported within the module, and produce errors when failing to find them.

Essentially, we are defining `HypershellHandlerComponents` purely as a key-value map at the _type-level_, and then use it as a _lookup table_ for `UseDelegate`. We can also see that with _types_ being the keys, we get additional expressivity to specify and capture generic parameters in the keys, which wouldn't be possible with value-level lookup tables.

Now that we have walked through the wiring declaration, let's try to visualize how CGP actually implements a trait instance of `CanHandle<SimpleExec<Command, Args>, Input>` for the `HypershellCli` context:

![Diagram](/blog/images/delegate-code.png)

The first two parts of the diagram is similar to how the implementation is done for the example `Greeter` component earlier. For the `HypershellCli` context to implement `CanHandle<SimpleExec<Command, Args>, Input>`, Rust's trait system would first find out that `HypershellCli` implements `HasProvider`, which points to `HypershellCliComponents`.

The trait system then tries to find an implementation of `Handler<HypershellCli, SimpleExec<Command, Args>, Input>` for `HypershellCliComponents`. Next, it sees that `HypershellCliComponents` implements `DelegateComponent<HandlerComponent>`, which points to `UseDelegate<HypershellHandlerComponents>`, and so it continues the implementation lookup there.

This time, the trait system finds that `UseDelegate<HypershellHandlerComponents>` has a candidate implementation for `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`. But for that to be implementated, `UseDelegate` requires `HypershellHandlerComponents` to contain a lookup entry for the `Code` parameter, i.e. that `HypershellHandlerComponents` should implement `DelegateComponent<SimpleExec<Command, Args>>`.

Finally, the system finds that `HypershellHandlerComponents` contains the given entry, which points to `HandleSimpleExec`. It then finds that `HandleSimpleExec` implements `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`, and so the implementation is now completed.

Compared to the earlier `Greeter` example, the delegation chain for `SimpleExec` handling goes 4 levels deep instead of 3. Aside from that, the underlying implementation for `UseDelegate` follows the same pattern as the blanket implementation of the `Handler` provider trait. However, instead of being a blanket implementation, `UseDelegate` is implemented as a _context-generic provider_ for `Handler`. Furthermore, aside from `Handler`, the same pattern has also been implemented by `UseDelegate` for many other CGP traits, such as `ErrorRaiser`, making it a _universal pattern_ that is applicable to any CGP trait that contains additional generic parameters.

The implementation of `UseDelegate` also demonstrates the power of CGP, showing that once the coherence restriction is lifted, there are whole new categories of patterns that can be defined to work with many traits in the same way. Other than `UseDelegate`, there are many other CGP patterns that have been implemented as context-generic providers, such as `UseContext`, `UseType`, `UseField`, `WithProvider`, etc.

## CGP Presets

Earlier, we had a simplified wiring of `HandleSimpleExec` to be used by the `HypershellCli` context. But as we learned in the examples from the beginning, we want to reuse the same wirings also for other contexts, such as `HypershellHttp` and `MyApp`. Furthermore, with the modularity provided by Hypershell, we want to make it easy to extend or customize existing component wirings, and build new collection of wirings that can be shared with the community.

CGP offers _presets_ as a powerful way to build these extensible component wiring, that allows an extensible collection of component wirings to be created. At a high level, a CGP preset is a _module_ that contains a _type-level_ key-value map, together with traits and macros that support _operations_ do be done on the key-value map.

The operations that can be done on a preset shares some similarity to _inheritance_ from the OOP world, at least on the implementation side. Or, to be less buzzwordy, it allows _iteration_ over the _keys_ stored in the type-level key-value map of the preset. As we know from basic algorithm courses, if we can iterate over the keys of a map, we can then construct _new_ maps that shares the same keys as the original map. Or to put it even more simply, CGP presets allows us to perform the Rust equivalent of `map.iter().filter_map()` on a `HashMap`, except that we are doing it at the type level.

Now that we understand how presets work at a high level, it should be clearer on how presets can be used to support inheritance like feature in CGP. There are two kinds of inheritance operations supported by CGP. The first is a simplified one-level single inheritance, which is implemented through Rust traits to allow a CGP context to implement traits like `DelegateComponent` based on all keys stored in one preset.

The second form is a macro-based approach, which enables _nested_ levels of _multiple inheritance_ to be used when defining new presets. The macros work by expanding the preset keys as list _syntax_, e.g. `[KeyA, KeyB, KeyC, ...]`, and then work on the keys syntactically through a separate macro. This means that the macro approach may be less reliable, as we lose access to the precise type information, and that ambiguity may arise if the same identifier may refer to multiple types in scope, or when there are _aliases_ used. However, it is more flexible that we can make it work with more than one map, which is not possible with the trait-based approach due to coherence restrictions.

## Hypershell Presets

Thanks to presets, Hypershell is able to allow its core implementation to be customized easily. Hypershell defines all its component wirings as extensible presets, so that users can choose to extend, replace, or customize any one of the presets.

The main preset that is provided by Hypershell is the `HypershellPreset`, which can be used directly by contexts like `HyperhellCli`. But underlying the main preset, Hypershell actually breaks the components down to a few smaller presets, including `HypershellTokioPreset` for the CLI components, and `HypershellReqwestPreset` for the HTTP components. This allows to one sub-part of the presets to be replaced entirely, while the other parts of the presets can be kept unmodified.

Furthermore, Hypershell also defines the dispatch mappings for components like `HandlerComponent` as presets. With this, we can extend the handler component presets, rather than the main preset, to introduce new _syntaxes_ to the DSL, or customize the wiring for existing syntaxes like `SimpleExec`.

### High Level Diagram

We will now walk through how `HandleSimpleExec` is wired within the Hypershell presets. But before we start, let's look at the high level diagram on the level of indirections:

![Diagram](/blog/images/preset-wiring.png)

There are a lot of indirections going on with the above diagram. We will go through each of the steps one by one, together with the relevant code snippets, so that you can get a better understanding of what is going on.

### Definition of `HypershellCli`

```rust
#[cgp_context(HypershellCliComponents: HypershellPreset)]
pub struct HypershellCli;
```

We start with the definition of the `HypershellCli` context, with the `HypershellPreset` specified as the preset to be inherited by the context's provider, `HypershellCliComponents`. The first part of the implementation stays the same as before, that `HypershellCli` has a blanket implementation for `CanHandle<SimpleExec<Command, Args>, Input>`, if `HypershellCliComponents` implements `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`.

Following that, in order for `HypershellCliComponents` to implement the provider trait, the trait system would look for its `DelegateComponent` entry with `HandlerComponent` being the key, which now points to `HypershellPreset`. The entry is found by the system, via a blanket implementation of `DelegateComponent` using a special `HypershellPreset::IsPreset` trait. This blanket implementation is generated by `#[cgp_context]`, which allows `HypershellCliComponents` to delegate all components from `HypershellPreset` without additional code written.

### Definition of `HypershellPreset`

`HypershellPreset` is defined as follows:

```rust
#[cgp::re_export_imports]
mod preset {
    ...

    cgp_preset! {
        HypershellPreset:
            HypershellTokioPreset
            + HypershellReqwestPreset
            + ...
        {
            ...,
            override HandlerComponent:
                HypershellHandlerPreset::Provider,
            ...,
        }
    }

    ...
}
```

First of all, when defining CGP presets, we need to wrap the code around a `mod preset` that is annotated with `#[cgp::re_export_imports]`. This macro captures all `use` statements within the module, and adds a hidden variant of the imports that become `pub use`. This hack is necessary for the macro-based preset operations to work, as we need to re-import all key identifiers at a child preset in order to bind the identifiers to their original types. The macro also re-exports everything in the inner module, so that we can import the preset as if the wrapper `preset` module is not preset.

We then define `HypershellPreset` using the `cgp_preset!` macro. We can see that the preset makes use of multiple-inheritance to inherit from a number of presets, including `HypershellTokioPreset`, which contains all component wirings for implementing the Hypershell CLI features using `tokio`.

In one of the entries in `HypershellPreset`, we can see that the entry for `HandlerComponent` is specified, but with an additional `override` keyword. An overridden preset entry is useful for handling conflicting entries that arise from multiple inheritance, i.e. the diamond problem, as well as allowing the child preset to override parts of the component wiring provided by the parent preset.

In the case for `HypershellPreset`, the `override` is used, as we want to define a new provider, `HypershellHandlerPreset`, that combines the handlers of different groups of syntaxes coming from different parent presets. When specifying the entry value, we use `HypershellHandlerPreset::Provider`, as `HypershellHandlerPreset` itself is really a module. When we need to refer to the preset as a type, we access the type through the `::Provider` item in the module.

### Definition of `HypershellHandlerPreset`

`HypershellHandlerPreset` is defined as follows:

```rust
cgp_preset! {
    #[wrap_provider(UseDelegate)]
    HypershellHandlerPreset:
        TokioHandlerPreset
        + ReqwestHandlerPreset
        + ...
    { }
}
```

From above, we can see that `HypershellHandlerPreset` is defined as a separate preset within the same module. The preset has an empty body, and merely combines the handler wirings from parent presets such as `TokioHandlerPreset`.

The preset is also annotated with `#[wrap_provider(UseDelegate)]`, which instructs `cgp_preset!` to wrap the `Preset::Provider` type in the preset module with `UseDelegate`. This is because the component entries themselves do not result in a blanket implementation of `Handler`, or any provider trait for that matter. But by wrapping the entry inside `UseDelegate`, the `Handler` trait becomes implemented by performing dispatch to the entries based on the `Code` type.

### Expansion of `cgp_preset!`

Altogether, the the call to `cgp_preset!` roughly expands into the following:

```rust
pub mod HypershellHandlerPreset {
    pub type Provider = UseDelegate<Components>;

    pub struct Components;

    delegate_components! {
        Components {
            // pseudo code for bulk delegation
            TokioHandlerPreset::Components::Keys:
                TokioHandlerPreset::Provider,
            ReqwestHandlerPreset::Components::Keys:
                ReqwestHandlerPreset::Provider,
        }
    }

    // other constructs
    ...
}
```

First, `cgp_preset!` defines a module called `HypershellHandlerPreset`. Inside the module, a `Components` struct is defined, with `delegate_components!` called with the mappings defined within the body of `cgp_preset!`. Additionally, `delegate_components!` is also applied with all the keys in the super presets, with the delegate target set to the super preset's `Provider` type. We use some pseudocode in the above example for better understanding of what actually happened, as the actual syntax looks more verbose and confusing.

When `#[wrap_provider(UseDelegate)]` is used, the macro defines `Provider` to be a type aliase to `UseDelegate<Components>`. When there is no `#[wrap_provider]` specified, such as when we defined `HypershellPreset` earlier, then `Provider` is just a type alias to `Components`.

### Definition of `TokioHandlerPreset`

Next, we will look at how `TokioHandlerPreset` is defined:

```rust
cgp_preset! {
    #[wrap_provider(UseDelegate)]
    TokioHandlerPreset {
        <Path, Args> SimpleExec<Path, Args>:
            HandleSimpleExec,
        <Path, Args> StreamingExec<Path, Args>:
            HandleStreamingExec,
        ...
    }
}
```

As we can see, `TokioHandlerPreset` is defined in similar ways as `HypershellHandlerPreset`, and is also wrapped with `UseDelegate`. The preset now has a non-empty list of entries, with `SimpleExec` mapped to `HandleSimpleExec`, `StreamingExec` mapped to `HandleStreamingExec`, and so on.

Given that `TokioHandlerPreset` implementing only the handlers for the Hypershell syntaxes, we can find the mappings for other syntaxes in other syntaxes, such as `ReqwestHandlerPreset` which provides mapping for `SimpleHttpRequest` and `StreamingHttpRequest`, etc. So when `HypershellHandlerPreset` inherits from both `TokioHandlerPreset` and `ReqwestHandlerPreset`, we are essentially "merging" the entries from both preset mappings to become a single mapping.

### Full Trace of Preset Delegations

Going back to the implementation diagram at the beginning, we can now trace the remaining implementation steps:

- `HypershellPreset`, or more specifically `HypershellPreset::Provider`, has a blanket implementation for `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`, because it has a `DelegateComponent` entry for `HandlerComponent`, which points to `HypershellHandlerPreset::Provider`, which is `UseDelegate<HypershellHandlerPreset::Components>`.
- `UseDelegate<HypershellHandlerPreset::Components>` has a context-generic implementation for `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`, because `HypershellHandlerPreset::Components` has a `DelegateComponent` entry for `SimpleExec<Command, Args>`, which points to `TokioHandlerPreset::Provider`, which is `UseDelegate<TokioHandlerPreset::Components>`
- `UseDelegate<TokioHandlerPreset::Components>` has a context-generic implementation for `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`, because `TokioHandlerPreset::Components` has a `DelegateComponent` entry for `SimpleExec<Command, Args>`, which points to `HandleSimpleExec`.
- `HandleSimpleExec` implements `Handler<HypershellCli, SimpleExec<Command, Args>, Input>`, and therefore the implementation chain completes, and the calls are ultimately forwarded to it.

At this point, you may feel that it is overly complicated to define so many levels of indirections just to wire up a single handler like `HandleSimpleExec`. However, each level of indirection is essential in enabling additional flexibility for customizing Hypershell. This section also serves as the background for understanding of the next section, which we will walk through how to make use of the presets defined here to add new language extensions to the Hypershell DSL. After that, hopefully you will better appreciate the level of modularity introduced in this section.

It is also worth noting that this is not necessarily a recommendation of how _you_ should write CGP code in your own application. In fact, you may not even need presets at all if your initial application only has one concrete context with no further need of customization.

Everything we have described in this section is to explain the internal architecture of Hypershell, which is _not_ required to be understood by end users who want to just use Hypershell without additional customization. Instead, the section is mainly useful for developers who are interested in _extending_ Hypershell, or want to build similar modular DSLs.

# Extending Hypershell

We now have a basic understanding of Hypershell structures and modularize its implementation. To see the kind of benefits it provides, we will try to extend the language to introduce new syntaxes to the DSL.

## Checksum

Recall that in the earlier example for [HTTP requests](#native-http-request), we try to fetch the web content of a URL, and then compute the HTTP checksum of that webpage using the `sha256sum` command. The approach allows us to iterate and get results quickly, but there are rooms for improvement once we get the initial prototype working.

In particular, since we are inside Rust, an obvious optimization would be to use a native library like [`sha2`](https://docs.rs/sha2) to compute the checksum.

### The `Checksum` Syntax

Following the modular DSL design of Hypershell, we first wants to define an _abstract_ syntax that users can use in their Hypershell program. The abstract syntax would decouples the language extension from the concrete implementation, so that users may choose an alternative implementation, such as using the `sha256sum` command, to compute the checsum.

For the purpose of this demo, we will define an abstract `Checksum` syntax as follows:

```rust
pub struct Checksum<Hasher>(pub PhantomData<Hasher>);
```

### `HandleStreamChecksum` Provider

```rust
#[cgp_new_provider]
impl<Context, Input, Hasher> Handler<Context, Checksum<Hasher>, Input> for HandleStreamChecksum
where
    Context: CanRaiseAsyncError<Input::Error>,
    Input: Send + Unpin + TryStream,
    Hasher: Send + Digest,
    Input::Ok: AsRef<[u8]>,
{
    type Output = GenericArray<u8, Hasher::OutputSize>;

    async fn handle(
        _context: &Context,
        _tag: PhantomData<Checksum<Hasher>>,
        mut input: Input,
    ) -> Result<Self::Output, Context::Error> {
        let mut hasher = Hasher::new();

        while let Some(bytes) = input.try_next().await.map_err(Context::raise_error)? {
            hasher.update(bytes);
        }

        Ok(hasher.finalize())
    }
}
```

## WebSocket
