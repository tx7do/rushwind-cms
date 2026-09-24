//! Cross-domain query helpers shared by the service layer (per-domain
//! repos live beside their service files' queries; these are the ones
//! several services call).

/// Shared DB failure mapping.
pub fn db_status(e: sea_orm::DbErr) -> tonic::Status {
    tonic::Status::internal(format!("db: {e}"))
}
