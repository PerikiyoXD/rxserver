use super::response::*;
use super::types::*;

pub trait ResponseSerializer {
    fn serialize(&self, response: &Response) -> Result<Option<Vec<u8>>>;

    fn serialized_size(&self) -> usize {
        panic!("serialized_size not implemented");
    }
}

// Example for ConnectionSetupAcceptedResponse (stub)
impl ResponseSerializer for ConnectionSetupAcceptedResponse {
    fn serialize(&self, _response: &Response) -> Result<Option<Vec<u8>>> {
        // TODO: Implement serialization logic
        Ok(Some(vec![]))
    }
}

// Implement for other response types as needed...
