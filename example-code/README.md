# Website example code

This crate holds compiled counterparts of the code shown on <https://contextgeneric.dev>. **It is not
part of the website.** Nothing here is rendered, linked, or served, no page refers to it, and readers
are not meant to find it. It exists for the agent revising a documentation page, so that the code on
that page can be checked against something the compiler has agreed to rather than read and hoped for.

Run it from this directory:

```sh
cargo test          # the full check: the code compiles, and the rejected snippets still fail
cargo check         # the fast check: the code compiles
```

## The layout mirrors `docs/`

One file per page, at the matching path with the file name in `snake_case`:

| Section | Coverage |
|---|---|
| `docs/concepts/` | complete — sixteen files under `src/concepts/`, one per page that shows code |
| `docs/reference/` | partial — `errors.md`, `macros/delegate_components.md`, all of `derives/`, the `traits/` pages that show checkable code, and most of `providers/` (the singletons, all of `error/`, `handler/`, and `monad/`, and the matcher-side of `dispatch/`); the section is being filled in lazily |
| `docs/cargo-cgp/` | complete — the two pages that show Rust, under `src/cargo_cgp/` |
| `docs/tutorials/` | none yet |
| front page, orientation pages | none yet |

Within a covered section the mapping is mechanical: `docs/concepts/coherence.md` is answered by
`src/concepts/coherence.rs`, and `docs/concepts/consumer-and-provider-traits.md` by
`src/concepts/consumer_and_provider_traits.rs`.

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

Several pages show code the compiler refuses, quoting the error as the point of the snippet. That
code cannot live in the crate, so it is carried as a `compile_fail` doctest in the module for its
section, with the quoted error code in a comment.

Two limits are worth knowing. **`compile_fail` does not check *which* error is produced** — rustdoc
accepts an error code after the annotation but does not enforce it, so a snippet that starts failing
for an unrelated reason still passes. What the doctest genuinely catches is the regression that
matters: a snippet the page calls rejected that the compiler has started accepting. And **a body the
page elides has to be filled in for these too**, so that the quoted error is the only thing wrong
with the snippet — an empty `/* ... */` body would fail on its own and the doctest would pass while
proving nothing.

## Generated code

Two of these pages show what a macro *generates*, in simplified form. A hand-rolled rendering of such
a listing — as `concepts::consumer_and_provider_traits::how_a_call_finds_its_provider` carries — is a
weaker check than the rest of this crate: it shows the listing is structurally sound, not that its
text is current. **The authority on what a macro emits is `cargo cgp expand --lib --item …`**, run
against the crate the page's example lives in. Check the text with `expand`; check the shape here.

## Adding a file

Files are added **lazily**: when a page that shows code is written, or revised, or reviewed, its file
is created or brought back into agreement in the same change. There is no obligation to backfill
pages nobody is working on, and a missing file means only that nobody has been through that page yet.

Adding one means creating it at the mirrored path, registering it in the parent `mod.rs`, adding a
row to the table above, and leaving `cargo test` green.

## The `cgp` version

The crate pins `cgp = "0.8.0-alpha"`, which is what resolves from crates.io today. Re-pin it to
`"0.8.0"` when that release ships, alongside the version pins in the tutorials.
