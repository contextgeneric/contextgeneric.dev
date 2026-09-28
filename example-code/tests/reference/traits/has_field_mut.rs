//! Code from `docs/reference/traits/field-access/has_field_mut.md` — `HasFieldMut`.
//!
//! Pins the Examples program: a provider that increments a field of its context, and a write
//! through a `Box` that resolves by the `DerefMut` forwarding impl.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Counter)]
    pub trait CanCount {
        fn count(&mut self);
    }

    #[cgp_impl(new IncrementCounter)]
    impl Counter
    where
        Self: HasFieldMut<Symbol!("counter"), Value = u64>,
    {
        fn count(&mut self) {
            *self.get_field_mut(PhantomData::<Symbol!("counter")>) += 1;
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub counter: u64,
    }

    delegate_components! {
        App {
            CounterComponent: IncrementCounter,
        }
    }

    check_components! {
        App {
            CounterComponent,
        }
    }

    pub fn reset<Context>(context: &mut Context)
    where
        Context: HasFieldMut<Symbol!("counter"), Value = u64>,
    {
        *context.get_field_mut(PhantomData) = 0;
    }

    pub fn demo() {
        let mut app = App { counter: 0 };
        app.count();
        app.count();
        assert_eq!(app.counter, 2);

        // `Box<App>` has the field mutably through the `DerefMut` forwarding impl.
        let mut boxed = Box::new(App { counter: 5 });
        reset(&mut boxed);
        assert_eq!(boxed.counter, 0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
