//! Code from `docs/reference/types/chars.md` — *`Chars`*.
//!
//! A `Symbol!` wraps a `Chars` chain. The Examples program reads a field through its `Symbol!` tag,
//! checks the chain a symbol expands to (with its byte length for a non-ASCII name), and rebuilds the
//! string through `Display`. The raw error for a missing field, whose `Chars` chain the page shows how
//! to read, is a trybuild fixture under `tests/compile_fail/reference/types/`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    // The tag names the field the function reads.
    pub fn name_of<Context>(context: &Context) -> &str
    where
        Context: HasField<Symbol!("name"), Value = String>,
    {
        context.get_field(PhantomData::<Symbol!("name")>)
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    pub fn demo() {
        let person = Person {
            name: "Alice".to_owned(),
        };
        assert_eq!(name_of(&person), "Alice");

        // `Symbol!` spells the string out as a `Chars` chain, with its byte length in front.
        let _: PhantomData<Symbol<3, Chars<'a', Chars<'b', Chars<'c', Nil>>>>> =
            PhantomData::<Symbol!("abc")>;
        let _: PhantomData<Symbol<6, Chars<'世', Chars<'界', Nil>>>> = PhantomData::<Symbol!("世界")>;

        // `Display` walks the chain to rebuild the text.
        let symbol = <Symbol!("hello")>::default();
        assert_eq!(symbol.to_string(), "hello");
        assert_eq!(core::mem::size_of::<Symbol!("hello")>(), 0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
