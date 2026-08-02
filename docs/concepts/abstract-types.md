---
sidebar_label: 'Abstract types'
sidebar_position: 5
---

# Abstract types

Naming a type in generic code — an error type, a scalar, a runtime — and letting each context decide
what it actually is.

This page answers *how does code name a type it does not choose?* It starts from the ordinary Rust
feature that already does this, shows what CGP adds to it, and works through the two things abstract
types buy: a type several pieces of code can agree on, and signatures that stop growing. It closes on
the costs, which are mostly about what an abstract type is *not*.

## Rust already has this

An abstract type in CGP is an ordinary associated type, and it is worth starting there because it means
half of what follows is a feature you already use:

```rust
pub trait HasScalarType {
    type Scalar: Copy;
}

impl HasScalarType for HighPrecision {
    type Scalar = f64;
}

impl HasScalarType for Embedded {
    type Scalar = f32;
}
```

Generic code names `Context::Scalar` and never commits to `f32` or `f64`. Each type answers for itself,
the compiler resolves the answer where a concrete type is known, and nothing costs anything at runtime.
That is the whole idea, and CGP does not replace it — a context can implement an abstract-type trait
directly, exactly as above, and everything else on this page still works.

What is worth noticing is the *direction*. A generic parameter is an input the caller supplies; an
associated type is an output the implementing type determines. That difference is small in one function
and decides how a codebase ages, which the last section of this page is about.

## Choosing the type by wiring

What CGP adds is that the choice can be made in the same place as every other choice a context makes.
[`#[cgp_type]`](/docs/reference/macros/cgp_type) turns an abstract-type trait into a component:

```rust
#[cgp_type]
pub trait HasScalarType {
    type Scalar: Copy;
}
```

and a context then names its concrete type in its wiring table rather than in a hand-written impl:

```rust
delegate_components! {
    HighPrecision {
        ScalarTypeProviderComponent: UseType<f64>,
    }
}

delegate_components! {
    Embedded {
        ScalarTypeProviderComponent: UseType<f32>,
    }
}
```

`UseType<f64>` says "the type is `f64`", and that is all it says — every abstract type is answered the
same trivial way, so `#[cgp_type]` generates that answer once and a context supplies the type as an
argument to it. The bound on the associated type is carried through and enforced on whatever the context
chooses: wiring `UseType<String>` against a `Copy` scalar is an error reading
`the trait bound String: Copy is not satisfied`. Like every other wiring choice it is
[checked lazily](./check-traits.md), so that error appears where the component is checked or used rather
than on the wiring line itself.

This is a small win on its own. What it buys is that a context's type choices sit beside its behaviour
choices, in one table, rather than being scattered across `impl` blocks — and that a type can be chosen
by something more interesting than a fixed answer, since `UseType<T>` is just one provider among the
ones a component can be wired to.

## One type, agreed on by everything that needs it

The idea compounds as soon as more than one piece of code needs the same type, because they all name the
*same* `Self::Scalar` and the context fixes it once.

That matters most when the code that needs the type is generic over something else entirely. A shape
calculator works over rectangles, circles, and whatever else, and none of those shapes should have to
carry a scalar type or agree with the others about one:

```rust
#[cgp_component(ShapeAreaCalculator)]
#[use_type(HasScalarType.Scalar)]
pub trait CanCalculateShapeArea<Shape> {
    fn shape_area(&self, shape: &Shape) -> Scalar;
}
```

`Rectangle` declares nothing. The application declares the scalar, every shape interoperates through it,
and switching the application from `UseType<f32>` to `UseType<f64>` changes the arithmetic for every
shape at once. The application here is a type standing for the program rather than a piece of data —
`struct App;` with no fields is a complete one — which is the shape most CGP code is in.

The `#[use_type(HasScalarType.Scalar)]` line is what lets the signature say `Scalar` instead of
`<Self as HasScalarType>::Scalar`. It imports the type and adds the requirement in one line, and it
reads like a `use` for a type because that is what it is.

## The canonical one: a context's error type

The abstract type CGP leans on hardest is the error type, and it is the clearest case of code naming
something it refuses to choose:

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

`LoadOrFail` fails, and never learns what failing means in this application. The context decides —
`anyhow::Error`, a domain enum, a plain `String` — and every fallible provider in that context refers to
the same one, so errors compose instead of needing conversion at each boundary.
[Modular error handling](./modular-error-handling.md) is the page for what else follows from that.

## Two things called `UseType`

One collision is worth naming before you meet it in a compiler error, because the two are complementary
and easy to conflate. **`UseType<T>` is a provider**: it goes in a wiring table and says what a
context's abstract type is. **`#[use_type(...)]` is an attribute**: it goes on a definition and imports
somebody's abstract type so the code can name it as a bare alias.

A context uses the first; code that consumes an abstract type uses the second. They appear in the same
program constantly and never in the same position.

## What it costs

**It is deferral, not encapsulation.** An abstract type leaves the choice open; it does not hide the
answer. Once a context wires `UseType<f64>`, code with that context in hand sees `f64` and can do
anything `f64` allows. If the goal is that callers must *not* know the representation, that is Rust's
module privacy, and an abstract type is the wrong tool for it — a distinction worth being precise about
with anyone arriving from ML modules.

**A bound on the associated type is the only thing generic code can rely on.** `type Scalar: Copy` means
generic code can copy a scalar and nothing else — no arithmetic, no comparison, unless those bounds are
declared too. Adding one later is a change every context has to satisfy, so the bounds are worth
thinking about when the trait is written.

**The type is one per context, not one per use.** All the code in a context sharing one `Error` is the
point, and it is also a constraint: a context needing two genuinely different error types needs two
components, or two contexts.

**And it is another thing that can be left unwired.** A context that never names its error type compiles
until something needs one, at which point the failure is a missing type rather than a missing field, and
looks much the same. [`check_components!`](/docs/reference/macros/check_components) catches it at the
wiring line, like everything else.

## Where to go next

[Impl-side dependencies](./impl-side-dependencies.md) puts this page in its place: an abstract type is
the type-shaped version of a requirement stated on the implementation, beside the capability leg and the
value leg. [Implicit arguments](./implicit-arguments.md) is that value leg.

[Modular error handling](./modular-error-handling.md) develops the canonical case at length — the error
type, how a foreign error becomes it, and what detail it carries, as three separate choices.

For the constructs, [`#[cgp_type]`](/docs/reference/macros/cgp_type) defines an abstract-type component,
[`UseType`](/docs/reference/providers/use_type) supplies the concrete type in a wiring table,
[`#[use_type]`](/docs/reference/attributes/use_type) imports one into a definition, and
[`HasType`](/docs/reference/components/has_type) is the built-in component the rest is built on.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
