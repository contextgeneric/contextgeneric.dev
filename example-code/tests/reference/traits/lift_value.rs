//! Code from `docs/reference/traits/monad/lift_value.md` — `LiftValue`.
//!
//! Pins the Examples program: each shipped marker's two lifting functions, called directly.

/// ## Examples
pub mod examples {
    use cgp::extra::monad::monadic::err::ErrMonadic;
    use cgp::extra::monad::monadic::ident::IdentMonadic;
    use cgp::extra::monad::monadic::ok::OkMonadic;
    use cgp::extra::monad::traits::LiftValue;

    pub fn demo() {
        assert_eq!(<IdentMonadic as LiftValue<u8, u8>>::lift_value(1), 1);

        // A bare value enters an `ErrMonadic` output as `Ok`, and an `OkMonadic` one as `Err`.
        let lifted: Result<u8, String> =
            <ErrMonadic as LiftValue<u8, Result<u8, String>>>::lift_value(1);
        assert_eq!(lifted, Ok(1));

        let lifted: Result<u8, String> =
            <OkMonadic as LiftValue<String, Result<u8, String>>>::lift_value("no".to_owned());
        assert_eq!(lifted, Err("no".to_owned()));

        // An output already in the inner shape is forwarded unchanged.
        let forwarded =
            <ErrMonadic as LiftValue<u8, Result<u8, String>>>::lift_output(Err("stop".to_owned()));
        assert_eq!(forwarded, Err("stop".to_owned()));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
