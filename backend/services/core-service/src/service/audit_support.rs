//! Shared machinery of the five audit-log services: the impl-stamping
//! macro (no Go counterpart — the package-level helpers of the
//! reference's audit service files). The varchar enum columns of the
//! audit tables ride their proto enum value names — the golden schema
//! stores what the reference's ent enums define — so every family keeps
//! its own name/ordinal converters beside its mapper (the reference's
//! EnumTypeConverter pairs).

macro_rules! audit_impl {
    ($svc:ident, $server_mod:ident, $trait_:ident, $entity:ident, $proto:ident,
     $list_resp:ident, $get_req:ident, $get_req_mod:ident, $mapper:ident,
     $create_req:ident, $insert:ident) => {
        pub struct $svc {
            pub state: std::sync::Arc<crate::state::AppState>,
        }

        #[async_trait::async_trait]
        impl proto::proto::audit::service::v1::$server_mod::$trait_ for $svc {
            async fn list(
                &self,
                request: Request<proto::proto::pagination::PagingRequest>,
            ) -> Result<Response<proto::proto::audit::service::v1::$list_resp>, Status> {
                let (rows, total) = fetch_paged(
                    &self.state.db,
                    store::entities::$entity::Entity::find(),
                    &request.into_inner(),
                )
                .await
                .map_err(|e| Status::internal(e.message))?;
                Ok(Response::new(
                    proto::proto::audit::service::v1::$list_resp {
                        items: rows.into_iter().map($mapper).collect(),
                        total: total as i64,
                    },
                ))
            }

            async fn get(
                &self,
                request: Request<proto::proto::audit::service::v1::$get_req>,
            ) -> Result<Response<proto::proto::audit::service::v1::$proto>, Status> {
                let req = request.into_inner();
                let id = match req.query_by {
                    Some(q) => match q {
                        auditv1::$get_req_mod::QueryBy::Id(id) => id as i64,
                    },
                    None => return Err(bad("query_by required")),
                };
                let row = store::entities::$entity::Entity::find_by_id(id as i64)
                    .one(&self.state.db)
                    .await
                    .map_err(db_status)?
                    .ok_or_else(|| not_found("audit log"))?;
                Ok(Response::new($mapper(row)))
            }

            async fn create(
                &self,
                request: Request<proto::proto::audit::service::v1::$create_req>,
            ) -> Result<Response<pbjson_types::Empty>, Status> {
                let req = request.into_inner();
                let Some(data) = req.data else {
                    return Err(bad("data required"));
                };
                $insert(&self.state.db, data).await?;
                Ok(Response::new(pbjson_types::Empty {}))
            }
        }
    };
}

pub(crate) use audit_impl;
