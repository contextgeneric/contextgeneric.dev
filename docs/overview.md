---
sidebar_position: 3
---

# Overview

CGP lets you choose trait implementations at compile time and reuse application logic across those
choices. This page explains its main features, the problems they address, and the trade-offs to
consider. Each section links to a fuller explanation; for a working example, start with the
[Hello World tutorial](/docs/tutorials/hello).

## Key Features

CGP builds on Rust's traits and associated types to separate reusable logic from the concrete choices
an application makes. A **context** is the type that holds those choices. A **provider** is a named
implementation, and wiring selects the provider for each component interface.

### One Interface, Many Implementations

CGP lets several implementations of the same interface coexist, even when they apply to the same
types. Each context selects the implementation it needs. For example, a production application can
send email while a test application records messages, with both exposing the same interface to
callers.

CGP separates the trait callers use, the consumer trait, from the provider trait that named
implementations implement. Each provider uses its own marker type, so the implementations remain
distinct under Rust's [coherence](/docs/reference/glossary#coherence) rules. Read
[consumer and provider traits](/docs/concepts/consumer-and-provider-traits) for the mechanism and
[coherence](/docs/concepts/coherence) for the rules it works within.

### Zero-Cost Abstraction

The compiler resolves CGP wiring into statically dispatched calls. Selecting a provider does not
require a runtime registry, reflection, or a virtual method table. The implementation you select
still has its own runtime costs, and generic composition can increase compilation time and generated
code size.

### Type-Safe Wiring

Rust checks that a selected provider satisfies the requirements of the code using it. Missing
dependencies become compile errors when the component is used or explicitly checked. Wiring alone
does not force that verification, so use
[check traits](/docs/concepts/check-traits) to check a context's components where you assemble them.

These checks establish that components fit together; they do not prove application logic correct.
[cargo-cgp](/docs/cargo-cgp) helps explain wiring failures by showing the root cause and dependency
chain. It is an early pre-release and does not rewrite every kind of compiler error.

### Abstract Over Every Dependency

CGP lets reusable logic state the traits it needs while each application supplies concrete
implementations. Those choices can include I/O, storage, error handling, and runtime operations.
Providers declare their requirements where they use them, through
[impl-side dependencies](/docs/concepts/impl-side-dependencies).

Keeping platform dependencies behind these interfaces can make the shared logic usable in `no_std`
environments. CGP itself supports `no_std`, but portability also depends on the providers and other
dependencies you select. A provider that requires an operating-system service still needs that
service on its target platform.

### Still Ordinary Rust

CGP's macros generate ordinary Rust traits and implementations that compile on stable Rust. You can
implement a consumer trait directly, introduce providers where you need them, and keep the rest of
your project unchanged. Contexts can contain ordinary generic types, enums, and [trait objects](/docs/reference/glossary#trait-object).

The additional structure has a cost: you need to learn the provider model and maintain the wiring.
For an interface with a single implementation, an ordinary trait often supplies all the abstraction
you need. The [modularity hierarchy](/docs/concepts/modularity-hierarchy) explains how to choose the
amount of separation your code needs.

### Abstract Types

An [abstract type](/docs/reference/glossary#abstract-type) lets each context choose a concrete type, such as its error type, without passing
that choice as a separate generic parameter through every layer. Code names the associated type
where it needs it. A caller that only invokes a trait's method can depend on that trait without
listing the types used inside its implementation.

These are Rust associated types, so their guarantees depend on the bounds you declare. Using `f64`
for both a distance and a weight does not create distinct concrete types or enforce units. Use
newtypes when you need that distinction. See [abstract types](/docs/concepts/abstract-types).

### Extensible Records and Variants

CGP lets generic code build records by field and handle enums by variant. Types opt in through
derives that expose their structure to the trait system. A library can then assemble records from
independent field providers or combine handlers for the variants an application uses.

The compiler checks this composition against the selected types. It does not discover new fields or
variants at runtime. Read [extensible records](/docs/concepts/extensible-records),
[extensible variants](/docs/concepts/extensible-variants), and
[dispatching](/docs/concepts/dispatching) for the patterns and their limits.

### Composable Handlers

CGP provides components for synchronous, asynchronous, fallible, and infallible computations.
Providers can implement individual steps and combine them into a pipeline whose adjacent input and
output types must agree. An application chooses the steps through its wiring.

The [handler family](/docs/concepts/handlers) covers these interfaces and their composition. Ordinary
function calls remain a simpler choice when a sequence has no need for interchangeable steps.

## Problems Solved

CGP is most useful when a program already needs several implementations or dependency choices. The
following cases show how its features address those needs.

### Error Handling

Reusable logic can return errors without committing to a particular error library. `HasErrorType`
supplies the context's error type, and `CanRaiseError` converts a source error into it. An application
chooses both the error representation and the conversion behavior, allowing the same logic to use a
custom error enum or a general-purpose error library.

Changing the error type also requires compatible conversion providers. CGP makes those choices
explicit; it does not invent conversions between arbitrary errors. See
[modular error handling](/docs/concepts/modular-error-handling).

### Async Runtime

Application logic can depend on the runtime operations it uses without naming a particular executor.
For example, a provider that needs a timer can require a timer trait, leaving the application to
supply its implementation. Another application can reuse that provider with a different compatible
timer implementation.

Changing runtimes requires providers for the operations the application needs, with compatible types
and behavior. It can also require attention to
[`Send` bounds](/docs/concepts/send-bounds) when futures move between threads. CGP separates these
choices but does not make runtime APIs interchangeable by itself.

### Overlapping Implementations

Rust rejects [blanket implementations](/docs/reference/glossary#blanket-implementation) that could apply to the same type, even when different
applications want different choices. CGP gives each implementation a provider name and makes the
choice explicit in the context's wiring. A crate can also implement a provider trait for its own
provider type even when the trait and the data being handled come from other crates.

This applies to interfaces that support CGP. It does not let you add arbitrary implementations of
unchanged foreign traits, such as the standard library's `Hash`, to foreign types. The
[coherence explanation](/docs/concepts/coherence) shows how providers stay within Rust's rules.

### Dynamic Dispatch

CGP offers static composition when an application's implementation choices are known at build time.
For enum-based designs, it can combine separate variant handlers so generic logic works across enums
containing different sets of variants. This lets each handler be reused without duplicating its
logic in every matching operation.

Runtime choices can still use enums or `dyn Trait` inside a CGP context. A program that loads
implementations dynamically needs a runtime mechanism for doing so. CGP's
[dispatching](/docs/concepts/dispatching) selects the composition at compile time; an enum's active
variant is still determined at runtime.

### Monolithic Traits

A large trait can force every implementation to supply methods it never uses. CGP lets you
split independently chosen behavior into smaller components, each with its own providers. A
provider declares the dependencies its implementation needs, while callers depend on the interface
they use.

Group methods and associated types when one implementation choice should decide them together.
Splitting every item into a separate component adds wiring without necessarily improving reuse.
See [impl-side dependencies](/docs/concepts/impl-side-dependencies) and the
[modularity hierarchy](/docs/concepts/modularity-hierarchy).

### Generic Parameters Through Every Layer

An error type, runtime, and storage backend can become generic parameters that intermediate code
must repeat even when it only forwards a call. CGP lets the context select those types and expose
the required operations as traits on the context. Intermediate code can then name the trait it calls,
while the implementation names the dependencies it actually uses.

Types that appear in an interface, such as a returned error, still belong in that interface.
[Abstract types](/docs/concepts/abstract-types) reduce repeated parameters; they do not remove the
need to state type relationships. For a function with one or two independent type parameters,
ordinary generics may be clearer.

---

*An AI agent revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
