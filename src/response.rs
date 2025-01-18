use bytes::Bytes;

use crate::Pop3Error;

#[derive(Debug)]
pub struct Response {
    data: Bytes
}

impl Response {

    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    pub fn raw(&self) -> &Bytes {
        &self.data
    }

    pub fn to_string(&self) -> Result<String, Pop3Error> {
        std::str::from_utf8(&self.data[..])
            .map(|s| s.to_string())
            .map_err(Pop3Error::InvalidString)
    }
}