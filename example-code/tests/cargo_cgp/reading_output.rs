//! Code from `docs/cargo-cgp/reading-output.md` — *Reading the output*.
//!
//! Every program the page shows is broken on purpose and is a `trybuild` fixture under
//! `tests/compile_fail/cargo_cgp/reading_output_*.rs`. The one program the page says compiles is the fix
//! it gives under *A fix in a help line*: adding `#[uses(PersonName)]` to `greeting`.

/// *A fix in a `help` line* — the program with the attribute the `[CGP-E012]` help names.
pub mod a_fix_in_a_help_line {
    use cgp::prelude::*;

    #[cgp_fn]
    fn person_name(&self, #[implicit] name: &str) -> String {
        name.to_owned()
    }

    #[cgp_fn]
    #[uses(PersonName)]
    fn greeting(&self) -> String {
        format!("Hello, {}!", self.person_name())
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[test]
    fn the_declared_dependency_resolves() {
        let person = Person {
            name: "World".to_owned(),
        };
        assert_eq!(person.greeting(), "Hello, World!");
    }
}
