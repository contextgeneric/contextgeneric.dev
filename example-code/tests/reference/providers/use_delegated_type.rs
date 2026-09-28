//! Code from `docs/reference/providers/use_delegated_type.md` — *`UseDelegatedType`*.
//!
//! Pins the direct form the page's Usage section shows: the built-in `TypeProviderComponent` wired to
//! `UseDelegatedType`, so `HasType<Tag>` looks each tag up in a table of types. The alias form lives in
//! `with_delegated_type.rs`.

/// ## Usage
pub mod usage {
    use core::marker::PhantomData;

    use cgp::core::types::{TypeProviderComponent, UseDelegatedType};
    use cgp::prelude::*;

    pub struct ScalarTag;
    pub struct IndexTag;

    pub struct AppTypes;

    delegate_components! {
        AppTypes {
            ScalarTag: f64,
            IndexTag: usize,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            TypeProviderComponent: UseDelegatedType<AppTypes>,
        }
    }

    pub fn types(
        scalar: PhantomData<<App as HasType<ScalarTag>>::Type>,
        index: PhantomData<<App as HasType<IndexTag>>::Type>,
    ) -> (PhantomData<f64>, PhantomData<usize>) {
        (scalar, index)
    }
}
