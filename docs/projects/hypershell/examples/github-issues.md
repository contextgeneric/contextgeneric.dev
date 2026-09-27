---
sidebar_label: 'github_issues'
sidebar_position: 8
description: 'A Hypershell program that calls the GitHub API with a URL built from context fields and decodes the JSON reply into a Rust type.'
---

# Call a JSON API and decode the response

This program asks the GitHub API for the open issues of a repository and decodes the JSON reply into
Rust values. It is an example from [Hypershell](../index.md), a shell-scripting language whose
programs are Rust types, built with [CGP](/docs/). The page shows a URL assembled from several
arguments, a request header, and a program whose output is a Rust type it names itself.

:::tip

### New to CGP?

This page builds on [`nix_manual`](./nix-manual.md), which introduces Hypershell's argument syntax.
For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) is the quickest start, and
[Impl-side dependencies](/docs/concepts/impl-side-dependencies) explains the requirements that let
arguments nest.

:::

## The problem

The task is to fetch a repository's open issues from the GitHub API and decode them into Rust
values. The request needs a URL assembled from a base URL and two path segments that must be
URL-encoded, and a `User-Agent` header, and the reply has to become a typed value rather than bytes.

### Without CGP

In plain Rust, `reqwest` and `serde` do this in a few lines: format the URL, set the header, send
the request, and decode the body with `.json::<Vec<Issue>>()`. For a one-off call, that is the right
approach, and simpler than this one. What it gives up is the program as a unit of its own: the URL's
parts, the header, and the decoding target are spread through code tied to one HTTP client. This
page writes all three into the program, where each URL part says where its value comes from and the
decoding target is a type the program names.

## Run it

From the root of the [Hypershell repository](https://github.com/contextgeneric/hypershell):

```sh
cargo run --example github_issues
```

It needs the network and unauthenticated access to `api.github.com`, which GitHub rate-limits. It
prints the open issues of `rust-lang/rust` as a debug-formatted list of `Issue` values.

## The program

From
[`github_issues.rs`](https://github.com/contextgeneric/hypershell/blob/main/crates/hypershell-examples/examples/github_issues.rs):

```rust
pub type Program = hypershell! {
    SimpleHttpRequest<
        GetMethod,
        JoinArgs [
            FieldArg<"base_url">,
            StaticArg<"/repos/">,
            UrlEncodeArg<FieldArg<"github_org">>,
            StaticArg<"/">,
            UrlEncodeArg<FieldArg<"github_repo">>,
            StaticArg<"/issues">,
        ],
        WithHeaders [
            Header<
                StaticArg<"User-Agent">,
                StaticArg<"hypershells">,
            >
        ],
    >
    | DecodeJson<Vec<Issue>>
};

#[derive(HasField)]
pub struct MyApp {
    pub http_client: Client,
    pub base_url: String,
    pub github_org: String,
    pub github_repo: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Issue {
    pub id: u64,
    pub state: String,
    pub title: String,
}
```

The context, the type the program runs on, joins `HypershellNamespace`, and `main` sets the three
fields to `https://api.github.com`, `rust-lang`, and `rust`.

## A URL built from parts

`JoinArgs` joins a list of arguments into one value. In the URL position it concatenates them, so
the URL is the base from a field, a literal path segment, and the organization and repository names
from two more fields. Each name is wrapped in `UrlEncodeArg`, which percent-encodes it for use in a
URL. The same `JoinArgs` in a file-path position joins path segments instead, because the path
interpreter gives it that meaning.

Arguments nest because each one is interpreted through the context. `UrlEncodeArg<FieldArg<…>>`
works because the encoder asks the context for its inner argument's value, and the context's wiring
answers with the field. Any argument the context can interpret may appear inside another.

## A header, and a response decoded by type

`WithHeaders` is a list of `Header` entries, each a name and a value given as arguments. GitHub's
API rejects requests without a `User-Agent`, so the program sends one.

`SimpleHttpRequest` collects the whole response body as bytes, and `DecodeJson<Vec<Issue>>` decodes
those bytes with `serde_json` into the type it names. `Issue` is an ordinary Rust struct that
derives `Deserialize`; the program names it as a type parameter, so the JSON's shape is part of the
program. Because the last stage produces a `Vec<Issue>`, that is what `handle` returns, and `main`
prints it without any conversion:

```rust
let response = app.handle(PhantomData::<Program>, Vec::new()).await?;

println!("List of GitHub issues: {response:#?}");
```

A request that fails, such as one refused by the rate limit, returns an error for the non-success
status rather than handing the error page to the decoder.

## The pattern

This program shows **nested interpretation through the context**: an argument expression built from
other expressions, each resolved by the context's wiring at every level. It is the same pattern the
pipeline uses for its stages, applied to a smaller language inside the larger one. In CGP terms,
each interpreter asks the context for the traits it needs for its inner parts, as [impl-side
dependencies](/docs/concepts/impl-side-dependencies), so nesting needs no special support.

The cost is the depth of what the compiler resolves. Every level of nesting is another lookup
through the context, so a mistake deep in an argument, such as a field the context lacks, is
reported at the end of a long chain of lookups, which is why checking the program first helps.

## Where to go next

- [`rust_playground`](./rust-playground.md): the next example, whose input is a Rust value too.
- [Writing a program](../guides/writing-a-program.md#read-runtime-values-from-a-custom-context):
  every kind of argument.
- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the requirements that make nested
  interpretation work.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
