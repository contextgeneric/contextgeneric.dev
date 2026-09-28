//! Code from `docs/reference/traits/type-level/map_type.md` — `MapType`.
//!
//! Pins the Examples program: what each standard marker stores, checked as type equalities, a
//! generic function bounded on a marker, and a bound that pins the generic associated type.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::IsOptional;
    use cgp::prelude::*;

    // Each function compiles only if the projection is the type on its right.
    pub fn present(value: <IsPresent as MapType>::Map<String>) -> String {
        value
    }

    pub fn nothing(value: <IsNothing as MapType>::Map<String>) -> () {
        value
    }

    pub fn void(value: <IsVoid as MapType>::Map<String>) -> Void {
        value
    }

    pub fn optional(value: <IsOptional as MapType>::Map<String>) -> Option<String> {
        value
    }

    // Generic over any marker, the way the builder and extractor families are.
    pub fn store<M: MapType, T>(value: M::Map<T>) -> M::Map<T> {
        value
    }

    // A bound that pins the storage names the generic associated type with its argument.
    pub fn unwrap_present<M: MapType<Map<String> = String>>(value: M::Map<String>) -> String {
        value
    }

    pub fn demo() {
        assert_eq!(present("a".to_owned()), "a");
        assert_eq!(optional(None), None);
        assert_eq!(store::<IsOptional, u8>(Some(1)), Some(1));
        assert_eq!(unwrap_present::<IsPresent>("b".to_owned()), "b");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
