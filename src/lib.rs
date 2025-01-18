mod builder;
mod client;
mod error;
mod request;
mod response;

#[cfg(feature = "with-rustls")]
use {
    rustls::StreamOwned,
    rustls::{ClientConfig, ClientSession},
    std::sync::Arc,
    webpki::DNSNameRef,
};

pub use error::Pop3Error;
pub use builder::Builder;
pub use client::*;
pub use request::Command;
pub use response::Response;

