#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate data_encoding;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_urlencoded;

pub mod errors;
mod mail;
mod sg_client;
pub mod v3;

pub use mail::{Mail,Destination};
pub use sg_client::SGClient;
