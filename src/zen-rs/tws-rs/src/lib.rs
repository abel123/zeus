#![feature(type_alias_impl_trait)]

pub use client::{Client, ClientRef};
pub use errors::Error;

pub mod client;
pub mod contracts;
pub mod errors;
mod messages;
mod server_versions;
