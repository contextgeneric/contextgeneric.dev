//! Code from `docs/reference/types/void.md` — *`Void`*.
//!
//! `Void` is uninhabited, so the Examples program never builds one. It checks the empty `Sum!` is
//! `Void`, closes a match behind a reference with an empty `match` on the `Void` arm, leaves the arm
//! out of a match by value, and unwraps a `Result` whose error type is `Void` through
//! `FinalizeExtractResult`. The reference match without its `Void` arm is a trybuild fixture under
//! `tests/compile_fail/reference/types/`.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::FinalizeExtractResult;
    use cgp::prelude::*;

    pub type Token = Sum![u32, bool];

    // Behind a reference, the `Void` arm is required.
    pub fn describe(token: &Token) -> String {
        match token {
            Either::Left(number) => format!("number {number}"),
            Either::Right(Either::Left(flag)) => format!("flag {flag}"),
            Either::Right(Either::Right(void)) => match *void {},
        }
    }

    // By value, it may be left out.
    pub fn is_number(token: Token) -> bool {
        match token {
            Either::Left(_) => true,
            Either::Right(Either::Left(_)) => false,
        }
    }

    pub fn demo() {
        // The annotations are the check: the chain ends in `Void`, and the empty sum is `Void`.
        let _: PhantomData<Either<u32, Either<bool, Void>>> = PhantomData::<Token>;
        let _: PhantomData<Void> = PhantomData::<Sum![]>;

        assert_eq!(describe(&Either::Right(Either::Left(true))), "flag true");
        assert!(is_number(Either::Left(3)));

        // A `Result` that cannot fail unwraps without an `unwrap`.
        let result: Result<u32, Void> = Ok(5);
        assert_eq!(result.finalize_extract_result(), 5);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
