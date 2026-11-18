// error: cannot find attribute `implicit` in this scope
pub fn area(width_source: &f64, #[implicit] width: f64) -> f64 {
    let _ = width_source;
    width
}

fn main() {}
