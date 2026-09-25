//! The deployment bridge to the framework paging pipeline
//! (`rushwind-storage-seaorm-support`): the generated `PagingRequest`
//! resolves into the framework's params (the impl lives in the proto
//! crate — the orphan rule pins it to the type's owner), and the
//! pipeline itself — filter binding by column kind, the orderBy
//! spellings, ordering, slicing — lives there. The repository call
//! sites keep passing the request verbatim.

pub use rushwind_storage_seaorm_support::paging::fetch_paged;
