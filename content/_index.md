+++

title = "Context-Generic Programming"

insert_anchor_links = "heading"

sort_by = "weight"

+++

# Announcement

I am excited to announce the release of [**cgp-serde**](/blog/cgp-serde-release/), a modular serialization library for Serde that leverages the power of [**Context-Generic Programming**](/).

[Read the announcement blog post](/blog/cgp-serde-release/) to find out more.

# Quick Introduction

Context-Generic Programming (CGP) is a modular programming paradigm that enables you to bypass the **coherence restrictions** in Rust traits, allowing for **overlapping** and **orphan** implementations of any CGP trait.

You can adapt almost any existing Rust trait to use CGP today by applying the `#[cgp_component]` macro to the trait definition. After this annotation, you can write **named** implementations of the trait using `#[cgp_impl]`, which can be defined without being constrained by the coherence rules. You can then selectively enable and reuse the named implementation for your type using the `delegate_components!` macro.

For instance, we can, in principle, annotate the standard library’s [`Hash`]([https://doc.rust-lang.org/std/hash/trait.Hash.html]\(https://doc.rust-lang.org/std/hash/trait.Hash.html\)) trait with `#[cgp_component]` like this:

```rust
#[cgp_component(HashProvider)]
pub trait Hash { ... }
```

This change does not affect existing code that uses or implements `Hash`, but it allows for new, potentially overlapping implementations, such as one that works for any type that also implements `Display`:

```rust
#[cgp_impl(HashWithDisplay)]
impl<T: Display> HashProvider for T { ... }
```

You can then apply and reuse this implementation on any type by using the `delegate_components!` macro:

```rust
pub struct MyData { ... }
impl Display for MyData { ... }

delegate_components! {
    MyData {
        HashProviderComponent: HashWithDisplay,
    }
}
```

In this example, `MyData` implements the `Hash` trait by using `delegate_components!` to delegate its implementation to the `HashWithDisplay` provider, identified by the key `HashProviderComponent`. Because `MyData` already implements `Display`, the `Hash` trait is now automatically implemented through CGP via this delegation.

# Current Status

As of 2025, CGP remains in its _early stages_ of development. While promising, it still has several rough edges, particularly in areas such as documentation, tooling, debugging techniques, community support, and ecosystem maturity.

As such, adopting CGP for serious projects comes with inherent challenges, and you are advised to proceed _at your own risk_. The primary risk is not technical but stems from the limited support available when encountering difficulties in learning or applying CGP.

At this stage, CGP is best suited for early adopters and potential [contributors](/overview/#contribution) who are willing to experiment and help shape its future.

# Getting Started

There are several places where you can get started learning and using CGP.

## Hello World Tutorial

The [Hello World Tutorial](/tutorials/hello) gives a high level walkthrough of various CGP features using a hello-world style example.

## Book

To better understand CGP, the best approach is to dive into our book, [Context-Generic Programming Patterns](https://patterns.contextgeneric.dev/). It provides a comprehensive guide to understanding and working with CGP.

## Applications

You can also explore real-world applications and projects that use CGP, such as [Hermes SDK](https://github.com/informalsystems/hermes-sdk/), to gain a deeper understanding of its practical uses.

## Resources

Check out the [Resources](/resources) page for more materials and learning tools to help you get up to speed with CGP.
