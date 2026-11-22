//! The app (C-side) BFF service library face: configuration, the shared
//! runtime state, the token store, and the REST/SSE transports. The
//! binary (`main.rs`) is a thin assembly entry over this surface.

pub mod captcha;
pub mod config;
pub mod login_rate_limiter;
pub mod server;
pub mod services;
pub mod state;
pub mod token;
