//! The payload re-export. The Redis token families live in the core
//! service (`core-service/src/token.rs`); the app BFF ships no cookies
//! — the reference's app layer keeps the refresh token in the response
//! body.

pub use store::auth::UserTokenPayload;
