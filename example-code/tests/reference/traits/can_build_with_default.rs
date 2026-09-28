//! Code from `docs/reference/traits/optional/can_build_with_default.md` — `CanBuildWithDefault`.
//!
//! Pins the Examples program: a record widened into one with an extra field, in one call and as the
//! three calls it stands for. A source field the target lacks is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::extra::field::impls::{CanBuildWithDefault, CanFinalizeWithDefault};
    use cgp::prelude::*;

    #[derive(Debug, Clone, Eq, PartialEq, CgpData)]
    pub struct Point2d {
        pub x: u64,
        pub y: u64,
    }

    #[derive(Debug, Clone, Eq, PartialEq, CgpData)]
    pub struct Point3d {
        pub x: u64,
        pub y: u64,
        pub z: u64,
    }

    pub fn demo() {
        let point_3d = Point3d::build_with_default(Point2d { x: 1, y: 2 });
        assert_eq!(point_3d, Point3d { x: 1, y: 2, z: 0 });

        let written_out: Point3d = Point3d::builder()
            .build_from(Point2d { x: 1, y: 2 })
            .finalize_with_default();
        assert_eq!(written_out, point_3d);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
