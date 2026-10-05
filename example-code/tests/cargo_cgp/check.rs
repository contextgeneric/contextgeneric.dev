//! Code from `docs/cargo-cgp/check.md` — *Check*.
//!
//! Both programs on the page are broken on purpose, so both are `trybuild` fixtures:
//!
//! - *Try it on a deliberate mistake* — `tests/compile_fail/cargo_cgp/check_try_it_on_a_deliberate_mistake.rs`,
//!   the unchecked call to `area` on a `Rectangle` without its `height` field. The same program is
//!   `index_the_problem_it_solves.rs`, for the section index.
//! - *Check the wiring where you write it* —
//!   `tests/compile_fail/cargo_cgp/check_check_the_wiring_where_you_write_it.rs`, the same mistake with
//!   `print_area` replaced by a `check_components!` block.
//!
//! The page's repaired program, with `height` restored, is the one [`super::expand`] carries.
