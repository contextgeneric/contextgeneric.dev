---
sidebar_label: 'Glossary'
sidebar_position: 91
toc_max_heading_level: 2
description: 'Short definitions of the CGP, Rust, and related terms used across this documentation, each linking to the page that explains it.'
---

# Glossary

Short definitions of the words used across this documentation, each pointing at the page that
explains the idea properly. It is a place to look a term up, not a place to learn CGP: every entry is
a sentence or two and a link onward.

If you arrived knowing the name of a *construct* rather than a word — `IsPresent`,
`#[cgp_new_provider]` — the [reference index](./index.md) is the page that routes those.

## CGP terms

The vocabulary this documentation uses for CGP's own constructs and roles.

### abstract type

A type that generic code names without fixing, leaving each context to choose it. An error type, a
runtime, or a scalar can be abstract, so intermediate code refers to it without taking it as a
separate type parameter.

[Abstract types](/docs/concepts/abstract-types) · [`#[cgp_type]`](./macros/cgp_type.md)

### aggregate provider

A zero-sized provider holding a wiring table of its own, so a bundle of components can be delegated to
it as one reusable unit. It is dispatched *to* by contexts and is never its own context.

[Aggregate providers](/docs/concepts/aggregate-providers) ·
[`delegate_components!`](./macros/delegate_components.md)

### application context

An environmental context standing for one application, where that application's choices live. It may
carry runtime values such as a client or a configuration, or be an empty `struct App;` whose only job
is to be a name the wiring hangs off.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### blanket trait

A trait with a single blanket implementation covering every type that satisfies its bounds, rather
than a component with interchangeable providers. `#[cgp_fn]` and `#[blanket_trait]` generate these.

[`#[cgp_fn]`](./macros/cgp_fn.md) · [`#[blanket_trait]`](./macros/blanket_trait.md)

### check trait

A generated trait whose supertraits assert that a context can use the components listed, turning a
latent wiring gap into a compile error at the wiring site. Wiring is otherwise **lazy**: a delegation
entry alone does not verify that the provider's dependencies hold.

[Checking your wiring](/docs/concepts/check-traits) ·
[`check_components!`](./macros/check_components.md)

### component

An interface for which a context can select an implementation, generated from one trait as a consumer
trait, a provider trait, and a marker used as the wiring key. A component may contain several methods,
associated types, and consts, exactly as an ordinary trait may.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`#[cgp_component]`](./macros/cgp_component.md)

### component marker

The zero-sized type naming a component in a wiring table, such as `GreeterComponent`. It is the key
the table looks a provider up by, and it carries no behavior of its own.

[`#[cgp_component]`](./macros/cgp_component.md) ·
[`DelegateComponent`](./traits/wiring/delegate_component.md)

### consumer trait

The ordinary, `self`-style trait that callers use, such as `CanGreet`. It stays a normal Rust trait
and can also be implemented directly on a type, without any provider machinery.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`#[cgp_component]`](./macros/cgp_component.md)

### context

The type the method runs on, which supplies the values it needs as its fields. It is also the type
that owns the wiring, so it decides which provider each of its components uses.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### context-generic programming

The name of the paradigm: writing implementations against what a context provides rather than against
a concrete type, so a context selects among them at compile time.

[Introduction](/docs/) · [Bypassing coherence](/docs/concepts/coherence)

### delegation

An entry in a wiring table, mapping one component to the provider that supplies it. Delegating a
component does not by itself check that the provider's dependencies hold — that is what a check trait
is for.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`DelegateComponent`](./traits/wiring/delegate_component.md)

### dispatch

Routing an input to the handler that matches it, chosen by the wiring rather than by a runtime test
of the value. Variant dispatch is checked for exhaustiveness at compile time, without a wildcard arm.

[Dispatching](/docs/concepts/dispatching) ·
[Dispatch combinators](./providers/dispatch/index.md)

### environmental context

A context standing for an application or an environment rather than for the data being operated on,
carrying the implementation choices and dependencies its providers need. Most CGP code is written this
way.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) ·
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits)

### expansion

The ordinary Rust a CGP macro generates — the traits, impls, and marker types behind a construct,
also called what a construct desugars to. `cargo cgp expand` prints it with CGP's type-level
constructs restored to readable macro notation.

[`cargo cgp expand`](/docs/cargo-cgp/expand) ·
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits)

### extensible record

A struct whose fields are exposed to generic code as type-level data, so a value can be built or read
field by field without naming its concrete type.

[Extensible records](/docs/concepts/extensible-records) ·
[`#[derive(CgpRecord)]`](./derives/derive_cgp_record.md)

### extensible variant

An enum whose variants are exposed to generic code as type-level data, so a value can be constructed
or matched variant by variant without naming its concrete type.

[Extensible variants](/docs/concepts/extensible-variants) ·
[`#[derive(CgpVariant)]`](./derives/derive_cgp_variant.md)

### getter trait

A trait declaring a named accessor for a value the context holds, used where an implicit argument
cannot reach — a field on another type, an accessor other code requires by name, or a return type
inferred from the field.

[Implicit arguments](/docs/concepts/implicit-arguments) ·
[`#[cgp_auto_getter]`](./macros/cgp_auto_getter.md)

### handler

A component in CGP's computation family, which models a computation along the axes of synchronous
versus async, fallible versus infallible, and input-taking versus input-free. `Handler` itself is the
general async, fallible case.

[Handlers](/docs/concepts/handlers) · [`Handler`](./components/handler/handler.md)

### higher-order provider

A provider taking another provider as a generic parameter and constraining it with a provider-trait
bound, so its inner behavior is chosen by wiring rather than fixed.

[Higher-order providers](/docs/concepts/higher-order-providers) ·
[`#[use_provider]`](./attributes/use_provider.md)

### impl-side dependency

A requirement an implementation declares without adding it to the interface callers see. A provider
can require `HasName` while the consumer trait says nothing about it, so the bound never reaches a
caller.

[Impl-side dependencies](/docs/concepts/impl-side-dependencies) · [`#[uses]`](./attributes/uses.md)

### implicit argument

A provider argument that looks like an ordinary function parameter but is read from a same-named field
on the context. It is the default way to read a context field.

[Implicit arguments](/docs/concepts/implicit-arguments) · [`#[implicit]`](./attributes/implicit.md)

### marker type

A zero-sized type carrying no data, used as a name the compiler can resolve rather than as a value.
Providers and component markers are both marker types, and neither is ever instantiated.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`PhantomData`](./types/phantom_data.md)

### namespace

A reusable wiring table a context can join, optionally inheriting from a parent, so a growing
configuration stays short. Lookups without a direct entry on the context forward through it.

[Namespaces](/docs/concepts/namespaces) · [`cgp_namespace!`](./macros/cgp_namespace.md)

### parameter-targeted component

A component whose target is a type parameter, while the context selects the implementation, as in
`CanEncode<Value>`. Different applications can treat the same target type differently.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### partial record

The intermediate value produced while a record is being built or taken apart field by field, carrying
markers recording which fields are present so far.

[Extensible records](/docs/concepts/extensible-records) ·
[`MapType`](./traits/type-level/map_type.md)

### provider

A named implementation a context can select, written as a zero-sized marker type that exists only at
the type level. Two providers whose implementations would otherwise overlap can coexist, because each
implements the provider trait on its own type.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`#[cgp_impl]`](./macros/cgp_impl.md)

### provider trait

The interface a provider implements, with `Self` moved out into an explicit context parameter, as in
`Greeter<Context>`. Writing a provider with `#[cgp_impl]` means you rarely write this form by hand.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`#[cgp_impl]`](./macros/cgp_impl.md)

### selector

A type parameter used to choose an implementation rather than to identify the value being operated on.
In `CanCompute<Code, Input>`, `Code` is the selector and `Input` is the target.

[Handlers](/docs/concepts/handlers) · [Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### self-targeted component

A component whose operation acts on, or describes, the context itself, as in `CanGreet` or
`HasErrorType`. Most components are self-targeted.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### target

The thing a component's operation acts on: the context itself for a self-targeted component, or a type
parameter for a parameter-targeted one. A generic parameter alone does not make a component
parameter-targeted.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### type-level string

A string encoded as a type, used to name a field or a path so that the compiler can resolve it. It is
written with the `Symbol!` macro and recovered as ordinary text where a runtime name is needed.

[`Symbol!`](./macros/symbol.md) · [`Chars`](./types/chars.md)

### value context

A context that *is* the data being operated on, as `String` is in `String: CanEncode`. One type is
both the target and the context, so it has one implementation of that component program-wide.

[Modularity Hierarchy](/docs/concepts/modularity-hierarchy)

### wiring

The type-level table on a context recording which provider supplies each component. The compiler
resolves it during type checking, so a wired call compiles to a direct call with no runtime lookup.

[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) ·
[`delegate_components!`](./macros/delegate_components.md)

## Rust terms

Rust concepts this documentation assumes, defined only as far as reading these pages requires. Each
links to Rust's own documentation, which explains them properly.

### associated type

A type declared by a trait and chosen by each implementation, so the implementation decides it rather
than the caller.

[Rust Reference: Associated items](https://doc.rust-lang.org/reference/items/associated-items.html)

### blanket implementation

An implementation written for every type satisfying a set of bounds, such as `impl<T: Display> Foo for T`,
rather than for one named type. CGP's generated code rests on these.

[Rust Reference: Implementations](https://doc.rust-lang.org/reference/items/implementations.html)

### coherence

Rust's guarantee that any given trait and type have at most one implementation, so every lookup
resolves the same way anywhere in a program. The overlap and orphan rules are what buy it.

[Rust Reference: Implementations](https://doc.rust-lang.org/reference/items/implementations.html) ·
[Bypassing coherence](/docs/concepts/coherence)

### dynamic dispatch

Choosing an implementation at run time through a pointer to a method table, as `dyn Trait` does.

[Rust Book: Trait objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)

### generic parameter

A type, lifetime, or const a definition is written against and a caller supplies, so the definition
works for many concrete arguments.

[Rust Book: Generic data types](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### lifetime

A named region of code for which a reference is valid, which the compiler uses to check that borrowed
data outlives the borrow.

[Rust Book: Lifetime syntax](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

### monomorphization

The compiler generating a separate copy of generic code for each set of concrete type arguments it is
used with.

[Rust Book: Generic data types](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### orphan rule

The restriction that an implementation must be written in the crate owning either the trait or the
type, which stops two crates implementing the same trait for the same type.

[Rust Reference: Implementations](https://doc.rust-lang.org/reference/items/implementations.html)

### `PhantomData`

A zero-sized marker letting a type carry a parameter it does not store a value of. CGP uses it to pass
type-level names into a method and to parameterize provider structs.

[std: `PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)

### procedural macro

A macro implemented as Rust code that reads and produces token streams, which is how CGP's attributes
and derives are built.

[Rust Reference: Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)

### static dispatch

Resolving a call at compile time to a direct call to a known function, with no runtime lookup. CGP
wiring resolves this way.

[Rust Book: Generic data types](https://doc.rust-lang.org/book/ch10-01-syntax.html)

### supertrait

A trait required by another trait, so implementing the second means the first is available too. CGP
adds one with `#[extend]` rather than native `:` syntax, which keeps the generated trait's bounds
together.

[Rust Reference: Traits](https://doc.rust-lang.org/reference/items/traits.html) ·
[`#[extend]`](./attributes/extend.md)

### trait

A named set of methods and associated items a type can implement, and the interface generic code is
written against.

[Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)

### trait bound

A requirement that a type parameter implement a given trait, written as `T: Display` or in a `where`
clause.

[Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)

### trait object

A value accessed through a trait rather than its concrete type, written `dyn Trait`, with its methods
resolved at run time.

[Rust Reference: Trait objects](https://doc.rust-lang.org/reference/types/trait-object.html)

## Related concepts

Ideas from other languages and paradigms that readers often arrive holding. Each entry defines the
term as its own community uses it, and links both to that community's documentation and to the
comparison page that places CGP against it.

### algebraic effect

An operation whose meaning is supplied by a handler installed by the caller, rather than by the code
performing it, letting effectful code be written without committing to an interpretation.

[Comparison: Algebraic effects](/docs/comparisons/algebraic-effects) ·
[Koka: the book](https://koka-lang.github.io/koka/doc/book.html)

### capability

An unforgeable value that both designates a resource and authorizes its use, so holding it *is* the
permission. CGP's arrangement resembles it in that a provider reaches only what its declared
dependencies supply, and the comparison page sets out how far the resemblance goes.

[Comparison: Capabilities](/docs/comparisons/capabilities) ·
[Object-capability model](https://en.wikipedia.org/wiki/Object-capability_model)

### CRTP

The curiously recurring template pattern: a C++ base class parameterized by its own derived class, so
a base can call into the derived type with calls resolved at compile time.

[Comparison: Policy-based design](/docs/comparisons/policy-based-design) ·
[CRTP](https://en.cppreference.com/w/cpp/language/crtp)

### dependency injection

Supplying a component's dependencies from outside rather than having it construct them, so the choice
of implementation is made by the code assembling the program.

[Comparison: Dependency injection](/docs/comparisons/dependency-injection) ·
[Baeldung: IoC and dependency injection](https://www.baeldung.com/inversion-control-and-dependency-injection-in-spring)

### dictionary passing

The implementation strategy in which a type-class constraint becomes a hidden argument carrying the
chosen implementation's methods, passed down through calls.

[Comparison: Type classes](/docs/comparisons/type-classes) ·
[How to make ad-hoc polymorphism less ad hoc](https://dl.acm.org/doi/10.1145/75277.75283)

### effect handler

The construct interpreting an algebraic effect's operations, deciding what each one does and where
control resumes afterwards.

[Comparison: Algebraic effects](/docs/comparisons/algebraic-effects) ·
[Plotkin and Pretnar: Handling algebraic effects](https://homepages.inf.ed.ac.uk/gdp/publications/Effect_Handlers.pdf)

### functor

In ML, a module parameterized by another module: given a structure satisfying a signature, it produces
a new structure.

[Comparison: ML modules](/docs/comparisons/ml-modules) ·
[OCaml: functors](https://ocaml.org/docs/functors)

### implicit parameter

An argument the compiler supplies from context rather than the caller writing it, as Scala's `using`
parameters and Haskell's `ImplicitParams` do.

[Comparison: Implicit parameters](/docs/comparisons/implicit-parameters) ·
[Scala 3: context parameters](https://docs.scala-lang.org/scala3/book/ca-context-parameters.html)

### instance resolution

The compiler's search for the implementation satisfying a type-class constraint, which in a coherent
system finds at most one and does not ask the programmer to choose.

[Comparison: Type classes](/docs/comparisons/type-classes) ·
[GHC: instance declarations](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/instances.html)

### IoC container

The runtime registry in a dependency-injection framework that holds bindings from interfaces to
implementations and constructs the object graph from them.

[Comparison: Dependency injection](/docs/comparisons/dependency-injection) ·
[Spring: the IoC container](https://docs.spring.io/spring-framework/reference/core/beans/dependencies/factory-collaborators.html)

### modular implicits

A proposed OCaml feature allowing modules to be passed implicitly, bringing type-class-style
resolution to the module system.

[Comparison: ML modules](/docs/comparisons/ml-modules) ·
[Modular implicits](https://arxiv.org/abs/1512.01895)

### named impl

An implementation given a name so that it can be selected explicitly, rather than being anonymous and
resolved by coherence. Several Rust proposals and Cairo's shipped feature take this shape.

[Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals) ·
[An Incoherent Rust](https://www.boxyuwu.blog/posts/an-incoherent-rust/)

### object capability

A capability in the object-capability model, where a reference to an object is itself the authority to
invoke it, and authority spreads only by passing references.

[Comparison: Capabilities](/docs/comparisons/capabilities) ·
[Robust composition](https://papers.agoric.com/assets/pdf/papers/robust-composition.pdf)

### policy-based design

The C++ technique of assembling a class from interchangeable policy classes supplied as template
arguments, so behavior is composed at compile time.

[Comparison: Policy-based design](/docs/comparisons/policy-based-design) ·
[Policy-based design](https://en.wikipedia.org/wiki/Policy-based_design)

### reflection

Inspecting a type's structure — its fields, variants, and names — from within the program. CGP's
equivalent is encoded in types and resolved at compile time rather than read from runtime metadata.

[Comparison: Reflection](/docs/comparisons/reflection) ·
[Rust: reflection and comptime goal](https://rust-lang.github.io/rust-project-goals/2026/reflection-and-comptime.html)

### row polymorphism

A type system in which a record or variant type is parameterized by a *row* of labelled fields, so
code can be written against types that have at least the fields it needs.

[Comparison: Row polymorphism](/docs/comparisons/row-polymorphism) ·
[Row polymorphism](https://en.wikipedia.org/wiki/Row_polymorphism)

### row variable

The variable standing for "the rest of the fields" in a row-polymorphic type, letting one definition
apply to records that differ in the fields it does not mention.

[Comparison: Row polymorphism](/docs/comparisons/row-polymorphism) ·
[PureScript: types](https://github.com/purescript/documentation/blob/master/language/Types.md)

### signature and structure

In ML, a signature is the interface a module satisfies, and a structure is a module satisfying it —
roughly a type and its implementation, related by ascription rather than by declaration.

[Comparison: ML modules](/docs/comparisons/ml-modules) ·
[ML modules, explained](https://cs.wellesley.edu/~cs251/s12/handouts/modules.pdf)

### specialization

A proposed Rust feature letting a more specific implementation override a more general one, which
would relax the overlap rule in a controlled way. It remains unstable.

[Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals) ·
[RFC 1210](https://rust-lang.github.io/rfcs/1210-impl-specialization.html) ·
[Tracking issue](https://github.com/rust-lang/rust/issues/31844)

### structural typing

Treating types as compatible when their structure matches, rather than when they were declared to be
related; its dynamic-language cousin is duck typing. Rust remains nominal, and CGP's wiring is
explicit, so the resemblance is partial.

[Comparison: Row polymorphism](/docs/comparisons/row-polymorphism) ·
[Structural type system](https://en.wikipedia.org/wiki/Structural_type_system)

### type class

An interface a type can be declared to satisfy after the fact, with the compiler selecting the
implementation from the types at a call site.

[Comparison: Type classes](/docs/comparisons/type-classes) ·
[Type class](https://en.wikipedia.org/wiki/Type_class)

### vtable

The table of function pointers behind a dynamically dispatched call, consulted at run time to find the
implementation. CGP wiring has no runtime counterpart to it.

[Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch) ·
[Virtual method table](https://en.wikipedia.org/wiki/Virtual_method_table)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
