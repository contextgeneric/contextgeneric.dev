//! Code from `docs/comparisons/traits-without-coherence.md` — *Traits without coherence*.
//!
//! Only the CGP side is checked here; the Cairo snippet was run with Cairo 2.21.0, and the Haskell,
//! Scala, and OCaml snippets come from the pages that name their toolchains.

/// ## Assembling the chain at the call
pub mod assembling_the_chain_at_the_call {
    use cgp::prelude::*;

    #[cgp_component(ValueSerializer)]
    pub trait CanSerializeValue<T> {
        fn serialize(&self, value: &T, output: &mut Vec<u64>);
    }

    #[cgp_impl(new VecSerdeWith<Inner>)]
    #[use_provider(Inner: ValueSerializer<T>)]
    impl<T, Inner> ValueSerializer<Vec<T>> {
        fn serialize(&self, value: &Vec<T>, output: &mut Vec<u64>) {
            output.push(value.len() as u64);
            for item in value {
                Inner::serialize(self, item, output);
            }
        }
    }

    #[cgp_impl(new PairSerdeWith<SA, SB>)]
    #[use_provider(SA: ValueSerializer<A>)]
    #[use_provider(SB: ValueSerializer<B>)]
    impl<A, B, SA, SB> ValueSerializer<(A, B)> {
        fn serialize(&self, (a, b): &(A, B), output: &mut Vec<u64>) {
            SA::serialize(self, a, output);
            SB::serialize(self, b, output);
        }
    }

    #[cgp_impl(new U8Serde)]
    impl ValueSerializer<u8> {
        fn serialize(&self, value: &u8, output: &mut Vec<u64>) {
            output.push(*value as u64);
        }
    }

    #[cgp_impl(new BoolSerde)]
    impl ValueSerializer<bool> {
        fn serialize(&self, value: &bool, output: &mut Vec<u64>) {
            output.push(*value as u64);
        }
    }

    pub struct App;

    #[test]
    fn the_caller_assembles_the_chain() {
        let value = vec![(1u8, false), (2, true)];
        let mut output = Vec::new();
        <VecSerdeWith<PairSerdeWith<U8Serde, BoolSerde>> as ValueSerializer<App, Vec<(u8, bool)>>>::serialize(
            &App, &value, &mut output,
        );
        assert_eq!(output, vec![2, 1, 0, 2, 1]);
    }
}

/// ## Fixing the choice on the context
pub mod fixing_the_choice_on_the_context {
    use cgp::prelude::*;

    #[cgp_component(ValueSerializer)]
    pub trait CanSerializeValue<T> {
        fn serialize(&self, value: &T, output: &mut Vec<u64>);
    }

    #[cgp_impl(new VecSerde)]
    #[uses(CanSerializeValue<T>)]
    impl<T> ValueSerializer<Vec<T>> {
        fn serialize(&self, value: &Vec<T>, output: &mut Vec<u64>) {
            output.push(value.len() as u64);
            for item in value {
                self.serialize(item, output);
            }
        }
    }

    #[cgp_impl(new PairSerde)]
    #[uses(CanSerializeValue<A>, CanSerializeValue<B>)]
    impl<A, B> ValueSerializer<(A, B)> {
        fn serialize(&self, (a, b): &(A, B), output: &mut Vec<u64>) {
            self.serialize(a, output);
            self.serialize(b, output);
        }
    }

    #[cgp_impl(new U8Serde)]
    impl ValueSerializer<u8> {
        fn serialize(&self, value: &u8, output: &mut Vec<u64>) {
            output.push(*value as u64);
        }
    }

    #[cgp_impl(new BoolSerde)]
    impl ValueSerializer<bool> {
        fn serialize(&self, value: &bool, output: &mut Vec<u64>) {
            output.push(*value as u64);
        }
    }

    pub struct App;

    delegate_components! {
        App {
            open ValueSerializerComponent;

            @ValueSerializerComponent.<T> Vec<T>: VecSerde,
            @ValueSerializerComponent.<A, B> (A, B): PairSerde,
            @ValueSerializerComponent.u8: U8Serde,
            @ValueSerializerComponent.bool: BoolSerde,
        }
    }

    check_components! {
        App {
            ValueSerializerComponent: Vec<(u8, bool)>,
        }
    }

    #[test]
    fn the_context_fixes_the_chain() {
        let value = vec![(1u8, false), (2, true)];
        let mut output = Vec::new();
        App.serialize(&value, &mut output);
        assert_eq!(output, vec![2, 1, 0, 2, 1]);
    }
}
