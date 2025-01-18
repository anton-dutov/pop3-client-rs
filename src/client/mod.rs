use crate::{Command, Response, Pop3Error};

pub type Result<T> = std::result::Result<T, Pop3Error>;

#[cfg(feature = "runtime-sync")]
mod sync;


#[cfg(feature = "runtime-tokio")]
mod tokio;


#[cfg(feature = "runtime-sync")]
pub use sync::SyncClient;


#[cfg(feature = "runtime-tokio")]
pub use tokio::AsyncClient;

fn join_bytes(arrays: &[&[u8]], separator: u8) -> Vec<u8> {
    let cap: usize = arrays.iter().map(|a| a.len()).sum();

    let mut result = Vec::with_capacity(cap + arrays.len() - 1);

    for (i, array) in arrays.iter().enumerate() {
        result.extend_from_slice(array);
        if i < arrays.len() - 1 {
            result.push(separator);
        }
    }

    result
}

