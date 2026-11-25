//! Async client for the qBittorrent WebUI API (`/api/v2`), pinned to WebAPI 2.17.0.
//!
//! See `docs/api-inventory.md` for the endpoint reference this crate implements.

pub mod capabilities;
pub mod client;
pub mod config;
pub mod endpoints;
pub mod error;
pub mod models;
pub mod params;

mod api;

pub use capabilities::{Capabilities, WebApiVersion};
pub use client::{Client, Response};
pub use config::{AuthMode, ClientConfig};
pub use error::{Error, Result};
pub use params::{Hashes, Params};
