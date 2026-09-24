//! The data layer — the persistence face, mirroring the reference's
//! `internal/data`. The sea-orm entity tree and the paging pipeline
//! live in the shared `store` crate (the equivalent of the reference's
//! ent-generated code); this layer adds the repos whose query shape
//! spans tables or needs raw SQL.

pub mod content_repo;
pub mod helpers;
