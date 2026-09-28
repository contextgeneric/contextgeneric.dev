use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::prelude::*;

#[cgp_component(PortParser)]
#[use_type(HasErrorType.Error)]
pub trait CanParsePort {
    fn parse_port(&self, raw: &str) -> Result<u16, Error>;
}

#[cgp_impl(new ParsePort)]
#[uses(CanRaiseError<ParseIntError>)]
#[use_type(HasErrorType.Error)]
impl PortParser {
    fn parse_port(&self, raw: &str) -> Result<u16, Error> {
        raw.parse().map_err(Self::raise_error)
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        PortParserComponent: ParsePort,
    }
}

check_components! {
    App {
        PortParserComponent,
    }
}

fn main() {}
