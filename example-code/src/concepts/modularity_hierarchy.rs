//! Code from `docs/concepts/modularity-hierarchy.md` — *Modularity Hierarchy*.
//!
//! One module per tier. The page walks the five tiers on one running capability — encoding a value —
//! so each module encodes the same kind of thing with more modularity than the one before it. The
//! `CanEncode` / `Encoder` / `EncodeAsText` names are the site's shared encoding example, carried over
//! from the front page and *Bypassing coherence*.

/// ## Tier 1: one implementation for every type
///
/// A blanket implementation: one body that applies everywhere the bound holds, with no alternative.
pub mod tier_1 {
    pub trait CanEncode {
        fn encode(&self) -> Vec<u8>;
    }

    impl<Value: AsRef<[u8]>> CanEncode for Value {
        fn encode(&self) -> Vec<u8> {
            self.as_ref().to_vec()
        }
    }
}

/// ## Tier 2: one implementation per type
///
/// A plain trait: a different body per type, and coherence allows only one of them.
pub mod tier_2 {
    pub trait CanEncode {
        fn encode(&self) -> Vec<u8>;
    }

    impl CanEncode for u32 {
        fn encode(&self) -> Vec<u8> {
            self.to_string().into_bytes()
        }
    }

    impl CanEncode for Vec<u8> {
        fn encode(&self) -> Vec<u8> {
            self.clone()
        }
    }
}

/// ## Tier 3: many implementations, one wired per type
///
/// The self-targeted component, wired on a value context (`u32`) and on an environmental context
/// (`App` / `TestApp`). The page shows the two shapes this tier holds.
pub mod tier_3 {
    use cgp::prelude::*;

    #[cgp_component(SelfEncoder)]
    pub trait CanEncodeSelf {
        fn encode(&self) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeSelfAsText)]
    #[uses(core::fmt::Display)]
    impl SelfEncoder {
        fn encode(&self) -> Vec<u8> {
            self.to_string().into_bytes()
        }
    }

    #[cgp_impl(new EncodeSelfAsBytes)]
    #[uses(AsRef<[u8]>)]
    impl SelfEncoder {
        // The page elides this body.
        fn encode(&self) -> Vec<u8> {
            self.as_ref().to_vec()
        }
    }

    delegate_components! {
        u32 {
            SelfEncoderComponent: EncodeSelfAsText,
        }
    }

    mod check_u32 {
        use super::*;
        check_components! { u32 { SelfEncoderComponent } }
    }

    // The application shape: the wired type stands for the application, not for data.
    #[cgp_component(EmailSender)]
    pub trait CanSendEmail {
        fn send_email(&self, to: &str, body: &str);
    }

    #[cgp_impl(new SendViaSmtp)]
    impl EmailSender {
        // The page elides this body as `/* connect and send over SMTP */`.
        fn send_email(&self, to: &str, body: &str) {}
    }

    #[cgp_impl(new RecordEmails)]
    impl EmailSender {
        // The page elides this body as `/* record to a Vec a test can read */`.
        fn send_email(&self, to: &str, body: &str) {}
    }

    pub struct App;
    pub struct TestApp;

    delegate_components! { App { EmailSenderComponent: SendViaSmtp } }
    delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }

    mod check_app {
        use super::*;
        check_components! { App { EmailSenderComponent } }
    }

    mod check_test_app {
        use super::*;
        check_components! { TestApp { EmailSenderComponent } }
    }
}

/// ## Tier 4: many implementations, one per type per context
///
/// The parameter-targeted component: two application contexts encode the same foreign type
/// (`String`) differently.
pub mod tier_4 {
    use cgp::prelude::*;

    #[cgp_component(Encoder)]
    pub trait CanEncodeValue<Value> {
        fn encode(&self, value: &Value) -> Vec<u8>;
    }

    #[cgp_impl(new EncodeAsText)]
    impl<Value> Encoder<Value>
    where
        Value: core::fmt::Display,
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
}

/// ## Tier 5: many implementations, per type per provider
///
/// A higher-order provider, built on tier 4's `Encoder` component. It encodes a `Vec` by encoding
/// each element through an inner provider, which defaults to the context.
pub mod tier_5 {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    use super::tier_4::{CanEncodeValue, EncodeAsHex, EncodeAsText, Encoder, EncoderComponent};

    pub struct EncodeVecWith<Inner = UseContext>(pub PhantomData<Inner>);

    #[cgp_impl(EncodeVecWith<Inner>)]
    #[use_provider(Inner: Encoder<Item>)]
    impl<Item, Inner> Encoder<Vec<Item>> {
        fn encode(&self, value: &Vec<Item>) -> Vec<u8> {
            value
                .iter()
                .flat_map(|item| Inner::encode(self, item))
                .collect()
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open EncoderComponent;
            @EncoderComponent.u32: EncodeAsText,
            @EncoderComponent.Vec<u32>: EncodeVecWith,
            @EncoderComponent.Vec<Vec<u8>>: EncodeVecWith<EncodeAsHex>,
        }
    }

    // Calling the consumer trait forces the whole wiring chain to resolve at compile time, which a
    // `delegate_components!` entry does not do on its own. The `Vec<u32>` call exercises the
    // `UseContext` default, and the `Vec<Vec<u8>>` call exercises the pinned inner provider.
    #[test]
    fn app_encodes_nested_collections() {
        let flat = <App as CanEncodeValue<Vec<u32>>>::encode(&App, &vec![1, 2, 3]);
        assert_eq!(flat, b"123");

        let nested = <App as CanEncodeValue<Vec<Vec<u8>>>>::encode(&App, &vec![vec![0xab]]);
        assert_eq!(nested, b"ab");
    }
}
