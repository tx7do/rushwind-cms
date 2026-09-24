//! The core domain service library face: configuration, state, the
//! token store, and the domain gRPC service implementations. The
//! binary (`main.rs`) is a thin tonic assembly entry over this surface.

pub mod config;
pub mod services;
pub mod state;
pub mod token;
