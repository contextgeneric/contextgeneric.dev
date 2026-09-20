---
sidebar_label: 'Dynamic dispatch'
sidebar_position: 9
description: 'Compare CGP static wiring with dynamic dispatch, duck typing, and prototype delegation, including how trait objects can coexist with CGP.'
---

# Dynamic dispatch, dynamic typing, and prototypal inheritance

CGP supports interchangeable implementations and delegation while resolving provider selection at
compile time. It is a language extension for Rust, with pluggable trait implementations at
compile-time, implemented as a library on stable Rust whose consumer traits are ordinary Rust
traits; the [Introduction](/docs/) covers the basics. For readers familiar with objects, messages,
vtables, or prototypes, this page explains the shared structure, the limits of static selection,
and where runtime dispatch remains useful.

## In your terms

A **context** is the type a CGP method runs on, supplying data through fields and implementations
through wiring. It plays the receiver's role: a provider operates on that context even when several
layers of delegation lead to the provider.

| In object-oriented code | In CGP |
| --- | --- |
| An object receiving a method call | A context value |
| An interface declaring methods | A consumer trait within a **component** |
| An implementation of that interface | A **provider** |
| A method table or vtable | A **wiring table**, resolved by the compiler |
| Shared behavior reached through delegation | An aggregate provider or namespace |
| Checking that a receiver supports an operation | Trait bounds and `check_components!` |

## The idea, briefly

Dynamic dispatch lets a call site invoke an implementation selected at runtime. Dynamic typing and
prototypal inheritance address related but separate questions: when operations are type-checked,
and how objects share behavior. Keeping these distinctions clear makes it possible to compare
CGP with each mechanism without treating them as one feature.

### Dynamic typing and duck typing

Duck typing accepts an object according to the operations it supports. In a dynamically typed
language, a call such as `x.quack()` can work without a declared interface as long as the receiver
responds appropriately at runtime. This makes generic behavior convenient, but an unsupported
operation may fail only when the call executes. Python's
[glossary](https://docs.python.org/3/glossary.html#term-duck-typing) describes this style.

### Dynamic dispatch and late binding

Dynamic dispatch selects an implementation using runtime information about the receiver.
Smalltalk expresses calls as message sends; statically typed languages also support late binding
through virtual methods or trait objects. Multiple-dispatch systems extend the selection to more
than one argument. The timing of implementation selection is distinct from whether the language
checks the call's interface statically.

### Vtables and method dictionaries

A vtable stores the method implementations used for dynamic calls. Rust's trait-object pointers
pair a pointer to the value with a pointer to a vtable containing the relevant method pointers.
The call therefore follows an indirect route, as described in the
[Rust Reference](https://doc.rust-lang.org/reference/types/trait-object.html).

Indirect dispatch can limit optimization, but its cost depends on the call and compiler.
Devirtualization can recover a direct call when the target is known. Dynamic-language runtimes
also optimize property and method lookup; V8's hidden classes help it identify object layouts.
The [V8 documentation](https://v8.dev/docs/hidden-classes) describes that mechanism. A comparison
of dispatch mechanisms alone does not establish a performance ranking for whole programs.

### Prototypal inheritance and delegation

Prototypal inheritance shares behavior by linking objects. JavaScript property lookup follows an
object's prototype chain when the property is absent from the object itself, and an own property
shadows an inherited one. The
[MDN guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Inheritance_and_the_prototype_chain)
explains these rules.

Delegation preserves the original receiver while obtaining behavior elsewhere. An inherited method
can therefore read the receiving object's state. This is the property of
[Lieberman's delegation model](https://web.media.mit.edu/~lieber/Lieberary/OOP/Delegation/Delegation.html)
that matters for CGP: selecting another implementation does not replace the context it operates on.
JavaScript's actual `this` binding also depends on how the function is called.

## How CGP expresses it

CGP combines generic provider code with statically selected implementations. Its wiring resembles
a method table in purpose, but the table consists of trait impls and is resolved during compilation.
The examples below omit supporting declarations where the surrounding prose identifies their roles.

### Provider code calls methods on a generic receiver

A CGP provider can call a method on a context without naming that context's concrete type.
The trait dependency makes the required operation explicit:

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

`self.name()` calls the context's `HasName` method. An implicit argument provides a shorter form
when the implementation simply needs a field value:

```rust
#[cgp_fn]
pub fn greet_implicitly(&self, #[implicit] name: &str) -> String {
    format!("Hello, {name}!")
}
```

Both forms declare a dependency that Rust checks statically. The first requires `HasName`; the
second requires access to a field named `name`. The method body may resemble duck-typed code,
but it relies on declared traits and generated bounds. Ordinary generic Rust can express these
bounds too; CGP's macros supply the context parameter and supporting impls.

A `Person` type with the required field and wiring is a value context in this example.
It is the value being greeted, rather than an application environment selecting behavior for a
separate target. [Implicit arguments](/docs/concepts/implicit-arguments) explains the field-based form.

### Implementation choice is deferred to wiring

A generic caller can invoke `context.area()` through `CanCalculateArea` without choosing an
implementation. The context's wiring supplies that choice, and monomorphization resolves the route
to a static call. This allows provider selection after the generic caller has been written,
while keeping that selection fixed for each concrete context type.

Generic components can also select providers by their type parameters. The
[`open` statement](/docs/reference/macros/delegate_components) supports entries for different
parameter types, giving a form of type-directed selection across several inputs. This is a
compile-time comparison with multiple dispatch; it does not inspect the runtime types of objects.

### Wiring serves the selection role of a vtable

A wiring entry maps a component key to a provider. This resembles the mapping from a vtable slot
to an implementation, except that both the key and selected provider are Rust types:

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

`Rectangle` is a value context, and the self-targeted area component operates on that rectangle.
The compiler resolves `AreaCalculatorComponent` to `RectangleArea` through the shown trait impl.
The program does not store or consult this wiring table at runtime. Components can group several
methods, associated types, and consts, so an entry is not necessarily equivalent to one method slot.

Runtime heterogeneity remains available through ordinary Rust trait objects. If the consumer trait
is dyn-compatible, a collection such as `Vec<Box<dyn CanCalculateArea>>` can contain different
context types and dispatch to their consumer methods dynamically. Each concrete context can still
use CGP wiring internally. The [consumer/provider explanation](/docs/concepts/consumer-and-provider-traits)
traces that internal route.

### Delegation preserves the original context

An [aggregate provider](/docs/concepts/aggregate-providers) groups wiring entries for reuse.
A context can delegate several components to the aggregate:

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

An area call follows `Rectangle` to `GeometryComponents` and then `RectangleArea`, with `Rectangle`
as the context at every step. The area provider reads the rectangle's `width` and `height`;
`GeometryComponents` holds neither value. This preserves the receiver in the same sense as
prototypal delegation, with the route resolved statically. [`UseContext`](/docs/reference/providers/use_context)
allows a provider to call back through the context's consumer-trait implementation.

### Namespaces share bindings and leave paths for contexts to fill

A CGP [namespace](/docs/concepts/namespaces) shares wiring across contexts. Its inheritance differs
from JavaScript property shadowing: a bound key cannot be redefined by a conflicting impl.
A context can instead fill a path the namespace leaves unbound:

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

`AppDefaults` supplies the farewell provider, while `AppA` supplies the greeting provider at the
open path. A child namespace can fill that path for contexts that join it:

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

`AppA` and `AppB` are environmental contexts representing applications. Their providers are found
through [`RedirectLookup`](/docs/reference/providers/redirect_lookup), which follows type-level
paths. This resembles sharing defaults through a prototype, but customization fills unbound paths
rather than shadowing already-bound entries. An attempt to bind the same key again conflicts under
Rust's coherence rules.

## What each approach costs

Dynamic dispatch supports runtime choice, including mixed collections of implementations.
Dynamically typed languages also allow programs to accept objects without an explicit interface
declaration, and mutable object systems can change behavior while running. Those freedoms are
useful for interactive work and runtime extensibility.

Dynamic typing can defer unsupported-operation errors until execution. That cost does not apply
to every form of dynamic dispatch: Rust checks a trait object's interface at compile time.
Runtime lookup and indirect calls also have costs, though optimizations can reduce them.
Prototype mutation and call-dependent `this` binding add separate reasoning demands, as the
[MDN guide](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Inheritance_and_the_prototype_chain)
explains.

CGP moves implementation selection into compilation and requires declarations and wiring to express
it. The learning cost includes tracing consumer traits through provider tables; the compile-time
cost includes trait resolution and monomorphization. Static wiring itself cannot discover plugins,
replace a provider on a live value, or handle an undeclared operation. Those features need other
runtime mechanisms, which CGP code can use alongside its wiring.

CGP's raw diagnostics expose generated traits and types.
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) compares these costs with ordinary
traits, generics, and trait objects.

## Where runtime dispatch is the better choice

Runtime dispatch fits implementations chosen while the program runs, such as mixed collections or
runtime-selected services. Dynamic object systems additionally support changing object behavior
and intercepting otherwise unknown messages. Plugin loading requires an appropriate loading and
interface mechanism as well as dispatch; a vtable alone does not provide it.

Rust's `dyn Trait` is useful for runtime polymorphism and can coexist with CGP. A context may hold a
trait object, or a dyn-compatible consumer trait may be used as a trait object. CGP wiring fits
implementation choices known at build time, where reusable providers justify the additional tables.

## What to expect that differs

CGP resolves wiring during compilation, while provider bodies execute at runtime.
Static selection removes the wiring lookup from execution; it does not guarantee that every call
is inlined or that the provider's work is free.

The context type fixes its provider choices. Two values of that type can contain different state,
but changing their fields does not rewrite the type's wiring. Runtime variation must be expressed
through the selected implementation or another Rust dispatch mechanism.

A dependency check catches missing declared operations before execution.
`check_components!` forces validation at the check site; a wiring declaration alone is lazy.
This is a guarantee about declared dependencies, not about every possible runtime failure.

Namespaces preserve bindings that are already defined. Contexts and child namespaces customize
paths left unbound rather than shadowing inherited entries. This restriction follows from the
ordinary trait impls used to represent wiring.

## Where to go next

These pages explain the static mechanisms and related comparisons:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): how a call reaches
  its provider.
- [Aggregate providers](/docs/concepts/aggregate-providers) and [Namespaces](/docs/concepts/namespaces):
  sharing and following wiring tables.
- [Row polymorphism](./row-polymorphism.md): structural interfaces and field requirements.
- [Reflection](./reflection.md): inspecting data and types at compile time or runtime.

## Sources

The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!` assertion per
wired context. This page contains only Rust snippets.

- [The Rust Reference, *Trait object types*](https://doc.rust-lang.org/reference/types/trait-object.html): trait-object pointers and dynamic dispatch.
- [Python glossary, *Duck typing*](https://docs.python.org/3/glossary.html#term-duck-typing): using an object through the operations it supports.
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
