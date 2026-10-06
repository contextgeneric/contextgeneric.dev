//! Code from `docs/reference/macros/enum.md` — *`Enum!`*.
//!
//! Type equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Overview
pub mod overview {
    use cgp::prelude::*;

    #[test]
    fn the_shape_is_the_hand_written_variant_list() {
        let _: PhantomData<Enum! { Circle(f64), Square(f64) }> =
            PhantomData::<Sum![Field<Symbol!("Circle"), f64>, Field<Symbol!("Square"), f64>]>;
    }
}

/// ## Usage
pub mod usage {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub enum Shape {
        Empty,
        Circle(f64),
        Rectangle { width: f64, height: f64 },
    }

    #[test]
    fn each_variant_shape_matches_the_derive() {
        let _: PhantomData<
            Enum! {
                Empty,
                Circle(f64),
                Rectangle { width: f64, height: f64 },
            },
        > = PhantomData::<<Shape as HasFields>::Fields>;
        let _: PhantomData<Enum! {}> = PhantomData::<Void>;
        let _: PhantomData<Enum! { r#match(u8) }> = PhantomData::<Sum![Field<Symbol!("match"), u8>]>;
    }
}

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, CgpData)]
    pub enum Value {
        Int(u64),
        Text(String),
    }

    pub fn from_shape<T>(fields: Enum! { Int(u64), Text(String) }) -> T
    where
        T: FromFields<Fields = Enum! { Int(u64), Text(String) }>,
    {
        T::from_fields(fields)
    }

    #[test]
    fn a_shape_value_converts_into_the_enum() {
        assert_eq!(from_shape::<Value>(Either::Left(Field::from(7))), Value::Int(7));
        assert_eq!(
            from_shape::<Value>(Either::Right(Either::Left(Field::from("seven".to_owned())))),
            Value::Text("seven".to_owned()),
        );
    }
}

/// ## Under the hood
pub mod under_the_hood {
    use cgp::prelude::*;

    #[test]
    fn the_expansion() {
        let _: PhantomData<Enum! { Circle(f64), Square(f64) }> = PhantomData::<
            Either<Field<Symbol!("Circle"), f64>, Either<Field<Symbol!("Square"), f64>, Void>>,
        >;
    }

    #[test]
    fn the_payload_table() {
        let _: PhantomData<Enum! { Empty }> = PhantomData::<Sum![Field<Symbol!("Empty"), Nil>]>;
        let _: PhantomData<Enum! { Empty() }> = PhantomData::<Enum! { Empty }>;
        let _: PhantomData<Enum! { Empty {} }> = PhantomData::<Enum! { Empty }>;
        let _: PhantomData<Enum! { Circle(f64) }> = PhantomData::<Sum![Field<Symbol!("Circle"), f64>]>;
        let _: PhantomData<Enum! { Pair(u32, u32) }> =
            PhantomData::<Sum![Field<Symbol!("Pair"), Struct!(u32, u32)>]>;
        let _: PhantomData<Enum! { Rect { width: f64, height: f64 } }> =
            PhantomData::<Sum![Field<Symbol!("Rect"), Struct! { width: f64, height: f64 }>]>;
    }

    #[test]
    fn equivalent_spellings() {
        let _: PhantomData<Enum! { V(Nil) }> = PhantomData::<Enum! { V }>;
        let _: PhantomData<Enum! { V(Struct! { a: u32 }) }> = PhantomData::<Enum! { V { a: u32 } }>;
    }
}
