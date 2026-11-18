//! Code from `docs/tooling/cargo-cgp/reading-output.md` — *Reading the output*.
//!
//! Every program the page shows is broken on purpose and is a `trybuild` fixture under
//! `tests/compile_fail/cargo_cgp/reading_output_*.rs`. The programs here are the two fixes it gives
//! under *A fix in a help line*: adding `#[uses(PersonName)]` to `greeting`, and importing
//! `AreaCalculatorComponent` beside `AreaCalculator`, as the `[CGP-E041]` fix does.

/// *A fix in a `help` line* — each program with the fix its help names applied.
pub mod a_fix_in_a_help_line {
    /// The attribute the `[CGP-E030]` help names.
    pub mod declared_dependency {
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

    /// The import the `[CGP-E041]` fix inserts. The fixture
    /// `reading_output_a_fix_in_a_help_line_import.rs` is the program without it.
    pub mod imported_key {
        pub mod components {
            use cgp::prelude::*;

            #[cgp_component(AreaCalculator)]
            pub trait CanCalculateArea {
                fn area(&self) -> f64;
            }
        }

        pub mod providers {
            use cgp::prelude::*;

            use super::components::{AreaCalculator, AreaCalculatorComponent};

            #[cgp_impl(new RectangleArea)]
            impl AreaCalculator {
                fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
                    width * height
                }
            }
        }

        use cgp::prelude::*;

        use components::{AreaCalculatorComponent, CanCalculateArea};

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f64,
            pub height: f64,
        }

        delegate_components! {
            Rectangle {
                AreaCalculatorComponent: providers::RectangleArea,
            }
        }

        check_components! {
            Rectangle {
                AreaCalculatorComponent,
            }
        }

        #[test]
        fn the_imported_key_wires_the_provider() {
            let rect = Rectangle {
                width: 3.0,
                height: 4.0,
            };
            assert_eq!(rect.area(), 12.0);
        }
    }
}
