//! Code from `docs/reference/components/handler/producer.md` — `Producer`.
//!
//! Pins the `MagicNumber` provider from the page's Examples, wired onto a context and invoked through
//! `CanProduce`.

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::extra::handler::{CanProduce, Producer, ProducerComponent};
    use cgp::prelude::*;

    #[cgp_new_provider]
    impl<Context, Code> Producer<Context, Code> for MagicNumber {
        type Output = u64;

        fn produce(_context: &Context, _code: PhantomData<Code>) -> u64 {
            42
        }
    }

    pub struct App;

    delegate_components! {
        App {
            ProducerComponent: MagicNumber,
        }
    }

    mod check_app {
        use super::*;

        check_components! {
            App {
                ProducerComponent: (),
            }
        }
    }

    #[test]
    fn app_produces_the_magic_number() {
        assert_eq!(App.produce(PhantomData::<()>), 42);
    }
}
