use super::request::*;
use super::types::*;

pub trait RequestValidator {
    fn validate(request: &Request) -> Result<()>;
}

// Example validator
pub struct X11RequestValidator;

impl RequestValidator for X11RequestValidator {
    fn validate(_request: &Request) -> Result<()> {
        // TODO: Implement real validation
        Ok(())
    }
}
