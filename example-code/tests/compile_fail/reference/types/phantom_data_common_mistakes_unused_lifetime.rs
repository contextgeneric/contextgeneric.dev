// The unused-parameter rule covers lifetimes too: a marker generic over a lifetime it stores no
// borrow of does not compile. This is the `PhantomData` page's *Common Mistakes* entry on the rule.

pub struct Borrowing<'a>;

fn main() {}
