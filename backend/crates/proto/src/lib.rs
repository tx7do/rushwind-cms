//! The CMS API contract crate — everything generated from the synced
//! proto tree (`api/protos`, checksummed by `api/MANIFEST.sha256`):
//!
//! * [`proto`] — the types: prost structs + pbjson protojson serde.
//! * [`DESCRIPTOR_BYTES`] / [`pool`] — the annotated full compile closure:
//!   buf-produced (protox's serializer would drop the custom-option
//!   bytes — google.api.http / errors.code — which are the point), decoded
//!   once into the immutable process-global pool. The schema surface for
//!   protojson serialization and form binding.
//! * [`gen`] — the build-time-emitted surface (the framework's
//!   `rushwind-gen-http` over the annotated closure): route table with
//!   form-binding plans, reason → HTTP status error tables, one service
//!   trait per annotated BFF service (the admin and the app BFF both
//!   ride here), null placeholder impls, and the public/gated mount
//!   emitters split by [`AUTH_FREE`].
//! * [`tables`] — handwritten accessors over the generated error tables.

use std::sync::OnceLock;

use prost_reflect::DescriptorPool;

/// Generated prost types + pbjson serde impls for the CMS contract modules.
///
/// Generated code carries no doc comments and is not held to hand-written lint
/// standards; the contract prose lives in the .proto sources.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto_include.rs"));
}

/// The CMS proto compile closure as raw `FileDescriptorSet` bytes —
/// annotations included — compiled by the build script from the checksummed
/// contract tree and the buf.lock-pinned dependency modules
/// (`backend/api/buf.yaml`).
pub static DESCRIPTOR_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/annotated_descriptor.bin"));

/// The decoded descriptor pool. The pool is immutable and process-global;
/// decoding happens once.
pub fn pool() -> &'static DescriptorPool {
    static POOL: OnceLock<DescriptorPool> = OnceLock::new();
    POOL.get_or_init(|| {
        DescriptorPool::decode(DESCRIPTOR_BYTES).expect("annotated_descriptor.bin must decode")
    })
}

/// Generated route/error/trait/mount surface of the ADMIN BFF face —
/// do not edit; regenerate by building.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod gen_admin {
    include!(concat!(env!("OUT_DIR"), "/admin_gen.rs"));
}

/// Generated route/error/trait/mount surface of the APP BFF face —
/// do not edit; regenerate by building.
#[allow(missing_docs)]
#[allow(clippy::all)]
pub mod gen_app {
    include!(concat!(env!("OUT_DIR"), "/app_gen.rs"));
}

/// Handwritten table accessors over the generated annotation tables.
pub mod tables;

include!("auth_free.rs");

/// The static redaction plan over the annotated pool — fail-closed
/// resolved once per process (the contract tree carries the same
/// redact.v1 options as the admin deployment: user email/mobile masks,
/// the nested list envelopes, the method_skip set). The generated
/// mounts thread this to the lifecycle glue.
pub fn redact_plan() -> &'static rushwind_redact::RedactPlan {
    static PLAN: std::sync::OnceLock<rushwind_redact::RedactPlan> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| {
        rushwind_redact::RedactPlan::build(pool())
            .expect("redact plan over the annotated contract pool must build")
    })
}

/// The generated `PagingRequest` resolves into the framework paging
/// pipeline's params — the impl lives in THIS crate because the orphan
/// rule pins it to the type's owner; the repositories' call sites keep
/// passing the request verbatim.
impl rushwind_storage_seaorm_support::paging::PagingInput
    for crate::proto::pagination::PagingRequest
{
    fn paging_params(&self) -> rushwind_storage_seaorm_support::paging::Params {
        use crate::proto::pagination::paging_request::FilteringType;
        rushwind_storage_seaorm_support::paging::Params {
            query: match &self.filtering_type {
                Some(FilteringType::Query(query)) => Some(query.clone()),
                _ => None,
            },
            order_by: self.order_by.clone(),
            sorting: self
                .sorting
                .iter()
                .filter(|s| !s.field.is_empty())
                .map(|s| rushwind_storage_seaorm_support::paging::Sorting {
                    field: s.field.clone(),
                    desc: s.direction == 1, // Direction::DESC
                })
                .collect(),
            page: self.page,
            page_size: self.page_size,
            offset: self.offset,
            limit: self.limit,
            no_paging: self.no_paging.unwrap_or(false),
        }
    }
}
