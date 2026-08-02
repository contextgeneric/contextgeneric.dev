//! Code from `docs/concepts/coherence.md` — *Bypassing coherence*.
//!
//! Module names follow the page's headings. Code the page **rejects** — the blocks whose comment
//! quotes an `error[E…]` — is carried as a `compile_fail` doctest rather than as code, since the
//! crate could not compile otherwise; each one records the code the compiler actually reports.

/// ## The trait system is already a dependency-injection mechanism
pub mod trait_system_as_dependency_injection {
    use core::fmt::Display;

    pub fn describe<T: Display>(value: &T) -> String {
        format!("{value}")
    }

    #[test]
    fn the_caller_names_no_implementation() {
        assert_eq!(describe(&42), "42");
    }
}

/// ## That only works because every lookup finds the same answer
///
/// Both blocks in this section are rejected by the compiler, which is the point of showing them, so
/// neither can be code in this crate. The bodies the page elides are filled in below so that the
/// quoted error is the *only* thing wrong with each — a `/* ... */` body would fail on its own and
/// the doctest would pass for the wrong reason.
///
/// The overlap rule. `String` is both `Display` and `AsRef<[u8]>`:
///
/// ```compile_fail
/// use core::fmt::Display;
///
/// pub trait CanEncode {
///     fn encode(&self) -> Vec<u8>;
/// }
///
/// // Legal on its own.
/// impl<T: Display> CanEncode for T {
///     fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
/// }
///
/// // error[E0119]: conflicting implementations of trait `CanEncode`
/// impl<T: AsRef<[u8]>> CanEncode for T {
///     fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
/// }
/// ```
///
/// The orphan rule. Neither `Display` nor `Vec<u8>` belongs to this crate:
///
/// ```compile_fail
/// use core::fmt::{self, Display, Formatter};
///
/// // error[E0117]: only traits defined in the current crate can be implemented
/// //               for types defined outside of the crate
/// impl Display for Vec<u8> {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result { Ok(()) }
/// }
/// ```
pub mod every_lookup_finds_the_same_answer {}

/// ## The move: make `Self` a type you own
///
/// … together with **## Coherence comes back, one context at a time**, which wires the component
/// this section defines. The page splits them across two headings; they are one program.
pub mod the_move {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_component(Encoder)]
    pub trait CanEncodeValue<Value> {
        fn encode(&self, value: &Value) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeAsText)]
    impl<Value> Encoder<Value>
    where
        Value: Display,
    {
        fn encode(&self, value: &Value) -> Vec<u8> {
            value.to_string().into_bytes()
        }
    }

    #[cgp_impl(new EncodeAsHex)]
    impl<Value> Encoder<Value>
    where
        Value: AsRef<[u8]>,
    {
        // The page elides this body as `/* hex-encode the bytes */`.
        fn encode(&self, value: &Value) -> Vec<u8> {
            value
                .as_ref()
                .iter()
                .flat_map(|byte| format!("{byte:02x}").into_bytes())
                .collect()
        }
    }

    // The page uses these two contexts without declaring them, since by that point it has only
    // introduced the word "context".
    pub struct ApiServer;
    pub struct Firmware;

    delegate_components! {
        ApiServer {
            open EncoderComponent;
            @EncoderComponent.String: EncodeAsText,
        }
    }

    delegate_components! {
        Firmware {
            open EncoderComponent;
            @EncoderComponent.String: EncodeAsHex,
        }
    }

    mod check_api_server {
        use super::*;
        check_components! { ApiServer { EncoderComponent: String } }
    }

    mod check_firmware {
        use super::*;
        check_components! { Firmware { EncoderComponent: String } }
    }

    /// The page quotes the output of this in a ```` ```text ```` block, so the two lines are pinned
    /// here rather than left to be re-derived by eye.
    #[test]
    fn the_two_contexts_encode_the_same_string_differently() {
        let value = "hi".to_owned();

        assert_eq!(
            String::from_utf8(ApiServer.encode(&value)).unwrap(),
            "hi", // ApiServer: hi
        );
        assert_eq!(
            String::from_utf8(Firmware.encode(&value)).unwrap(),
            "6869", // Firmware:  6869
        );
    }
}

/// ### The shape this depends on
///
/// The arrangement of [`the_move`] with the CGP taken back out: two application types, one value
/// type, two encodings, and no CGP at all. It compiles, which is the section's whole point.
pub mod the_shape_this_depends_on {
    pub trait CanEncodeValue<Value> {
        fn encode(&self, value: &Value) -> Vec<u8>;
    }

    pub struct ApiServer;
    pub struct Firmware;

    impl CanEncodeValue<String> for ApiServer {
        // The page elides this body as `/* as text */`.
        fn encode(&self, value: &String) -> Vec<u8> {
            value.clone().into_bytes()
        }
    }

    impl CanEncodeValue<String> for Firmware {
        // The page elides this body as `/* as hexadecimal */`.
        fn encode(&self, value: &String) -> Vec<u8> {
            value
                .as_bytes()
                .iter()
                .flat_map(|byte| format!("{byte:02x}").into_bytes())
                .collect()
        }
    }
}

/// ### The shape this depends on — where it stops scaling
///
/// The second block of that section: factoring the shared logic into a blanket implementation runs
/// into the overlap rule, this time on an application type. Rejected, so it is a doctest.
///
/// ```compile_fail
/// use core::fmt::Display;
///
/// pub trait CanEncodeValue<Value> {
///     fn encode(&self, value: &Value) -> Vec<u8>;
/// }
///
/// pub struct ApiServer;
///
/// impl<V: Display> CanEncodeValue<V> for ApiServer {
///     fn encode(&self, value: &V) -> Vec<u8> { value.to_string().into_bytes() }
/// }
///
/// // error[E0119]: conflicting implementations of trait `CanEncodeValue<_>`
/// //               for type `ApiServer`
/// impl<V: AsRef<[u8]>> CanEncodeValue<V> for ApiServer {
///     fn encode(&self, value: &V) -> Vec<u8> { value.as_ref().to_vec() }
/// }
/// ```
pub mod the_shape_stops_scaling {}
