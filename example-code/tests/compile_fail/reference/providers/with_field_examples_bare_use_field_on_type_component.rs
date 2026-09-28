use cgp::prelude::*;

#[cgp_type]
pub trait HasWidthType {
    type Width;
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f32,
}

delegate_components! {
    Rectangle {
        WidthTypeProviderComponent: UseField<Symbol!("width")>,
    }
}

check_components! {
    Rectangle {
        WidthTypeProviderComponent,
    }
}

fn main() {}
