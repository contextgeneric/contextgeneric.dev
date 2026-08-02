---
sidebar_label: 'Modular error handling'
sidebar_position: 10
---

# Modular error handling

The error type, how a foreign error becomes it, and what detail it carries: three choices a context
makes independently of the code that fails.

This page answers *how does fallible code avoid committing to an error type?* It separates the three
decisions error handling actually contains, shows a provider that makes none of them, and ends with the
same pattern applied to an application's own error vocabulary. It closes on where the separation stops
paying.

## Why the error type is the hard one

Every fallible function has to return *something*, and generic code has no business deciding what. A
provider parsing a port number does not know whether the application wants `anyhow::Error`, a domain
enum, or a plain string — and hard-coding any of them commits every caller in every application to that
choice.

The usual answers each cost something. A concrete error type in a library is a decision imposed on
users. A generic `<E>` parameter is an input the caller supplies, so it lands in every intermediate
signature along with its bounds, which is the leak [impl-side dependencies](./impl-side-dependencies.md)
describes. Converting by hand at each boundary is the boilerplate error-handling crates exist to remove,
and it comes back the moment the boundary is generic.

What makes this tractable is noticing that "error handling" is not one decision but three, and that they
are independent:

- **what the error type is** — a type the context names;
- **how a foreign error becomes it** — `ParseIntError`, `io::Error`, a `String` from a domain rule;
- **what detail is attached** as it propagates.

CGP makes each of the three a wiring choice, and the code that fails makes none of them.

## Code that fails without knowing what failing means

The capability names its error abstractly, and the provider raises into it:

```rust
#[cgp_component(PortParser)]
#[use_type(HasErrorType.Error)]
pub trait CanParsePort {
    fn parse_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new ParsePortFromStr)]
#[uses(CanRaiseError<ParseIntError>, CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl PortParser {
    fn parse_port(&self, raw: &str) -> Result<u16, Error> {
        let parsed: u32 = raw.parse().map_err(Self::raise_error)?;

        if parsed > u16::MAX as u32 {
            return Err(Self::raise_error(format!("port {parsed} out of range")));
        }

        Ok(parsed as u16)
    }
}
```

`Error` is the context's [abstract type](./abstract-types.md), imported by `#[use_type]`.
`Self::raise_error` turns a concrete failure into it, and the `#[uses]` line declares which failures
this provider raises — two of them, a parse error and a string — as requirements on the implementation,
not on the interface. A caller bounding on `CanParsePort` learns none of it.

`raise_error` is called on the *type* rather than on a value, because constructing an error is something
the context knows how to do rather than something a particular value does.

## Three sources, three strategies, one table

The context supplies the answers, and the second and third decisions are made per source error type:

```rust
delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        PortParserComponent: ParsePortFromStr,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

Three lines, three decisions. The error type is `String`. A raised `String` is converted straight
through with `From`. A `ParseIntError` is formatted with `Debug` into a `String` — and then handed back
to the context's own `String` route, which is what makes the two compose rather than each needing to
know the final type.

That last point is what the per-source dispatch buys. A real application raises a dozen unrelated
failures, and most of them want the same treatment; naming a strategy per source type lets the
interesting ones differ without a match arm anywhere. CGP ships the strategies as ordinary providers —
`RaiseFrom` for a `From` conversion, `DebugError` and `DisplayError` for formatting, `ReturnError` when
the source already is the error type, `RaiseInfallible` for a step that cannot fail — and they stay
generic over whatever error type the context chose.

## Changing the answer changes nothing else

Because the decisions are separate, moving to a different error type is a change to the table:

```rust
delegate_components! {
    StrictApp {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<AppError>,
        PortParserComponent: ParsePortFromStr,

        @ErrorRaiserComponent.String: RaiseFrom,
        @ErrorRaiserComponent.ParseIntError: DebugError,
    }
}
```

`ParsePortFromStr` is the same provider, unmodified and un-recompiled in any meaningful sense. It never
named `String`, so nothing in it referred to the thing that changed. Swapping `anyhow` for `eyre`, or a
prototype's `String` for a real domain type, is this edit.

Concrete backends come as separate crates for the same reason — `cgp-error-anyhow` and its siblings each
supply a type-setting provider and the raisers that go with it, so the dependency on `anyhow` lives in
the wiring rather than in any code that fails.

## An application's own error vocabulary

None of this is confined to CGP's built-in components, and the clearest sign of that is defining your
own. A service that wants every failure to carry an HTTP status declares a capability for it:

```rust
#[cgp_component(HttpErrorRaiser)]
#[use_type(HasErrorType.Error)]
pub trait CanRaiseHttpError<Code, Detail> {
    fn raise_http_error(code: Code, detail: Detail) -> Error;
}
```

`ErrUnauthorized` and `ErrNotFound` are empty structs standing for status codes, and a provider per code
builds the concrete error. A handler deep in the request pipeline then writes

```rust
Self::raise_http_error(ErrUnauthorized, "you must first login")
```

and knows neither the status number nor the error type. The two are decided in the table, in the same
place as everything else the application decides — which is the whole pattern, applied to a vocabulary
the built-in components know nothing about.

## What it costs

**The imports are not in the prelude, deliberately.** `HasErrorType` and `CanRaiseError` are, but the
wiring keys live under `cgp::core::error` and the strategy providers under `cgp::extra::error`. That is
a real papercut the first time, and it is the price of the error components not being forced on code
that does not use them.

**Three decisions means three ways to be under-wired.** A context can name an error type and forget a
raiser for a source some provider raises, and it compiles until something raises one.
[`check_components!`](/docs/reference/macros/check_components) is what catches it, as with any other
wiring.

**The error type is one per context.** Everything in a context agreeing on one `Error` is what lets
errors compose without conversion; it also means a context genuinely needing two unrelated error types
needs two components or two contexts.

**And it does not decide what a good error is.** CGP makes the type swappable and the construction
routable. Whether your errors carry useful context, whether they are matchable, whether the messages
help — all of that is the same design problem it always was, and none of it is answered by wiring.

## Where to go next

[Abstract types](./abstract-types.md) is the mechanism the error type rests on, and worth reading first
if `#[use_type]` above was unfamiliar. [Impl-side dependencies](./impl-side-dependencies.md) is why the
`#[uses(CanRaiseError<…>)]` line does not reach callers, and
[Dispatching](./dispatching.md) is the general form of the per-source routing.

For the constructs, [`HasErrorType`](/docs/reference/components/has_error_type) is the abstract error
type, [`CanRaiseError`](/docs/reference/components/can_raise_error) covers raising and wrapping, and
[the error providers](/docs/reference/providers/error_providers) is the catalogue of strategies with the
bound each one places on the context.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
