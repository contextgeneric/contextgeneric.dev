---
sidebar_label: 'Algebraic effects'
sidebar_position: 6
description: 'Compare CGP providers with algebraic effect handlers: shared operation interfaces, static selection, and the continuation boundary.'
---

# Algebraic effects and handlers

CGP separates operations from their implementations, much as algebraic effects separate operations
from handlers, but CGP does not capture continuations. It is a language extension for Rust, with
pluggable trait implementations at compile-time, implemented as a library on stable Rust whose
consumer traits are ordinary Rust traits; the [Introduction](/docs/) covers the basics. For readers
familiar with Koka, OCaml 5, Flix, or Eff, this page develops the correspondence with handlers that
resume in place, its limits, and the costs of each approach.

## In your terms

A **context** is the type a CGP method runs on, supplying data through fields and implementations
through wiring. In these examples it represents an application and selects the implementations its
operations use. This plays part of a handler environment's role, but the choices are attached to a
Rust type rather than installed on the runtime call stack.

| In an effect system | In CGP |
| --- | --- |
| An effect signature | A **component**, with a consumer trait declaring operations |
| An operation | A method of the consumer trait |
| A handler interpreting an operation | A **provider** implementing the corresponding method |
| Installing a handler | **Wiring** a provider on a context |
| Requirements recorded in an effect row | **[Impl-side dependencies](/docs/reference/glossary#impl-side-dependency)**, with a narrower guarantee than effect typing |
| Checking that required operations have handlers | `check_components!` verifies declared provider dependencies |
| A captured continuation | Without a direct counterpart; provider methods use ordinary Rust control flow |

## The idea, briefly

Algebraic effects let a computation request an operation without choosing its implementation.
A handler supplies the operation's meaning and can control how the computation continues. This lets
the same computation use different implementations of configuration access, logging, state, or
more general control flow.

### Operations, handlers, and the continuation

An effect signature declares operations, while a handler interprets them. In the algebraic account,
equations describe relationships between operations; the resulting theory connects effects to
monads. [Plotkin and Pretnar](https://homepages.inf.ed.ac.uk/gdp/publications/handling-algebraic-effects.pdf)
develop this account and the interpretation of operations by handlers.

A general effect handler receives the continuation: the suspended remainder of the computation.
It can decide whether and when to resume that computation, subject to the language's restrictions.
[Pretnar's tutorial](https://www.eff-lang.org/handlers-tutorial.pdf) explains the main possibilities:

- **Discard it:** abort the suspended computation, as an exception handler does.
- **Resume it once:** continue with a supplied result, either immediately or after suspension.
- **Resume it more than once:** explore alternatives, as in nondeterministic search.

Resuming once does not necessarily mean resuming in place. A scheduler can store a continuation and
resume it later, and a generator can suspend before its next resumption. CGP's closest correspondence
is narrower: an operation completes and its caller continues through an ordinary return.

### Effect typing

Effect typing records which effects a computation may perform, but languages differ in whether
they provide it. Koka uses effect rows; Flix uses effect sets and supports purity reflection.
These systems can check whether a computation's effects are handled. See
[Leijen's Koka account](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v2.pdf)
and [Madsen et al. on Flix](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2023.18).

OCaml 5 and Eff provide handlers without statically tracking all performed effects in function
types. In OCaml, an unhandled effect raises `Effect.Unhandled` at runtime. The
[OCaml manual](https://ocaml.org/manual/5.5/effects.html) states this limit, while the
[row polymorphism](./row-polymorphism.md) comparison explains the row machinery used by Koka.

### The same effect in three languages

Koka distinguishes operations by their control-flow behavior. A `fun` operation resumes once in
place; `ctl` allows general resumption. This `ask` handler supplies a value:

```koka
effect fun ask() : int      // one `fun` operation; the effect is also named `ask`

fun add-twice() : ask int
  ask() + ask()

fun main() : console ()
  with fun ask() 21         // install a handler that supplies 21
  println( add-twice() )    // 42
```

A Koka `ctl` handler can resume more than once. The following handler explores both results of a
Boolean choice and concatenates the outcomes:

```koka
effect ctl choice() : bool

fun choice-all(action : () -> <choice|e> a) : e list<a>
  with handler
    return(x)    [x]
    ctl choice() resume(False) ++ resume(True)
  action()
```

The [Koka book](https://koka-lang.github.io/koka/doc/book.html#sec-opfun) explains why `fun` operations
behave like ordinary calls and how `ctl` provides additional control over resumption.

OCaml 5 declares operations by extending `Effect.t` and performs them with `perform`. An effect
pattern binds a continuation that the handler can resume. This syntax requires OCaml 5.3 or later;
earlier code uses functions such as `Effect.Deep.try_with`:

```ocaml
open Effect

type _ Effect.t += Ask : int Effect.t

let add_twice () = perform Ask + perform Ask

let () =
  let result =
    match add_twice () with
    | v -> v
    | effect Ask, k -> Effect.Deep.continue k 21
  in
  print_int result   (* 42 *)
```

OCaml continuations are one-shot: they cannot be resumed repeatedly to explore alternatives.
They can still be suspended and resumed later, which is sufficient for uses such as cooperative
scheduling. The [manual](https://ocaml.org/manual/5.5/effects.html) also explains the need to resume
or discontinue a captured continuation to release its resources.

Flix declares an effect with `eff` and records it after a backslash in a function's type. A handler
supplies the result and resumes the continuation:

```flix
eff Ask {
    def ask(): Int32
}

def addTwice(): Int32 \ Ask =
    Ask.ask() + Ask.ask()

def main(): Unit \ IO =
    let result = run {
        addTwice()
    } with handler Ask {
        def ask(k) = k(21)
    };
    println(result)   // 42
```

`addTwice` performs `Ask`, while the handler in `main` handles that effect. Flix supports multiple
resumptions as well as the single resumption shown here. See the
[Flix documentation](https://doc.flix.dev/effects-and-handlers.html).

### The fragment CGP corresponds to

Tail-resumptive handlers provide the closest comparison to CGP. They resume exactly once in tail
position, so the operation can run in place without capturing the rest of the computation.
[Xie et al.](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/07/evidently-5f0b7dbc1a998.pdf)
identify dynamic binding as the canonical example. Evidence-passing implementations can make
handler selection efficient by passing evidence of the selected handler, as developed in
[Generalized Evidence Passing](https://xnning.github.io/papers/multip.pdf).

CGP uses a related separation of operation and implementation, with selection resolved through
Rust traits. This is a comparison of programming structure, not a claim that CGP implements the
same effect calculus or dynamic scoping rules. The [capabilities](./capabilities.md) page explores
the related view of computations declaring what they require from an environment.

## How CGP expresses it

CGP represents an operation with a consumer trait and its implementation with a provider.
The examples use [environmental contexts](/docs/reference/glossary#environmental-context): types representing applications and supplying the
implementations for [self-targeted](/docs/reference/glossary#self-targeted-component) components. Supporting declarations and imports are omitted
where they do not affect the comparison.

### Components declare operations; providers implement them

A component declares an operation independently of its implementation. A context then selects a
provider through wiring:

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {
        println!("Hello!");
    }
}

delegate_components! { App { GreeterComponent: GreetHello } }
```

`CanGreet` declares `greet`, `GreetHello` implements it, and `App` selects that implementation.
The provider receives the context rather than a continuation. If the method returns normally,
its caller continues once at the call site. Like any Rust function, the method can also panic or
diverge; CGP does not enforce termination or an exactly-once return guarantee.

### Context fields supply an environment value

CGP's [implicit arguments](/docs/concepts/implicit-arguments) let nested implementations read
values from a shared context. This resembles a reader effect whose handler supplies an environment
value:

```rust
#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) -> String {
    format!("Hello, {name}!")
}
```

The `name` argument comes from the context's field. Koka's `ask` example supplies a value through
an installed handler; CGP supplies it through `self`. The similarity is that intermediate callers
do not forward the individual value. The difference is selection: a CGP field dependency is
resolved against a context type, without dynamic handler installation. The
[implicit parameters](./implicit-parameters.md) comparison develops that distinction.

### Raising an error constructs a value

CGP's error components choose how to construct an error, while Rust's `Result` controls propagation.
This provider can produce an error without fixing the context's concrete error type:

```rust
#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self, path: &str) -> Result<String, Error> {
        if path.is_empty() {
            return Err(Self::raise_error("empty path".to_owned()));
        }
        Ok(format!("contents of {path}"))
    }
}
```

The context chooses both the error type and the provider used for each source error type:

```rust
delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        LoaderComponent: LoadOrFail,

        @ErrorRaiserComponent.String: RaiseFrom,
    }
}
```

`Self::raise_error(...)` returns an error value; the surrounding `return Err(...)` exits `load`.
A caller can then propagate that result with `?`. An aborting effect handler instead abandons the
captured continuation. CGP's wiring supplies error-construction behavior without adding that
control-flow mechanism. [Modular error handling](/docs/concepts/modular-error-handling) explains
the error components in detail.

### Dependency checks verify the selected providers

A provider's impl-side dependencies state the traits and fields it needs from a context.
For example, `LoadOrFail` requires `CanRaiseError<String>`. A check verifies those requirements
transitively for the selected implementation:

```rust
check_components! {
    App {
        LoaderComponent,
    }
}
```

The compiler rejects this check if `App` cannot satisfy a required dependency. This resembles
checking that required operations have implementations, but it is not an effect-row check.
CGP does not track every effect a provider body can perform: the body may still print, mutate
state, or panic through ordinary Rust APIs. The
[impl-side dependencies](/docs/concepts/impl-side-dependencies) page explains the guarantee.

### Configuring abstract types through the same wiring

CGP wiring can select associated types as well as operation implementations. This independent
configuration chooses the context's error type:

```rust
use cgp::core::error::ErrorTypeProviderComponent;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<anyhow::Error>,
    }
}
```

The same table can therefore select error-construction behavior and the concrete `Error` type.
This follows from Rust's associated types and CGP's
[abstract-type components](/docs/concepts/abstract-types). It is an additional use of the wiring
mechanism, separate from the comparison with handlers interpreting operations.

### A related theoretical view: coeffects

Coeffects offer another way to describe CGP's declared context dependencies. An effect describes
what a computation does; a coeffect describes what it requires from its environment.
[Petricek's coeffects work](https://tomasp.net/coeffects/) develops that distinction. CGP's field
and trait requirements suggest such a comparison, but CGP does not implement a coeffect calculus.

## What each approach costs

Effect handlers separate operation use from interpretation and support reusable control-flow
abstractions. Their flexibility also makes control flow less local: understanding a `perform` may
require locating the handler and following its resumption behavior. The
[Ante language's account](https://antelang.org/blog/why_effects/) discusses both the modularity
benefit and the difficulty of following effects across a program.

Handler costs depend on the language and the resumption pattern. General handlers may capture
continuations, while tail-resumptive handlers admit simpler compilation. The evidence-passing
research documents those implementation choices and their performance trade-offs
([Xie and Leijen](https://xnning.github.io/papers/multip.pdf)). OCaml additionally restricts
continuations to one use and reports unhandled effects at runtime. Effect-typed languages move
that handling check into the type system.

CGP requires component declarations, wiring, and compile-time trait resolution. It also requires
readers to learn the consumer/provider split. Its raw diagnostics expose generated types:
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
and the tool is a v0.1.0-alpha that does not yet reshape every class. CGP does not provide
continuation handling, effect typing, or checked algebraic laws for its operations. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) weighs its machinery against simpler
Rust abstractions.

## Where effect handlers are the better choice

Effect handlers fit abstractions that need access to the suspended computation, such as a
scheduler, generator, or backtracking interpreter. The chosen language must support the required
resumption pattern; one-shot handlers do not provide multi-shot search directly. CGP wiring alone
cannot supply this control over a continuation.

CGP can still participate in asynchronous Rust programs. Its
[async handler family](/docs/concepts/handlers) uses Rust futures and `async`/`await`, and its
[type-level DSLs](/docs/concepts/type-level-dsls) select interpreters through providers. These
constructs compose ordinary Rust computations; they do not add general algebraic effect handlers.

## What to expect that differs

Provider calls use ordinary Rust control flow. A provider receives its context rather than a captured continuation it could
store or resume repeatedly. Normal return resumes the caller in place, while panic and divergence
retain their usual Rust meanings.

The context type determines provider selection. CGP does not choose the nearest handler on the
runtime stack. Different context values may carry different data while sharing the same wiring.

A wiring table does not impose a handler-stack order. Each entry selects a provider for its key.
Provider composition can still be order-sensitive: a wrapper may act before or after an inner
provider. Static selection does not make arbitrary operations commute.

Dependency checks cover declared requirements rather than every effect in a method body.
They establish that the selected providers can be used with a context. They do not establish
purity or confinement, and they do not turn CGP into an effect system.

## Where to go next

These pages expand the CGP mechanisms and nearby comparisons:

- [Modular error handling](/docs/concepts/modular-error-handling): selecting error types and
  error-construction behavior.
- [Implicit arguments](/docs/concepts/implicit-arguments): reading values from a context.
- [Handlers](/docs/concepts/handlers): CGP's computation family and its async forms.
- [Capabilities](./capabilities.md): requirements on an environment and the limits of that analogy.
- [Row polymorphism](./row-polymorphism.md): the row types used by effect-typed languages.

## Sources

The Koka `ask` snippet was compiled with Koka 3.2.3, and the `choice` handler follows the Koka book's
own example; the OCaml snippet was compiled with OCaml 5.5.0 and the Flix snippet run with Flix
0.76.0. The CGP snippets were compiled against `cgp` `0.8.0-alpha` with a `check_components!`
assertion per wired context.

- [Plotkin & Pretnar, *Handling Algebraic Effects* (LMCS 2013)](https://homepages.inf.ed.ac.uk/gdp/publications/handling-algebraic-effects.pdf) and [*Handlers of Algebraic Effects* (ESOP 2009)](https://homepages.inf.ed.ac.uk/gdp/publications/Effect_Handlers.pdf): effects as operations with an equational theory and handlers interpreting them through the continuation.
- [Pretnar, *An Introduction to Algebraic Effects and Handlers*](https://www.eff-lang.org/handlers-tutorial.pdf): the tutorial account and the Eff language.
- [Leijen, *Algebraic Effects for Functional Programming*](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v2.pdf) and [the Koka book](https://koka-lang.github.io/koka/doc/book.html): row-typed effects, the `ctl`/`fun`/`val` kinds, and evidence passing for `fun` operations.
- [Xie et al., *Effect Handlers, Evidently* (ICFP 2020)](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/07/evidently-5f0b7dbc1a998.pdf) and [Xie & Leijen, *Generalized Evidence Passing for Effect Handlers* (ICFP 2021)](https://xnning.github.io/papers/multip.pdf): the tail-resumptive handler as dynamic binding, and its dictionary-passing compilation.
- [OCaml manual, *Effect handlers* (5.5)](https://ocaml.org/manual/5.5/effects.html) and [Sivaramakrishnan et al., *Retrofitting Effect Handlers onto OCaml*](https://kcsrk.info/slides/handlers_edinburgh.pdf): the 5.3 syntax, one-shot continuations, and the absence of effect typing.
- [Flix, *Effects and Handlers*](https://doc.flix.dev/effects-and-handlers.html) and [Madsen et al., *Programming with Purity Reflection* (ECOOP 2023)](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2023.18): the set-based effect system and purity reflection.
- [Kammar, Lindley & Oury, *Handlers in Action* (ICFP 2013)](https://denotational.co.uk/publications/kammar-lindley-oury-handlers-in-action.pdf): handler composition and the significance of handler order.
- [Petricek, *Coeffects*](https://tomasp.net/coeffects/): the framework describing what a computation requires from its environment.
- [Abramov, *Algebraic Effects for the Rest of Us*](https://overreacted.io/algebraic-effects-for-the-rest-of-us/) and [*Why Algebraic Effects?* (Ante)](https://antelang.org/blog/why_effects/): community accounts of what users value and dislike.
- [`fused-effects`](https://hackage.haskell.org/package/fused-effects): the type-class-based effect libraries and the n²-instances problem.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
