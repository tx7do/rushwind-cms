//! The service layer — the domain implementations behind the gRPC
//! face, mirroring the reference's `internal/service`. Queries ride
//! the data layer ([`crate::data`]) and the shared store crate.

pub mod audit;
pub mod authentication;
pub mod content;
pub mod content_model;
pub mod context;
pub mod credential;
pub mod dict;
pub mod identity;
pub mod internal_message;
pub mod login_policy;
pub mod media_asset;
pub mod messaging;
pub mod org;
pub mod permission;
pub mod permission_sync;
pub mod policy_evaluation_log;
pub mod site;
pub mod social;
pub mod stats;
pub mod storage;
pub mod writes;
