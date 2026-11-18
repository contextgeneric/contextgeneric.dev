// A zero-sized marker generic over a parameter it stores no value of does not compile without a
// `PhantomData` field: Rust requires every type parameter to be used. This is the "before" of the
// `PhantomData` page's *Why a marker needs it* section; the "after" adds `PhantomData<Field>`.

pub struct Multiply<Field>;

fn main() {}
