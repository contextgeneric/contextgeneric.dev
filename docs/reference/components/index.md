---
title: 'Component reference'
description: 'The components CGP ships, grouped by job: the abstract error type, raising and wrapping errors, abstract types, the handler family, runtimes, and runners.'
sidebar_label: 'Overview'
sidebar_position: 0
---

# Components

A component is one trait, defined once, that a context can wire to any of several
implementations. The constructs on the other pages of this reference are the tools you use to *build*
your own components; the pages in this section document the components CGP already ships. Each is a
consumer trait you call, a provider trait an implementation targets, and a wiring key a context
delegates, exactly like a component you define yourself, so you wire a built-in error type or runtime
with the same [`delegate_components!`](../macros/delegate_components.md) table as everything else.

A context is the type a method runs on, which supplies whatever values an implementation needs
as its own fields. You wire these components onto a context and call them the way you call any CGP
trait. This page groups them by the job they do, in roughly the order most CGP code reaches for
them. If you are new to CGP, start with the [reference overview](/docs/reference/) rather than here.

## Handle errors

CGP keeps the error type abstract so that fallible generic code never names a concrete error, and three
components carry that. [`HasErrorType`](./has_error_type.md) gives a context one shared abstract `Error`
type, chosen once at wiring time, so that every fallible operation refers to the same error and their
results compose. It is the component nearly every non-trivial program meets, and the two below build on
it.

[`CanRaiseError`](./can_raise_error.md) turns a concrete source error into that abstract error, so a
provider can call `Context::raise_error(source)` without knowing what the context's error type is.
[`CanWrapError`](./can_wrap_error.md) attaches detail to an error the context already holds, such as a
message or a path, as it propagates. Both dispatch per source or detail type, and both are usually
satisfied by wiring one of the [error providers](../providers/error/index.md) or a standalone backend
crate rather than by hand.

## Let each context choose a type

[`HasType`](./has_type.md) is CGP's tag-indexed [abstract type](/docs/reference/glossary#abstract-type):
a trait whose associated type a context resolves, one tag at a time, to a concrete type through wiring.
CGP's other abstract types, such as `HasErrorType` and [`HasRuntimeType`](./has_runtime_type.md) below,
are named components defined with [`#[cgp_type]`](../macros/cgp_type.md), and the same
[`UseType<T>`](../providers/use_type.md) marker supplies all of them. You reach for `HasType` directly
only rarely; a named `#[cgp_type]` component reads better.

## Compute things

The [handler family](./handler/index.md) models a computation as a swappable component along three axes:
synchronous or async, fallible or not, and taking an input or not. It is a subsection of its own, with
one page per member. [`Computer`](./handler/computer.md) is the pure synchronous transform that never
fails, [`TryComputer`](./handler/try_computer.md) adds a failure path, and
[`Handler`](./handler/handler.md) is the general corner, async and fallible, that generic pipeline code
targets because every simpler member promotes up to it. [`Producer`](./handler/producer.md) is the
input-free case that yields a value from the context alone.

Each of `Computer`, `TryComputer`, and `Handler` also has by-reference and, for the computers, async
siblings, such as `ComputerRef`, `AsyncComputer`, and `HandlerRef`. Each differs from its base member by
one axis and has a page of its own in the subsection. Providers for the family are written from plain
functions with [`#[cgp_computer]`](../macros/cgp_computer.md) and
[`#[cgp_producer]`](../macros/cgp_producer.md), then composed with the
[handler combinators](../providers/handler/index.md).

## Run tasks against a runtime

The last group executes work and supplies the runtime that work runs on. [`HasRuntime`](./has_runtime.md)
hands out a borrow of the context's runtime value, and [`HasRuntimeType`](./has_runtime_type.md) declares
the abstract runtime *type* it borrows, so a provider reaches a Tokio runtime, a mock, or a test executor
generically. [`CanRun`](./runner.md) runs a task named at the type level to a `Result<(), Error>`, and
[`CanSendRun`](./send_runner.md) is the variant whose returned future is `Send`, for work that must cross
a thread boundary.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
