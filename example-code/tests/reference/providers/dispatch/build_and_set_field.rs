//! Code from `docs/reference/providers/dispatch/build_and_set_field.md` — `BuildAndSetField`.
//!
//! Pins the Examples program: the second step reads the field the first step set, since each
//! provider receives a reference to the partial record built so far.

/// ## Examples
pub mod examples {
    use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};
    use cgp::extra::handler::CanCompute;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub struct PoolConfig {
        pub max_connections: u32,
        pub max_idle: u32,
    }

    #[derive(HasField)]
    pub struct PoolBuilder {
        pub connections: u32,
    }

    #[cgp_impl(new BuildMaxConnections)]
    impl<Code, Input> Computer<Code, Input> {
        type Output = u32;

        fn compute(
            &self,
            _code: PhantomData<Code>,
            _input: Input,
            #[implicit] connections: u32,
        ) -> u32 {
            connections
        }
    }

    // Reads `max_connections` from the partial record, which the step before it has set.
    #[cgp_new_provider]
    impl<'a, Context, Code, Builder> Computer<Context, Code, &'a Builder> for BuildMaxIdle
    where
        Builder: HasField<Symbol!("max_connections"), Value = u32>,
    {
        type Output = u32;

        fn compute(_context: &Context, _code: PhantomData<Code>, builder: &'a Builder) -> u32 {
            builder.get_field(PhantomData) / 2
        }
    }

    delegate_components! {
        PoolBuilder {
            ComputerComponent:
                BuildWithHandlers<PoolConfig, Product![
                    BuildAndSetField<Symbol!("max_connections"), BuildMaxConnections>,
                    BuildAndSetField<Symbol!("max_idle"), BuildMaxIdle>,
                ]>,
        }
    }

    check_components! {
        PoolBuilder {
            ComputerComponent: ((), ()),
        }
    }

    pub fn demo() {
        let builder = PoolBuilder { connections: 8 };

        assert_eq!(
            builder.compute(PhantomData::<()>, ()),
            PoolConfig {
                max_connections: 8,
                max_idle: 4,
            },
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
