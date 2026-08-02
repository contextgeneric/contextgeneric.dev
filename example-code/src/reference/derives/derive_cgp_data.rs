//! Code from `docs/reference/derives/derive_cgp_data.md` — *`#[derive(CgpData)]`*.
//!
//! The umbrella page's own snippets. Its *Under the hood* section defers the two expansions to the
//! shape pages, so what this file pins is the accepted and rejected shapes, that the umbrella and the
//! two shape-specific faces agree on their output, and that the degenerate shapes compile.

/// ## Using it
///
/// The two inputs the page opens on, the shape-specific faces, and the shapes the page says are
/// accepted. The one it says is rejected is a doctest in the module below this one.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Circle {
        pub radius: u32,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Rectangle {
        pub width: u32,
        pub height: u32,
    }

    #[derive(CgpData)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(CgpData)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum Shape {
        Circle(Circle),
        Rectangle(Rectangle),
    }

    /// The shape-specific faces, which the page says emit what the umbrella emits.
    #[derive(CgpRecord)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct RecordFace {
        pub value: u32,
    }

    #[derive(CgpVariant)]
    #[derive(Debug, Eq, PartialEq)]
    pub enum VariantFace {
        One(Circle),
    }

    /// The degenerate struct shape: a parameterless companion, so `builder()` finalizes at once.
    #[derive(CgpData)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct NoConfig {}

    /// A tuple struct, whose builder steps are keyed by position rather than by name.
    #[derive(CgpData)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Pair(pub u32, pub u32);

    #[test]
    fn test_the_degenerate_record_finalizes_immediately() {
        let config: NoConfig = NoConfig::builder().finalize_build();

        assert_eq!(config, NoConfig {});
    }

    #[test]
    fn test_a_tuple_records_builder_is_keyed_by_position() {
        let pair: Pair = Pair::builder()
            .build_field(PhantomData::<Index<0>>, 3)
            .build_field(PhantomData::<Index<1>>, 4)
            .finalize_build();

        assert_eq!(pair, Pair(3, 4));
    }

    #[test]
    fn test_the_record_face_delivers_all_three_slices() {
        let record: RecordFace = RecordFace::builder()
            .build_field(PhantomData::<Symbol!("value")>, 7)
            .finalize_build();

        // The getter slice.
        assert_eq!(*record.get_field(PhantomData::<Symbol!("value")>), 7);

        // The representation slice.
        assert_eq!(
            RecordFace::from_fields(record.to_fields()),
            RecordFace { value: 7 }
        );
    }

    #[test]
    fn test_the_variant_face_constructs_by_tag() {
        let variant =
            VariantFace::from_variant(PhantomData::<Symbol!("One")>, Circle { radius: 2 });

        assert_eq!(variant, VariantFace::One(Circle { radius: 2 }));
    }
}

/// The page's rejected snippet: a struct-style variant has no single payload type, so the derive
/// refuses it.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(CgpData)]
/// pub enum Shape {
///     Circle { radius: f64 },
/// }
/// ```
pub mod rejected_struct_style_variant {}
