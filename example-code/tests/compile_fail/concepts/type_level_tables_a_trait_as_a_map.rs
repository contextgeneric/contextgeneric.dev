pub trait HasEntry<Key> {
    type Value;
}

pub struct AreaTable;

pub struct Rectangle;
pub struct RectangleArea;
pub struct SquareArea;

impl HasEntry<Rectangle> for AreaTable {
    type Value = RectangleArea;
}

// error[E0119]: conflicting implementations of trait `HasEntry<Rectangle>` for type `AreaTable`
impl HasEntry<Rectangle> for AreaTable {
    type Value = SquareArea;
}

fn main() {}
