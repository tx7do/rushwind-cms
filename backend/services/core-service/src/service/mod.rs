//! The service layer — the domain implementations behind the gRPC
//! face, mirroring the reference's `internal/service`. Queries ride
//! the data layer ([`crate::data`]) and the shared store crate.

pub mod audit;
pub mod authentication;
pub mod content;
pub mod credential;
pub mod dict;
pub mod identity;
pub mod messaging;
pub mod misc;
pub mod permission;
pub mod policy_evaluation_log;
pub mod site;
pub mod social;
pub mod stats;
pub mod storage;
pub mod writes;
