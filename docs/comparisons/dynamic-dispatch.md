---
sidebar_label: 'Dynamic dispatch'
sidebar_position: 9
description: 'CGP read against dynamic dispatch, duck typing, vtables, and prototypal inheritance: the same openness, resolved at compile time.'
---

# Dynamic dispatch, dynamic typing, and prototypal inheritance

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who thinks in
objects, messages, vtables, and prototypes: Python, Ruby, JavaScript, Smalltalk, or the virtual
dispatch of C++ and Java. CGP reproduces the openness those mechanisms buy, many implementations
behind one interface, behavior assembled by delegation, defaults inherited from a shared table, and
provider code that reads as if it sends messages to an object, and resolves every bit of it at
compile time into direct calls. The page covers the correspondence, the one change from which the
rest follows, where runtime dispatch remains the right tool, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. It plays the object's part.

| In a dynamic language | In CGP |
| --- | --- |
| An object | The context |
| A message or method | A **component**: one trait with many possible implementations |
| A method implementation | A **provider** |
| The object's method table or vtable | The **wiring table**, written with `delegate_components!` and erased before runtime |
| A prototype the object delegates to | An aggregate provider or a namespace |
| `respond_to?` before a call | `check_components!`, made total and moved to compile time |

## The idea, briefly

Dynamic dispatch decouples a call site from the implementation it invokes, so one piece of code works
over many implementations chosen later. Dynamically typed languages take this to its limit: a value is
whatever it can *do*, behavior is shared by pointing one object at another, and the program is
malleable at runtime.

### Dynamic typing and duck typing

A dynamically typed language checks types at runtime, and *duck typing* is the style this permits: an
object's usability is decided by the methods it has, not by a class it declares. A function that
calls `x.quack()` works for any `x` that responds to `quack`
([Wikipedia, *Duck typing*](https://en.wikipedia.org/wiki/Duck_typing)). Its appeal is that an
interface need never be spelled out.

### Dynamic dispatch and late binding

Dynamic dispatch is the runtime selection of which implementation a method call invokes, based on the
receiver ([Wikipedia, *Dynamic dispatch*](https://en.wikipedia.org/wiki/Dynamic_dispatch)). Smalltalk
gave the purest form: every call is a message send, resolved at call time by consulting the receiver's
class method dictionary. Statically typed object languages offer the same late binding for methods
marked virtual, and a few languages (CLOS, Julia) generalize to *multiple dispatch* on several
arguments.

### Vtables and method dictionaries

Compiled languages implement dynamic dispatch with a *virtual method table*: a per-class array of
function pointers that each object reaches through a hidden pointer. A virtual call loads the vtable
pointer, indexes to the method's slot, and calls the function found there
([Wikipedia, *Virtual method table*](https://en.wikipedia.org/wiki/Virtual_method_table)). Rust's own
dynamic dispatch works this way: a `dyn Trait` value is a fat pointer pairing a data pointer with a
vtable pointer ([The Rust Book, *Trait objects*](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)).
The indirect call defeats inlining and adds a load per dispatch. Dynamically typed languages pay more,
resolving a method by name up a class or prototype chain, which is why their engines invest in inline
caches and hidden classes ([V8, *Maps*](https://v8.dev/docs/hidden-classes)).

### Prototypal inheritance and delegation

Prototype-based languages share behavior by *delegation*: an object holds a link to a prototype, and a
message the object does not handle is forwarded along the chain. Lieberman introduced the model in
1986 and the Self language realized it
([Lieberman, *Using Prototypical Objects*](https://web.media.mit.edu/~lieber/Lieberary/OOP/Delegation/Delegation.html)).
In JavaScript every object has a `[[Prototype]]` link, a lookup walks the chain, and an own property
*shadows* an inherited one ([MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Inheritance_and_the_prototype_chain)).

The property that distinguishes delegation from mere forwarding matters most for CGP: `self` stays
bound to the original receiver. When a prototype's method refers to `self`, it denotes the object that
originally received the message, so an inherited method still sees the receiver's own state.
Forwarding rebinds `self` to the object the message was passed to and loses that connection.

## How CGP expresses it

CGP reproduces dynamic dispatch, vtables, and prototypal delegation, and resolves each at compile
time. What is a runtime lookup in a dynamic language is a type-resolution step in CGP that
monomorphizes to a direct call.

### CGP code reads like a duck-typed program

A CGP provider is written against an unknown context and reads like duck-typed code that sends
messages to a receiver and trusts it to respond:

```rust
#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

The body `self.name()` is a message send to a context whose type is not written down. With an
[implicit argument](/docs/concepts/implicit-arguments) the resemblance is stronger still: a value
appears from the context, as an unbound name would in Ruby or Python:

```rust
#[cgp_fn]
pub fn greet_implicitly(&self, #[implicit] name: &str) -> String {
    format!("Hello, {name}!")
}
```

Ordinary generic Rust would demand `fn greet<C: HasName>(c: &C)`, naming the trait in the signature.
CGP moves the bound into `#[uses]` and abstracts the context away, so the provider body carries less
visible type ceremony than a plain generic function. The decisive difference is *when* the trust is
discharged. A duck-typed program finds out at runtime whether the object responds, and fails with a
`NoMethodError` if not. CGP finds out at compile time, because trait resolution and
[`check_components!`](/docs/reference/macros/check_components) check the `#[uses(HasName)]`
dependency and the field access. The `Person` these run on is a **value context**: the type being
greeted carries the `name` field and the wiring.

### Static dispatch with the flexibility of dynamic dispatch

A caller writes `context.area()` against the `CanCalculateArea` consumer trait without naming an
implementation, as a dynamic call names a method and lets the receiver decide. In CGP the receiver
decides during type checking: the consumer trait's generated impl routes the call through the
context's wiring table to the provider, and the compiler monomorphizes the whole route to a direct
call, with no fat pointer and no vtable load. Rust already offers real dynamic dispatch through
`dyn Trait`, and CGP is its static sibling: both let an implementation be chosen after the calling
code is written, and one pays a runtime indirection while the other resolves it away. Where a
component carries a generic parameter, the [`open` statement](/docs/reference/macros/delegate_components)
selects a provider per value of that parameter, which reproduces *multiple* dispatch, decided at
compile time.

### `DelegateComponent` is a compile-time vtable

The type-level table a context carries is a vtable that exists only during compilation. Each entry
maps one component key to the provider that implements it, as a vtable slot maps a method to its
implementation:

```rust
delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

// expands to the type-level table entry:
// impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
//     type Delegate = RectangleArea;
// }
```

The mapping is exact on structure and opposite on timing. The context type plays the class, a
`DelegateComponent` impl is a slot, and the provider is the function the slot points to. But the key
is a type rather than an offset, the compiler performs the lookup once rather than the CPU on every
call, and the table is erased before the program runs. The honest cost is that CGP loses the runtime
heterogeneity a real vtable enables: a `Vec<Box<dyn CanCalculateArea>>` can hold different shapes and
dispatch each at runtime, whereas a CGP context is one monomorphic type resolved once. The
[Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) page traces a call
through the table.

### Component delegation is delegation, with `self` bound

CGP's delegation chain is delegation in Lieberman's sense, because the context stays bound to the
original as lookup walks the chain. An [aggregate provider](/docs/concepts/aggregate-providers)
bundles a group of wirings, and a context delegates a whole group to it in one entry:

```rust
delegate_components! {
    new GeometryComponents {
        AreaCalculatorComponent: RectangleArea,
        PerimeterCalculatorComponent: RectanglePerimeter,
    }
}

delegate_components! {
    Rectangle {
        [AreaCalculatorComponent, PerimeterCalculatorComponent]: GeometryComponents,
    }
}
```

When `rect.area()` resolves, the lookup walks `Rectangle`, then `GeometryComponents`, then
`RectangleArea`, and the context stays `Rectangle` at every step: `GeometryComponents` appears only in
the delegate position, never as the context, so the leaf provider reads `width` and `height` from
`Rectangle`. That is delegation's defining property. The delegate chain supplies the *behavior* while
`self` remains the original *identity*. The [`UseContext`](/docs/reference/providers/use_context)
provider is the same relationship pointed the other way, letting a provider route a call back to the
context's own wiring.

### Namespaces are shared prototypes with open slots, not shadowable ones

A CGP [namespace](/docs/concepts/namespaces) is a shared table of wirings that many contexts inherit,
which is the prototype's job. But the override rule differs from JavaScript's. In a prototype chain
an own property *shadows* an inherited one. In CGP, a key the namespace binds is not overridable:
joining a namespace generates a forwarding impl covering every key it answers, so a direct entry for
one of those keys is rejected with `E0119`. What a context may supply is a path the namespace routes
to but leaves unbound, an open slot left for each context to fill:

```rust
// The shared prototype: it binds the farewell and leaves the greeting path open.
cgp_namespace! {
    new AppDefaults: AppNamespace {
        @app.FarewellComponent: SayGoodbye,
    }
}

delegate_components! {
    AppA {
        namespace AppDefaults;         // inherit every wiring the namespace binds

        @app.GreeterComponent: GreetHello,   // fill a slot the namespace leaves open
    }
}
```

Namespaces inherit from one another, so a child may bind an open path the parent leaves; it may not
redefine a key the parent binds:

```rust
cgp_namespace! {
    new QuietDefaults: AppDefaults {
        @app.GreeterComponent: GreetQuietly,
    }
}

delegate_components! {
    AppB {
        namespace QuietDefaults;
    }
}
```

The lookup that walks these layers is the [`RedirectLookup`](/docs/reference/providers/redirect_lookup)
provider tracing a type-level path, which is the prototype chain being walked. The closer
object-oriented analogy is an abstract base class rather than a prototype: the namespace fixes what
every context agrees on and declares slots each context must fill, and it does not let a child
silently redefine a concrete inherited member. `AppA` and `AppB` are **environmental contexts**, types
standing for an application.

## What each approach costs

Dynamic dispatch and dynamic typing are valued for immediacy and flexibility. Duck typing lets a
function work over any object that responds to its messages, which programmers value for reuse,
shorter code, and rapid prototyping
([SitePoint, *Making Ruby Quack*](https://www.sitepoint.com/making-ruby-quack-why-we-love-duck-typing/)).
Prototypal inheritance lets objects be created, linked, and reshaped at runtime, and hooks such as
`method_missing` let one object answer messages it was never written to handle. Their costs, as their
users state them, follow from the same deferral. A mistake surfaces as a runtime error discovered only when the
offending line runs, which makes refactoring hazardous, and the standard mitigation is tests or a
`respond_to?` guard ([DevGex, *Duck Typing*](https://devgex.com/en/article/00035033)). The vtable
indirection defeats inlining, and dictionary-based lookup is worse, which is why so much engineering
goes into inline caches ([V8](https://v8.dev/docs/hidden-classes)). A mutable prototype chain is easy
to get confused about, and JavaScript's `this` binding, the same self-binding that makes delegation
work, is lost when a method is detached from its receiver and called on its own.

CGP's costs are the ones dynamic dispatch exists to avoid. It cannot hold a heterogeneous collection
of implementations and choose among them at runtime; Rust's `dyn Trait` exists for that. It
cannot load an implementation chosen at runtime from configuration, patch a live object, or answer a
message it was not built to handle, because there is no runtime object graph and no runtime dispatch
to intercept. Its wiring is code somebody writes. And its errors, though caught early, are trait-solver
output over generated types: [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause
for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class.
The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs these costs against the
alternatives.

## Where runtime dispatch is the better choice

When a program needs runtime openness, plugins discovered at startup, objects of mixed types in one
collection, behavior reshaped live, or the metaprogramming that proxies and DSLs rely on, dynamic
dispatch is the right tool, and emulating it with compile-time wiring is impossible rather than
awkward, since the whole point is deferral to runtime. In Rust that means `dyn Trait`, and a CGP
context may hold one. CGP is the better tool when the set of implementations is known at build time
and the program wants the decoupling of dynamic dispatch with none of the cost or the runtime failure
modes.

## What to expect that differs

**Nothing happens at runtime.** A reader from this background will assume a vtable is a data
structure the program carries, that dispatch chooses at the moment of the call, that a prototype chain
is walked when a property is missing, and that the object graph can change while the program runs. In
CGP the table is erased after compilation, the type checker resolves the dispatch and the compiler
inlines it, the chain is walked during type resolution, and the wiring is fixed once the program is
built.

**Late binding is late to the wiring site, not to runtime.** The flexibility is real, and it is spent
at compile time, at the place a context declares its providers.

**A missing method is a compile error.** A context that lacks a trait a provider needs fails at the
wiring site, named by `check_components!`, rather than as a `NoMethodError` in production.

**A namespace's bound entries cannot be shadowed.** The inherit-and-customize pattern works through
slots the namespace leaves open, not through override.

**CGP is not a dynamic language with the types added.** It is the static resolution of the mechanisms
this reader knows, trading runtime malleability for zero cost and compile-time safety.

## Where to go next

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): how a call finds its
  provider through the table.
- [Aggregate providers](/docs/concepts/aggregate-providers) and [Namespaces](/docs/concepts/namespaces):
  the delegation chain and the shared table.
- [Row polymorphism](./row-polymorphism.md): the type-system side of duck typing, structural versus
  nominal.
- [Reflection](./reflection.md): the introspection counterpart to this page's runtime mechanisms.

## Sources

The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per
wired context. This page shows no code in another language.

- [Wikipedia, *Dynamic dispatch*](https://en.wikipedia.org/wiki/Dynamic_dispatch), [*Virtual method table*](https://en.wikipedia.org/wiki/Virtual_method_table), and [*Duck typing*](https://en.wikipedia.org/wiki/Duck_typing): late binding, the vtable, and usability decided by the methods an object has.
- [The Rust Programming Language, *Trait objects*](https://doc.rust-lang.org/book/ch18-02-trait-objects.html) and [geo-ant, *Rust Dyn Trait Objects and Fat Pointers*](https://geo-ant.github.io/blog/2023/rust-dyn-trait-objects-fat-pointers/): Rust's own dynamic dispatch as a fat pointer and a vtable.
- [Lieberman, *Using Prototypical Objects to Implement Shared Behavior in Object-Oriented Systems* (OOPSLA 1986)](https://web.media.mit.edu/~lieber/Lieberary/OOP/Delegation/Delegation.html): delegation, and the rule that `self` stays bound to the original receiver.
- [MDN, *Inheritance and the prototype chain*](https://developer.mozilla.org/en-US/docs/JavaScript/Guide/Inheritance_and_the_prototype_chain): JavaScript's prototype chain and own-property shadowing.
- [V8, *Maps (Hidden Classes)*](https://v8.dev/docs/hidden-classes): the engineering that dynamic dispatch's runtime cost demands.
- [SitePoint, *Making Ruby Quack*](https://www.sitepoint.com/making-ruby-quack-why-we-love-duck-typing/), [GeeksforGeeks, *Type Systems*](https://www.geeksforgeeks.org/python/type-systemsdynamic-typing-static-typing-duck-typing/), and [DevGex, *Duck Typing*](https://devgex.com/en/article/00035033): community sentiment on what duck typing buys and costs.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
