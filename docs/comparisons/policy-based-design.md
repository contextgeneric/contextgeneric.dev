---
sidebar_label: 'Policy-based design'
sidebar_position: 2
description: 'CGP read against C++ policy classes, host templates, CRTP, and concepts, for a reader who composes behavior with templates.'
---

# C++ policy-based design

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows C++
policy-based design, the curiously recurring template pattern (CRTP), and C++20 concepts. CGP's
context, providers, and wiring table are the same compile-time composition with the same zero
runtime cost, and this page shows where the two agree, where CGP declares and checks what a template
leaves implicit, and what a template can do that CGP cannot.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. It plays the host class's part.

| In C++ | In CGP |
| --- | --- |
| A policy class | A **provider**: a zero-sized type implementing one component |
| The policy interface, documented by hand | A **component**: the interface written as a trait |
| The host class | The context |
| A host class template with policies as type parameters | A **higher-order provider** |
| The template argument list `Host<PolicyA, PolicyB>` | The **wiring table** written with `delegate_components!` |
| `static_cast<Derived*>(this)` in a CRTP base | `self` inside `#[cgp_impl]`, which already means the context |
| A concept check at instantiation | `check_components!` at the wiring site |

## The idea, briefly

Policy-based design builds a class from interchangeable *policy* classes supplied as template
parameters, so a *host* class composes its behavior at compile time from parts a user chooses.
Andrei Alexandrescu named the technique in *Modern C++ Design*: decompose a class's behavior into
orthogonal policies, each a small class implementing one aspect, and write the main class as a
template that takes the policies as parameters and derives from or holds them. A library "can
support an exponential number of different behavior combinations, resolved at compile time"
([Wikipedia, *Modern C++ Design*](https://en.wikipedia.org/wiki/Modern_C%2B%2B_Design)). The
technique is "a compile-time variant of the strategy pattern"
([Wikipedia, *Policy-based design*](https://en.wikipedia.org/wiki/Policy-based_design)): a policy
selects an algorithm through a template argument, and the compiler inlines the result.

### Policies and the host class

The reference example composes a greeting from an output policy and a language policy:

```cpp
template <typename OutputPolicy, typename LanguagePolicy>
class HelloWorld : private OutputPolicy, private LanguagePolicy {
public:
    void run() const {
        write(message());
    }
};

class WriteToStdout {
protected:
    void write(std::string&& message) const {
        std::println("{}", message);
    }
};

class EnglishMessage {
protected:
    [[nodiscard]] std::string message() const noexcept { return "Hello, World!"; }
};

class GermanMessage {
protected:
    [[nodiscard]] std::string message() const noexcept { return "Hallo Welt!"; }
};

int main() {
    HelloWorld<WriteToStdout, EnglishMessage> helloWorld;
    helloWorld.run();

    HelloWorld<WriteToStdout, GermanMessage> helloWorld2;
    helloWorld2.run();
}
```

`HelloWorld<WriteToStdout, GermanMessage>` is a distinct type from
`HelloWorld<WriteToStdout, EnglishMessage>`, each with its own inlined `run`. The host's `run`
calls `write` and `message` without knowing which policy supplies them, and the compiler checks that
the chosen policies supply them only when `run` is instantiated.

### The policy interface is implicit

A policy has no declared interface. Any class with members of the right names and types qualifies.
The policy interface "doesn't have a direct, explicit representation in code, but rather is defined
implicitly, via duck typing, and must be documented separately and manually"
([Wikipedia, *Policy-based design*](https://en.wikipedia.org/wiki/Policy-based_design)). The
consequence is the classic template failure mode: a policy missing a member produces an error inside
the host's body, after instantiation, phrased in terms of the substituted types rather than the
requirement that was violated. The [row polymorphism](./row-polymorphism.md) page places this duck
typing in the wider structural-versus-nominal landscape.

### CRTP: a base that knows its derived class

The curiously recurring template pattern lets a base class template call into the class deriving
from it, statically. The derived class passes itself as the base's template argument, and the base
reaches it with a `static_cast`:

```cpp
template <class Derived>
struct Base {
    void name() { static_cast<Derived*>(this)->impl(); }
protected:
    Base() = default;
};

struct D1 : public Base<D1> { void impl() { std::puts("D1::impl()"); } };
struct D2 : public Base<D2> { void impl() { std::puts("D2::impl()"); } };
```

The pattern gives static polymorphism with no virtual call, because `Derived` is known at compile
time, and policies and mixins use it whenever they need the host's own type. C++23's *deducing this*
removes the manual parameter and cast for the common case
([cppreference, *CRTP*](https://en.cppreference.com/w/cpp/language/crtp)). Its limit is
homogeneity: `Base<D1>` and `Base<D2>` are unrelated types, so a `std::vector<Base*>` cannot hold
both.

### Concepts: making the requirements explicit

C++20 concepts give a template's requirements a name and let the compiler check them before entering
the body:

```cpp
template<typename T>
concept Hashable = requires(T a) {
    { std::hash<T>{}(a) } -> std::convertible_to<std::size_t>;
};

template<Hashable T>
void f(T) {}
```

The gain is in diagnostics: instead of dozens of lines about an invalid expression deep inside an
algorithm, the error reads that a named concept was not satisfied, at the call
([cppreference, *Constraints and concepts*](https://en.cppreference.com/w/cpp/language/constraints)).
What concepts do not change is *when* the body is checked. Satisfaction is checked by substitution at
instantiation, and a template body is not verified against its concepts at definition, so a host may
use a member its concept never mentions and compile until a policy without that member is
substituted.

## How CGP expresses it

CGP is policy-based design with the policy interface declared as a trait, the composition gathered
into a wired context, and the checking moved to the provider's definition. The correspondence is
construct for construct, and the differences fall out of Rust's trait system doing the job that C++
templates leave to duck typing.

### Providers are policies; the context is the host

Each policy becomes a provider of a component, and the host becomes a context that wires one provider
per component. The greeting example declares the two policy interfaces as components, writes each
policy as a provider, and writes the host's `run` as a function over any context that supplies both:

```rust
#[cgp_component(MessageProvider)]
pub trait HasMessage {
    fn message(&self) -> String;
}

#[cgp_component(Writer)]
pub trait CanWrite {
    fn write(&self, message: String);
}

#[cgp_impl(new EnglishMessage)]
impl MessageProvider {
    fn message(&self) -> String {
        "Hello, World!".to_owned()
    }
}

#[cgp_impl(new GermanMessage)]
impl MessageProvider {
    fn message(&self) -> String {
        "Hallo Welt!".to_owned()
    }
}

#[cgp_impl(new WriteToStdout)]
impl Writer {
    fn write(&self, message: String) {
        println!("{message}");
    }
}

#[cgp_fn]
#[uses(HasMessage, CanWrite)]
pub fn run(&self) {
    self.write(self.message());
}
```

Two hosts are two contexts, each selecting its policies in a wiring table where the C++ version
passes them as template arguments:

```rust
pub struct EnglishApp;
pub struct GermanApp;

delegate_components! {
    EnglishApp {
        MessageProviderComponent: EnglishMessage,
        WriterComponent: WriteToStdout,
    }
}

delegate_components! {
    GermanApp {
        MessageProviderComponent: GermanMessage,
        WriterComponent: WriteToStdout,
    }
}

EnglishApp.run();   // Hello, World!
GermanApp.run();    // Hallo Welt!
```

`EnglishApp` and `GermanApp` are **environmental contexts**: fieldless types whose only job is to
carry the choice, as `HelloWorld<WriteToStdout, EnglishMessage>` is a type whose only job is to fix
the policies. Both compositions resolve at compile time, both monomorphize `run` per host, and both
emit a direct call to the chosen `message` and `write`. The difference is in the declarations
around them. `HasMessage` and `CanWrite` are the policy interfaces written down, and
`#[uses(HasMessage, CanWrite)]` is the host stating which policies its body relies on.

### Policies as type parameters are higher-order providers

The wiring table is not the only place CGP can put a policy choice. C++ passes policies as template
parameters of the host, and CGP has the same form in the
[higher-order provider](/docs/concepts/higher-order-providers): a provider whose type parameters are
other providers, bound with `#[use_provider]`. The `HelloWorld` host template translates almost
token for token:

```rust
#[cgp_component(Runner)]
pub trait CanRun {
    fn run(&self);
}

#[cgp_impl(new HelloWorld<W, M>)]
#[use_provider(W: Writer)]
#[use_provider(M: MessageProvider)]
impl<W, M> Runner {
    fn run(&self) {
        W::write(self, M::message(self))
    }
}

pub struct App;

delegate_components! {
    App {
        RunnerComponent: HelloWorld<WriteToStdout, GermanMessage>,
    }
}

App.run();   // Hallo Welt!
```

The wiring entry names the C++ instantiation as a Rust type: `HelloWorld<WriteToStdout, GermanMessage>`
on both sides. The `#[use_provider(W: Writer)]` bound is the concept the C++ version lacks, spelled
out: `W` must implement the `Writer` provider trait for this context, and the body calls
`W::write(self, ...)` as an associated function. The same shape works on a function, which is the
closer reading of a C++ function template that takes policy types. A
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) may be generic over providers, and the caller
instantiates it the way C++ instantiates a template:

```rust
#[cgp_fn]
#[use_provider(W: Writer)]
#[use_provider(M: MessageProvider)]
pub fn hello<W, M>(&self) {
    W::write(self, M::message(self))
}

<App as Hello<WriteToStdout, EnglishMessage>>::hello(&App);   // Hello, World!
```

CGP therefore offers both placements. Passing policies as type parameters keeps the choice at the
instantiation site and repeats it wherever the type is named, which is the C++ arrangement with its
costs and its flexibility. Wiring the policies as components on the context names each choice once
and lets generic code require only the traits it uses. A higher-order provider may also default its
inner parameter to [`UseContext`](/docs/reference/providers/use_context), which routes the inner
call back to whatever the context wires for that component. That is a default template argument
whose default is "whatever the host is wired with", and it has no C++ counterpart.

### `#[cgp_impl]` is CRTP with the cast done for you

A CGP provider's body refers to the context as `self` and `Self`. CRTP arranges the same thing for a
base class through `static_cast<Derived*>(this)`. The [`#[cgp_impl]`](/docs/reference/macros/cgp_impl)
macro rewrites a provider written in consumer-trait shape into a provider-trait impl whose `Self` is
the provider's own marker and whose context is an explicit parameter, so the `self` a provider body
uses is the context. A policy that needs the host's data reads it as an
[implicit argument](/docs/concepts/implicit-arguments):

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

In CRTP terms, `GreetHello` is the base template, the context is `Derived`, and the `#[implicit]`
argument is `static_cast<Derived*>(this)->name` with the cast replaced by a `HasField` bound the
compiler checks. CGP never has the CRTP hazard of passing the wrong derived class, because the
context is supplied by the wiring rather than named at each derivation. The `Person` this provider is
wired on is a **value context**: the type being greeted also carries the wiring.

### Checked at definition, not at instantiation

The deepest difference is when a mistake surfaces. A C++ host body is checked when it is instantiated
with concrete policies, and even with concepts the body is not verified against the concept at
definition. A CGP provider is checked when it is written. The `#[uses(HasMessage, CanWrite)]` bounds
are the whole contract `run` may rely on, and a call to a method outside them is an error at `run`'s
definition, for every context at once. The instantiation-time question that remains, whether a
particular context supplies what its providers need, is answered by
[`check_components!`](/docs/reference/macros/check_components) at the wiring site, with the missing
dependency named rather than inside a monomorphized body. The
[type classes](./type-classes.md) and [reflection](./reflection.md) pages draw out the same
property against Haskell's neighbours and Zig's `comptime`.

### One wired context instead of a repeated parameter list

A policy-based host carries its policies in its type, so every place that names the host names the
policies: `SmartPtr<Widget, RefCounted, NoChecking, DefaultStorage>` appears wherever such a pointer
is declared, and a helper generic over the host repeats the parameter list. CGP can reproduce that
arrangement with a higher-order provider, and it carries the same cost there. The alternative CGP adds
is to wire the policies as components on the context, so the choices live in one
`delegate_components!` table on a context type named once, and code generic over the context requires
only the traits it uses through `#[uses]`. Adding a policy to a host is then a new wiring line rather
than a new template parameter threaded through every signature that mentions the host. The
[dependency injection](./dependency-injection.md) page develops this centralization from the
container side.

## What each approach costs

Policy-based design's costs are the familiar costs of C++ templates, as its users state them. The
policy interface is implicit and must be documented by hand, so a wrong policy fails deep inside the
host with an error about substituted types
([Wikipedia, *Policy-based design*](https://en.wikipedia.org/wiki/Policy-based_design)). Long
parameter lists spread through every signature that names a policy-heavy host. Every combination of
policies is a distinct type, so code generic over the host must itself be a template, and
heterogeneous collections need a separate virtual interface. CRTP adds the derived class passed to
the wrong base and a base that cannot see members of a still-incomplete derived class
([Wikipedia, *CRTP*](https://en.wikipedia.org/wiki/Curiously_recurring_template_pattern)). Concepts
improve diagnostics but leave template bodies checked only at instantiation
([cppreference](https://en.cppreference.com/w/cpp/language/constraints)). Compile times grow with the
number of instantiations, which is a cost CGP's monomorphized wiring shares.

CGP's costs are of the same species. A provider needs a component to implement, so CGP cannot accept
an arbitrary existing type as a policy the way a template accepts any class with the right members.
Its type-level programming is narrower than template metaprogramming, and its policies cannot
contribute data members to the host by inheritance. The wiring table is more ceremony than a template
argument list for a host with one or two policies. And its raw diagnostics are trait-solver output
over generated types, which a C++ programmer will recognize as the same species as a template error:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where a template is the better choice

Where a class has a few orthogonal policies and its users name the instantiation in one place, a
policy-based template is the smaller tool. Where a policy must contribute data members or types to the
host by inheritance, or where the composition needs value-level template metaprogramming, templates
express what CGP's type level cannot. Where any existing class must be accepted as a policy with no
declaration, structural composition is the requirement and CGP's declared provider traits are in the
way. And where a program needs a heterogeneous collection of hosts with different policies, both
systems fall back to runtime dispatch: virtual functions in C++, `dyn Trait` in Rust, as the
[dynamic dispatch](./dynamic-dispatch.md) page describes.

## What to expect that differs

**A provider needs an impl.** A reader used to dropping an existing class in as a policy will find
that CGP requires a provider impl for a declared component. The declaration lets the compiler check
the body once, at its definition, for every context.

**CGP's type level is types and associated types only.** There is no value-level computation and no
data-member contribution from a provider. CGP is composition of behavior and of abstract types, not a
metaprogramming language.

**The common form is the wiring table, not the parameter list.** A reader may expect to name policies
at every use, as a template argument list does. CGP can, through a higher-order provider, but the
idiomatic form names each choice once on the context, and a provider that must pin an inner choice
locally is the exception.

**CGP is not templates done right.** Templates are the more general mechanism. CGP is policy-based
design with declared interfaces, definition-time checking, and a centralized wiring table, on a
language whose trait system already does the checking templates leave to instantiation.

## Where to go next

- [Higher-order providers](/docs/concepts/higher-order-providers): the construct this page maps onto
  a host template, including the `UseContext` default.
- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): what the declared
  policy interface is made of.
- [Dependency injection](./dependency-injection.md): the same centralization of choices, seen from
  the container side.
- [Checking your wiring](/docs/concepts/check-traits): how the instantiation-time question is
  answered at the wiring site.

## Sources

The C++ snippets are the reference examples from Wikipedia and cppreference, compiled with GCC 15.3
in C++23 mode. The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!`
assertion per wired context.

- [Wikipedia, *Modern C++ Design*](https://en.wikipedia.org/wiki/Modern_C%2B%2B_Design): Alexandrescu's book, the policy and host-class vocabulary, and the exponential-combinations argument.
- [Wikipedia, *Policy-based design*](https://en.wikipedia.org/wiki/Policy-based_design): the `HelloWorld` example, policies as a compile-time strategy pattern, and the statement that the policy interface is implicit.
- [Wikipedia, *Curiously recurring template pattern*](https://en.wikipedia.org/wiki/Curiously_recurring_template_pattern) and [cppreference, *CRTP*](https://en.cppreference.com/w/cpp/language/crtp): the pattern's mechanism, static polymorphism, the C++23 deducing-`this` alternative, and the homogeneous-container limit.
- [cppreference, *Constraints and concepts*](https://en.cppreference.com/w/cpp/language/constraints): concept syntax, satisfaction checked at instantiation, and the improvement in diagnostics.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
