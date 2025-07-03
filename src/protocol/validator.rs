use super::request::*;
use super::types::*;

pub trait RequestValidator {
    fn validate(request: &Request) -> Result<()>;
}

// Example validator
pub struct X11RequestValidator;

impl RequestValidator for X11RequestValidator {
    fn validate(_request: &Request) -> Result<()> {
        match _request.kind {
            RequestKind::CreateWindow(ref req) => {
                if req.width == 0 || req.height == 0 {
                    return Err(X11Error::DetailedValue(
                        "CreateWindow: width and height must be non-zero".to_string(),
                    ));
                }
                if req.depth == 0 {
                    return Err(X11Error::DetailedValue(
                        "CreateWindow: depth must be non-zero".to_string(),
                    ));
                }
            }
            RequestKind::DestroyWindow(ref req) => {
                if req.window == 0 {
                    return Err(X11Error::DetailedValue(
                        "DestroyWindow: window id must be non-zero".to_string(),
                    ));
                }
            }
            RequestKind::MapWindow(ref req) => {
                if req.window == 0 {
                    return Err(X11Error::DetailedValue(
                        "MapWindow: window id must be non-zero".to_string(),
                    ));
                }
            }
            RequestKind::UnmapWindow(ref req) => {
                if req.window == 0 {
                    return Err(X11Error::DetailedValue(
                        "UnmapWindow: window id must be non-zero".to_string(),
                    ));
                }
            }
            RequestKind::InternAtom(ref req) => {
                if req.atom_name.is_empty() {
                    return Err(X11Error::DetailedValue(
                        "InternAtom: atom_name must not be empty".to_string(),
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    }
}
