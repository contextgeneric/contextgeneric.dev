//! Code from `docs/concepts/extensible-records.md` — *Extensible records*.

/// ## A struct as a list of named fields
pub mod a_struct_as_a_list_of_named_fields {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct DatabaseClient {
        pub url: String,
        pub pool_size: u32,
    }

    /// The derive gives the struct a type-level description of its own shape. Naming it here is
    /// what pins the claim the page makes about it.
    #[test]
    fn the_shape_is_a_type() {
        fn assert_shape<T>()
        where
            T: HasFields<Fields = Product![Field<Symbol!("url"), String>, Field<Symbol!("pool_size"), u32>]>,
        {
        }

        assert_shape::<DatabaseClient>();
    }
}

/// ## Building one field at a time
pub mod building_one_field_at_a_time {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct App {
        pub database: String,
        pub http_timeout: u32,
        pub feature_flag: bool,
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct DatabaseConfig {
        pub database: String,
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct HttpConfig {
        pub http_timeout: u32,
        pub feature_flag: bool,
    }

    #[test]
    fn independent_pieces_merge_into_a_whole() {
        use cgp::core::field::impls::CanBuildFrom;

        let database = DatabaseConfig {
            database: "postgres://…".to_owned(),
        };

        let http = HttpConfig {
            http_timeout: 30,
            feature_flag: true,
        };

        let app: App = App::builder()
            .build_from(database)
            .build_from(http)
            .finalize_build();

        assert_eq!(app.http_timeout, 30);
        assert!(app.feature_flag);
    }
}

/// ## Finalizing early does not compile
///
/// The partial record tracks which fields are present, and the operation that turns it back into
/// the struct exists only at the all-present configuration.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/extensible_records_finalizing_early_does_not_compile.rs`.
pub mod finalizing_early_does_not_compile {}
