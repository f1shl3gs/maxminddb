#![deny(trivial_casts, trivial_numeric_casts, unused_import_braces)]

mod decode;
mod errors;
mod metadata;
pub mod models;
mod reader;

pub use errors::Error;
pub use reader::{
    AnonymousIp, Asn, City, ConnectionType, Country, Domain, Enterprise, Isp, Reader,
};
