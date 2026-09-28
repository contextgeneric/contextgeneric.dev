//! Code from `docs/reference/traits/type-level/transform_map_fields.md` — `TransformMapFields`.
//!
//! Pins the Examples program: a transform applied by naming the trait at a concrete call site, and
//! through a bound in generic code. `under_the_hood` pins the walk's order, last declared field
//! first. Calling the method bare at a concrete site is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::{TransformMap, TransformMapFields};
    use cgp::prelude::*;

    // Doubles every present number and keeps it present.
    pub struct Double;

    impl<T: core::ops::Add<Output = T> + Copy> TransformMap<IsPresent, IsPresent, T> for Double {
        fn transform_mapped(value: T) -> T {
            value + value
        }
    }

    #[derive(Debug, PartialEq, CgpData)]
    pub struct Point {
        pub x: u32,
        pub y: u32,
    }

    // At a concrete site, name the transform and the target marker on the trait.
    pub fn doubled(point: Point) -> Point {
        TransformMapFields::<Double, IsPresent>::transform_map_fields(point.into_builder())
            .finalize_build()
    }

    // In generic code, the bound fixes both, so the method is called bare.
    pub fn double_all<Builder>(builder: Builder) -> Builder::Output
    where
        Builder: TransformMapFields<Double, IsPresent>,
    {
        builder.transform_map_fields()
    }

    pub fn demo() {
        assert_eq!(doubled(Point { x: 1, y: 2 }), Point { x: 2, y: 4 });

        let point = double_all(Point { x: 3, y: 4 }.into_builder()).finalize_build();
        assert_eq!(point, Point { x: 6, y: 8 });
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## Under the hood
///
/// The walk recurses into the rest of the list first, so a transform with a side effect sees the
/// last declared field first.
pub mod under_the_hood {
    use std::cell::RefCell;

    use cgp::core::field::traits::{TransformMap, TransformMapFields};
    use cgp::prelude::*;

    thread_local! {
        static SEEN: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    }

    pub struct Record;

    impl TransformMap<IsPresent, IsPresent, u32> for Record {
        fn transform_mapped(value: u32) -> u32 {
            SEEN.with(|seen| seen.borrow_mut().push(value));
            value
        }
    }

    #[derive(CgpData)]
    pub struct Triple {
        pub a: u32,
        pub b: u32,
        pub c: u32,
    }

    #[test]
    fn test_last_field_first() {
        let builder = Triple { a: 1, b: 2, c: 3 }.into_builder();
        let _ =
            TransformMapFields::<Record, IsPresent>::transform_map_fields(builder).finalize_build();
        SEEN.with(|seen| assert_eq!(*seen.borrow(), vec![3, 2, 1]));
    }
}
