---
sidebar_label: 'Algebraic effects'
sidebar_position: 6
description: 'CGP read against the effect handlers of Koka, OCaml 5, Flix, and Eff: the exactly-once fragment it shares and the continuation power it does not.'
---

# Algebraic effects and handlers

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
algebraic effects and handlers from Koka, OCaml 5, Flix, or Eff. CGP shares the split between an
operation and its interpretation, and the idea of choosing the interpretation from the surroundings,
but it keeps only the fragment in which the continuation is used exactly once and in place. The
literature identifies that fragment as dynamic binding, and CGP resolves it statically per context
rather than dynamically down a call stack. The page covers the correspondence, the boundary, where
effect handlers remain the only tool, and what to expect that differs.

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. In most CGP code it is a type you define to stand for an application, and it plays the part
of the dynamic scope in which handlers are installed.

| In an effect system | In CGP |
| --- | --- |
| An effect signature | A **component**: one trait with many possible implementations |
| An operation | A method of that trait |
| A handler | A **provider**: a named implementation, always tail-resumptive |
| Installing a handler with `with` or `run ... with handler` | **Wiring** the provider on a context |
| The effect row a function performs | The provider's **impl-side dependencies**, declared with `#[uses]` |
| A type error for an unhandled effect | A `check_components!` failure at the wiring site |
| The continuation | Nothing: a provider returns exactly once |

## The idea, briefly

Algebraic effects let code perform an effect without fixing how the effect is carried out. A function
that reads configuration, logs, throws, or suspends normally commits to a concrete mechanism, and
that commitment leaks into its type. With algebraic effects the function *performs an operation*
named by an abstract effect, and a *handler* installed up the call stack gives the operation its
meaning, so the same code runs against a real logger, a test collector, or a no-op.

### Operations, handlers, and the continuation

An *effect* is a signature of *operations*, and a computation produces the effect by performing one
of them. Plotkin and Power founded the idea, and the account is *algebraic* because each effect came
with an equational theory whose free model induces the monad for that effect
([Plotkin & Pretnar, *Handling Algebraic Effects*](https://homepages.inf.ed.ac.uk/gdp/publications/handling-algebraic-effects.pdf)).
A *handler* interprets the operations, and its defining power is that it receives the *continuation*:
the suspended rest of the computation from the point where the operation was performed
([Pretnar, *An Introduction to Algebraic Effects and Handlers*](https://www.eff-lang.org/handlers-tutorial.pdf)).
An exception handler can only abandon the computation; an effect handler can resume it.

How many times a handler invokes the continuation determines the effect it realizes, and the three
cases mark where CGP can and cannot follow:

- **Zero times.** The handler discards the continuation. This is exception behavior.
- **Once.** The handler resumes with a result. This is the ordinary case: reading state, dynamic
  binding, logging, any call that yields a value and lets the caller carry on.
- **Many times.** The handler resumes more than once. This is nondeterminism, generators, and
  cooperative scheduling, and it cannot be expressed as an ordinary function return.

### Effect typing

Languages differ sharply in whether a computation's type records the effects it may perform. Koka
types effects as a *row*, so `<exn,div>` in a signature means the function may throw and diverge
([Leijen, *Algebraic Effects for Functional Programming*](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v2.pdf));
the [row polymorphism](./row-polymorphism.md) page covers the row machinery. Flix types effects as a
set and uses the result for purity reflection
([Madsen et al., *Programming with Purity Reflection*](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2023.18)).
Eff and OCaml 5 track no effects in types: the OCaml manual states that "the compiler does not
statically ensure that all the effects performed by the program are handled", and an unhandled effect
raises `Effect.Unhandled` at run time ([OCaml manual](https://ocaml.org/manual/5.5/effects.html)).

### The same effect in three languages

In **Koka**, an operation is declared `ctl` (may resume any number of times), `fun` (resumes exactly
once in place), or `val`. A handler is installed with `with`:

```koka
effect fun ask() : int      // one `fun` operation; the effect is also named `ask`

fun add-twice() : ask int
  ask() + ask()

fun main() : console ()
  with fun ask() 21         // install a handler that supplies 21
  println( add-twice() )    // 42
```

The Koka book describes `with fun ask() 21` as a "statically typed dynamic binding" and notes that a
`fun` operation "behaves just like a regular function without changing the control-flow"
([Koka book](https://koka-lang.github.io/koka/doc/book.html#sec-opfun)). A `ctl` operation is the
multi-shot form; the book's `choice` handler resumes twice to collect every outcome:

```koka
effect ctl choice() : bool

fun choice-all(action : () -> <choice|e> a) : e list<a>
  with handler
    return(x)    [x]
    ctl choice() resume(False) ++ resume(True)
  action()
```

In **OCaml 5**, an effect extends the type `Effect.t`, is performed with `perform`, and is handled
with a `match` whose `effect` cases bind the continuation `k`. The `effect` pattern syntax arrived in
OCaml 5.3; earlier code used `Effect.Deep.try_with`:

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

OCaml's continuations are one-shot: "every captured continuation must be resumed either with a
`continue` or `discontinue` exactly once" ([OCaml manual](https://ocaml.org/manual/5.5/effects.html)).

In **Flix**, an effect is declared with `eff`, appears in a function type after a backslash, and is
handled with `run ... with handler`:

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

`\ Ask` is part of `addTwice`'s type, and `main` must discharge it
([Flix documentation](https://doc.flix.dev/effects-and-handlers.html)).

### The fragment CGP corresponds to

A handler that resumes the continuation exactly once, in tail position, is dynamic binding. The
literature names its canonical example outright: **the canonical tail-resumptive handler is dynamic
binding** ([Xie et al., *Effect Handlers, Evidently*](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/07/evidently-5f0b7dbc1a998.pdf)).
Such a handler never needs to capture the continuation, so a compiler can run the operation in place
and replace the dynamic search for a handler with a constant-offset lookup into an *evidence vector*
passed down like a dictionary ([Xie & Leijen, *Generalized Evidence Passing*](https://xnning.github.io/papers/multip.pdf)).
The efficient compilation of the exactly-once fragment *is* dictionary passing, and that is the
fragment CGP occupies directly, with no dynamic search underneath. Effekt and Scala's capture checking
describe the same fragment as *effects as capabilities*; the [capabilities](./capabilities.md) page
places CGP against that reading.

## How CGP expresses it

CGP reproduces the operations-and-handlers structure with its consumer and provider split, and every
CGP "handler" is an ordinary function that returns once. The correspondence is exact for the
tail-resumptive case and stops wherever an effect would reach for the continuation. The contexts below
are **environmental contexts**: the `App` that installs a `Loader` or a `Greeter` stands for an
application, as a handler's dynamic scope does, and every component shown is self-targeted.

### Components are effect signatures; providers are tail-resumptive handlers

Declaring a component names an operation without giving it meaning, and a provider interprets it:

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

`CanGreet` is the effect signature, `greet` the operation, `GreetHello` the handler, and the wiring
line its installation. The provider's body receives no continuation and cannot choose whether to
resume: it computes a value and returns, and the caller resumes in place, exactly once. In Koka's
terms every CGP provider is a `fun` operation and never a `ctl` one.

### Reading from the context is dynamic binding, exactly

Koka's `with fun ask() 21` installs a tail-resumptive handler that supplies a value to deeply nested
code without threading it through every call. CGP's [implicit arguments](/docs/concepts/implicit-arguments)
do the same by reading a field from the context that every provider receives as `self`:

```rust
#[cgp_fn]
pub fn greet(&self, #[implicit] name: &str) -> String {
    format!("Hello, {name}!")
}
```

The context supplies `name`, not the caller, as the enclosing `ask` handler supplies Koka's `ask()`.
The equivalence is exact rather than an analogy: the reader effect is the canonical tail-resumptive
handler, and reading a context field is CGP's whole realization of it. The two part ways on how the
value is found. Koka searches the dynamic handler stack; CGP reads a field of a statically known
context. The [implicit parameters](./implicit-parameters.md) page covers the same pattern from
Scala's side.

### Raising an error looks like `raise` but passes a value, not control

CGP's error handling shows the boundary most sharply. `CanRaiseError<SourceError>` reads like the
`raise` operation, and a provider raises without knowing the concrete error type:

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

The raise components dispatch per source type, so a context routes each source error to a different
strategy, which reads like installing one exception handler per exception type:

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

The resemblance stops at *selection*. An effect `raise` is the zero-resume handler: it abandons the
continuation and unwinds to the handler. CGP's `raise_error` unwinds nothing. It constructs and
returns a value of the abstract `Error` type, and the abort is Rust's own `return` or `?`, outside
the wiring. CGP selects the interpretation of the error and leaves the control flow to `Result`. The
[Modular error handling](/docs/concepts/modular-error-handling) page develops the construct.

### Impl-side dependencies are the effect row; `check_components!` is "all effects handled"

A provider states its needs as [impl-side dependencies](/docs/concepts/impl-side-dependencies), such
as the `#[uses(CanRaiseError<String>)]` above. That list is CGP's counterpart of the effect row Koka
would infer for a function that performs `raise`, and a generic provider is effect-polymorphic in a
loose sense, since the context type variable stands for "whatever else this context can do".
[`check_components!`](/docs/reference/macros/check_components) is the discharge check:

```rust
check_components! {
    App {
        LoaderComponent,
    }
}
```

This asserts that `App` supplies every trait `LoadOrFail` transitively needs, and it fails to compile
naming the missing one if not. That is Koka's guarantee that an unhandled effect is a type error, and
the opposite of OCaml's runtime `Effect.Unhandled`.

### Configuring abstract types through the same wiring

CGP extends the operations-and-handlers wiring to something effect systems do not touch: abstract
*types*. Alongside choosing the provider for an operation, a context chooses the concrete type behind
an [abstract type](/docs/concepts/abstract-types) through the same table:

```rust
use cgp::core::error::ErrorTypeProviderComponent;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<anyhow::Error>,
    }
}
```

An effect handler interprets operations, which are values and computations. It has no notion of
supplying a *type member* the way `HasErrorType` supplies `Error`. The table that says "raise errors
this way" also says "and the error type is `anyhow::Error`".

### A closer theoretical fit: coeffects

The *coeffect* framework may describe CGP more precisely than the effect framework does. Where an
effect describes what a computation produces or performs, a coeffect describes what a computation
*requires from its environment* ([Petricek, *Coeffects*](https://tomasp.net/coeffects/)). A CGP
context carries exactly such requirements, resolved all at once when a concrete context is defined,
and the framing also accounts for the abstract-type extension above, which an effect handler has no
place for. The connection is a pointer rather than a developed account.

## What each approach costs

Algebraic effects are valued for ending *function coloring*, since code between the operation and its
handler needs no awareness of the effect
([Abramov, *Algebraic Effects for the Rest of Us*](https://overreacted.io/algebraic-effects-for-the-rest-of-us/)),
for separating an effect's interface from its semantics, and for composing handlers where monad
transformers are painful ([*Why Algebraic Effects?*, Ante](https://antelang.org/blog/why_effects/)).
Their costs, as their users state them, fall into three clusters. Unfamiliarity and control-flow
opacity: following control from a `perform` to the handler that catches it is as hard as reasoning
about a distant exception handler ([Ante](https://antelang.org/blog/why_effects/)). Performance:
general handlers must capture continuations, and non-tail handlers remain more expensive than native
control flow even after evidence passing ([Xie & Leijen, 2021](https://xnning.github.io/papers/multip.pdf)).
And, in the untyped designs, an unhandled effect is a runtime crash rather than a compile error, and
OCaml's one-shot restriction rules out the multi-shot uses ([OCaml manual](https://ocaml.org/manual/5.5/effects.html)).
Readers who reach effects from Haskell know the type-class-based effect libraries and their
n²-instances problem, which handler-based systems avoid
([`fused-effects`](https://hackage.haskell.org/package/fused-effects)).

CGP's costs are the ordinary ones. A component has to be declared before it can have providers, and
the wiring is code somebody writes. The compile-time work is real. And the raw diagnostics are
trait-solver output over generated types: [`cargo cgp check`](/docs/cargo-cgp/check) leads with the
root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape
every class. CGP also keeps even less of the "algebraic" than the practical effect languages do: it
has no equational theory relating its operations. Since Koka, OCaml, and Flix mostly drop the laws
too, this is a shared simplification. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy)
page weighs CGP's costs against the alternatives.

## Where effect handlers are the only tool

When a program needs the continuation, for a scheduler, a generator, a backtracking solver, or a
suspendable coroutine, algebraic effects are the right and only tool of the two. Emulating them with
CGP is not possible, because a provider never captures a continuation. CGP does have an
[async handler family](/docs/concepts/handlers) and [type-level DSLs](/docs/concepts/type-level-dsls)
that interpret a `Code` tag by dispatching to a provider, which is the closest CGP comes to the
operations-and-interpreters shape, but even there the interpretation is a straight call chain resolved
at compile time. Async in CGP is Rust's own `async` and `await` threaded through provider calls.

## What to expect that differs

**A provider returns exactly once.** There is no `resume` to call zero times or twice, because there
is no reified continuation. CGP takes the fragment of effect handlers the literature identifies as
dynamic binding, the fragment that compiles to direct calls, and builds everything on it.

**Handlers are chosen by the context's type, not by dynamic scope.** The nearest handler on the
runtime stack does not win, because there is no stack of handlers. A CGP provider is fixed once in a
wiring table and resolved through the trait system.

**The wiring is a set, not a stack.** An effect handler stack is ordered, and reordering handlers
changes results: state over nondeterminism and nondeterminism over state compute different things
([Kammar, Lindley & Oury, *Handlers in Action*](https://denotational.co.uk/publications/kammar-lindley-oury-handlers-in-action.pdf)).
CGP's table has one provider per component with no nesting to shadow an outer handler and no order to
permute, and because dispatch is exactly-once and resume-in-place, the non-commutativity never arises.

**Abstract types are a requirement kind effect systems lack.** A context fixes a type as well as an
operation's interpretation, through the same table.

**CGP is not an effect system.** It has no continuations, no dynamic scope, and no effect kind in the
type system. It is the exactly-once, resume-in-place fragment of effect handlers, which is dynamic
binding, made into a compile-time, per-context, type-directed wiring mechanism that also configures
abstract types.

## Where to go next

- [Modular error handling](/docs/concepts/modular-error-handling): the error type, its raising, and
  its wrapping as three wiring decisions.
- [Implicit arguments](/docs/concepts/implicit-arguments): the dynamic-binding fragment on its own
  terms.
- [Handlers](/docs/concepts/handlers): CGP's computation family, and where async lives in it.
- [Capabilities](./capabilities.md): the effects-as-capabilities reading of the same fragment.
- [Row polymorphism](./row-polymorphism.md): the row types Koka uses for its effect rows.

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
