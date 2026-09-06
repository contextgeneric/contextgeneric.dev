# Website example code

This crate holds compiled counterparts of the code shown on <https://contextgeneric.dev>. **It is not
part of the website.** Nothing here is rendered, linked, or served, no page refers to it, and readers
are not meant to find it. It exists for the agent revising a documentation page, so that the code on
that page can be checked against something the compiler has agreed to rather than read and hoped for.

Run it from this directory:

```sh
cargo test              # the full check: the code compiles, and the rejected snippets still fail
cargo test --no-run     # the fast check: everything compiles, nothing runs
```

The code lives in **integration tests**, which a plain `cargo check` skips; use `cargo test --no-run`
(or `cargo check --tests`) for a compile-only pass. The crate has no library or binary target.

## The layout mirrors `docs/`

The code lives under `tests/`, as one integration-test binary per `docs/` section. Each entrypoint
pulls in a module tree that mirrors that section's pages:

- `tests/concepts_tests.rs` → `tests/concepts/`, mirroring `docs/concepts/`
- `tests/reference_tests.rs` → `tests/reference/`, mirroring `docs/reference/`
- `tests/cargo_cgp_tests.rs` → `tests/cargo_cgp/`, mirroring `docs/cargo-cgp/`
- `tests/compile_fail_tests.rs` → `tests/compile_fail/`, the snippets the pages **reject** (see below)

One file per page, at the matching path with the file name in `snake_case`:

| Section | Coverage |
|---|---|
| `docs/concepts/` | complete — one file under `tests/concepts/` per page that shows code |
| `docs/reference/` | partial — `errors.md`, `macros/delegate_components.md`, the `attributes/` pages `default_impl.md`, `impl_generics.md`, and `prefix.md`, all of `derives/`, the `traits/` pages that show checkable code, most of `providers/` (the singletons, all of `error/`, `handler/`, and `monad/`, and the matcher-side of `dispatch/`), all of `components/` (each component page that shows code, including the `handler/` subsection), and all of `types/` (each type page that shows code, the `spines/` subsection included); the section is filled in lazily |
| `docs/cargo-cgp/` | complete — the two pages that show Rust, under `tests/cargo_cgp/` |
| `docs/tutorials/` | none yet |
| front page, orientation pages | none yet |

Within a covered section the mapping is mechanical: `docs/concepts/coherence.md` is answered by
`tests/concepts/coherence.rs`, and `docs/concepts/consumer-and-provider-traits.md` by
`tests/concepts/consumer_and_provider_traits.rs`.

A page that shows no code gets no file — `docs/concepts/modularity-hierarchy.md` is prose and tables,
so it has no module here, and neither does a section index.

Inside a file, **one module per heading of the page**, named after that heading and in the page's
order, so a snippet can be found from where it sits on the page rather than by searching. Where two
consecutive headings are really one program, one module covers both and says so.

**Duplication between files is expected and wanted.** Each file answers for one page on its own, so a
scenario two pages share is written out twice rather than factored into a helper that neither page
shows. A reader comparing a page against its file should never have to follow an import to see what
the page said.

## What "matches" means

A page shows the shape of a program; this crate shows a program. The two differ in ways that are
correct, and confusing an expected difference for a defect wastes more time than the check saves.

**Expected, and not a defect.** A body the page elides as `/* ... */` is filled in here, and every
such fill-in carries a comment saying the page elided it. Imports the page omits are present.
Types the page names without declaring — a provider it has not introduced yet, a context it
introduces two sections later — are declared. `check_components!` assertions are added for every
wired context, since that is what proves the wiring resolves rather than merely parses, and pages do
not show them. Tests are added where a page quotes an output or makes a claim worth pinning.

**A defect, in one or the other.** Anything else: a signature, a trait bound, an attribute, a derive,
a wiring line, a provider name, an item that exists on one side and not the other. When they
disagree, find out which is wrong rather than assuming — the crate is checked by the compiler and the
page is not, so the page is the likelier culprit, but a stale file here is worse than no file.

## Code a page deliberately rejects

Several pages show code the compiler refuses, quoting the error as the point of the snippet. That code
cannot live in a module, so each rejected snippet is a standalone fixture under `tests/compile_fail/`,
at the page's mirrored path, and [`trybuild`](https://docs.rs/trybuild) compiles each one and checks
that it fails. The harness is `tests/compile_fail_tests.rs`.

Each fixture is a **complete program** whose only fault is the intended one: the elided bodies are
filled in, an item-only snippet carries a trailing `fn main() {}`, and a snippet with statements is
wrapped in `fn main() { ... }` — the way rustdoc wraps the doctest these were carried as before.

trybuild compares each fixture's output against a sibling `.stderr` file. **What that genuinely guards
is the regression that matters: a snippet the page calls rejected that the compiler has started
accepting.** The exact wording of a `.stderr` is toolchain- and `cgp`-version-specific, so re-bless
them after a bump rather than editing them by hand:

```sh
TRYBUILD=overwrite cargo test --test compile_fail_tests
```

## Generated code

Two of these pages show what a macro *generates*, in simplified form. A hand-rolled rendering of such
a listing — as `concepts::consumer_and_provider_traits::how_a_call_finds_its_provider` carries — is a
weaker check than the rest of this crate: it shows the listing is structurally sound, not that its
text is current. **The authority on what a macro emits is `cargo cgp expand`**, run against the test
target the example lives in (for example `cargo cgp expand --test concepts_tests --item …`). Check the
text with `expand`; check the shape here.

## Adding a file

Files are added **lazily**: when a page that shows code is written, or revised, or reviewed, its file
is created or brought back into agreement in the same change. There is no obligation to backfill
pages nobody is working on, and a missing file means only that nobody has been through that page yet.

Adding a runnable one means creating it at the mirrored path under `tests/<section>/`, registering it
in the parent `mod.rs`, adding a row to the table above, and leaving `cargo test` green. Adding a
rejected snippet means writing a fixture under `tests/compile_fail/`, blessing its `.stderr` with
`TRYBUILD=overwrite cargo test --test compile_fail_tests`, and leaving `cargo test` green.

## The `cgp` version

The crate pins `cgp = "0.8.0-alpha"`, which is what resolves from crates.io today. Re-pin it to
`"0.8.0"` when that release ships, alongside the version pins in the tutorials.
