//! Code from `docs/reference/providers/use_context.md` — *`UseContext`*.
//!
//! Pins the default-inner-provider use: `EncodeVec` encodes a `Vec` by encoding each element through its
//! inner provider, which defaults to `UseContext` so the elements route back to the context's own
//! `CanEncode` wiring for their type. No cycle, because the element lookup is for a different type
//! (`u32`) than the wired one (`Vec<u32>`).

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Encoder)]
    pub trait CanEncode<Value> {
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

    pub struct EncodeVec<Inner = UseContext>(pub PhantomData<Inner>);

    #[cgp_impl(EncodeVec<Inner>)]
    #[use_provider(Inner: Encoder<Item>)]
    impl<Item, Inner> Encoder<Vec<Item>> {
        fn encode(&self, values: &Vec<Item>) -> Vec<u8> {
            values
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
            @EncoderComponent.Vec<u32>: EncodeVec,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                EncoderComponent: [u32, Vec<u32>],
            }
        }
    }

    #[test]
    fn test_use_context_default_inner() {
        // The `Vec<u32>` element encoding routes through `UseContext` to the `u32` encoder.
        let flat = <App as CanEncode<Vec<u32>>>::encode(&App, &vec![1u32, 2, 3]);
        assert_eq!(flat, b"123");
    }
}
