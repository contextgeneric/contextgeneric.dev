//! Code from `docs/concepts/coherence.md` — *Bypassing coherence*.

/// ## The trait system is already a dependency-injection mechanism
pub mod trait_system_as_dependency_injection {
    use core::fmt::{self, Display, Formatter};

    pub fn describe<T: Display>(value: &T) -> String {
        format!("{value}")
    }

    struct Pair<A, B>(A, B);

    impl<A: Display, B: Display> Display for Pair<A, B> {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.0, self.1)
        }
    }

    #[test]
    fn the_caller_names_no_implementation() {
        assert_eq!(describe(&42), "42");
        assert_eq!(describe(&Pair("foo", 42u32)), "(foo, 42)");
    }
}

/// ## That only works because every lookup finds the same answer
///
/// Rejected snippets are covered by the trybuild fixtures
/// `coherence_every_lookup_finds_the_same_answer_1.rs` (overlap) and
/// `coherence_every_lookup_finds_the_same_answer_2.rs` (orphan rule).
pub mod every_lookup_finds_the_same_answer {}

/// ## The move: make `Self` a type you own
/// ## Coherence comes back, one type at a time
pub mod the_move {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_component(Encoder)]
    pub trait CanEncode {
        fn encode(&self) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeAsText)]
    #[uses(Display)]
    impl Encoder {
        fn encode(&self) -> Vec<u8> {
            self.to_string().into_bytes()
        }
    }

    #[cgp_impl(new EncodeAsHex)]
    #[uses(AsRef<[u8]>)]
    impl Encoder {
        // The page elides the hexadecimal encoding body.
        fn encode(&self) -> Vec<u8> {
            self.as_ref()
                .iter()
                .flat_map(|byte| format!("{byte:02x}").into_bytes())
                .collect()
        }
    }

    delegate_components! { String  { EncoderComponent: EncodeAsText } }
    delegate_components! { Vec<u8> { EncoderComponent: EncodeAsHex  } }

    mod check_string {
        use super::*;
        check_components! { String { EncoderComponent } }
    }

    mod check_bytes {
        use super::*;
        check_components! { Vec<u8> { EncoderComponent } }
    }

    #[test]
    fn each_value_type_selects_its_encoding() {
        assert_eq!("hi".to_owned().encode(), b"hi");
        assert_eq!(b"hi".to_vec().encode(), b"6869");
    }
}
