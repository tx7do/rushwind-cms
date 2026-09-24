//! The pass-through proxy kit — the one declarative macro the generated
//! `proxies` module is a table of. Each generated face expands to the
//! proxy struct + the BFF trait impl, its methods one of four kinds:
//!
//! - `pass` — the plain pass-through: forward with the operator stamp,
//!   return the inner response (the BFF and domain methods share their
//!   message types by contract, so no mapping rides here);
//! - `drop` — forward untouched, discard the domain response body (the
//!   BFF face returns `Empty` where the domain returns the entity);
//! - `stub` — the contract carries the method but no domain face backs
//!   it on this deployment yet: always "not implemented";
//! - `hand` — delegate to the crate's hand-written behavior module (the
//!   BFF-only semantics a bare forward cannot carry).
//!
//! Hand-written faces that are not proxies at all (authentication —
//! captcha/cookie concerns; admin-portal aggregation; file-transfer
//! multipart) live in their own modules. Regenerate `proxies.rs` via
//! scripts/gen-proxies.py when the contract re-syncs.

/// Expands one pass-through proxy: the struct + the BFF trait impl.
#[macro_export]
macro_rules! passthrough_proxy {
    (
        $(#[$meta:meta])*
        $($face:ident)::+ for $proxy:ident {
            client: $client:ty,
            behaviors: $behaviors:ident,
            methods: [$($kind:ident $method:ident($req:ty) -> $resp:ty $(=> $target:ident)? ,)*]
        }
    ) => {
        $(#[$meta])*
        pub struct $proxy {
            pub state: ::std::sync::Arc<$crate::state::AppState>,
        }

        #[async_trait::async_trait]
        impl $($face)::+ for $proxy {
            $(
                async fn $method(
                    &self,
                    _ctx: rushwind_http_binding::ctx::RequestContext,
                    req: $req,
                ) -> ::std::result::Result<$resp, $crate::state::StatusError> {
                    // The method-context values ride as arguments — the
                    // inner expansion cannot see the outer `self`/`_ctx`.
                    $crate::passthrough_proxy!(@body
                        $kind $method($req) -> $resp $(=> $target)?
                        , client: $client, behaviors: $behaviors
                        , state: &self.state, ctx: &_ctx, msg: req
                    )
                }
            )*
        }
    };

    // pass — the operator-stamped forward, the inner response back.
    (@body pass $method:ident($req:ty) -> $resp:ty,
        client: $client:ty, behaviors: $behaviors:ident,
        state: $state:expr, ctx: $ctx:expr, msg: $msg:expr) => {
        {
            let mut core = <$client>::new($state.core_channel.clone());
            Ok(core
                .$method($crate::services::with_operator($ctx, $msg))
                .await
                .map_err($crate::services::map_status)?
                .into_inner())
        }
    };

    // drop — the forward whose domain response body the face discards.
    (@body drop $method:ident($req:ty) -> $resp:ty,
        client: $client:ty, behaviors: $behaviors:ident,
        state: $state:expr, ctx: $ctx:expr, msg: $msg:expr) => {
        {
            let mut core = <$client>::new($state.core_channel.clone());
            let _ = core
                .$method(tonic::Request::new($msg))
                .await
                .map_err($crate::services::map_status)?;
            Ok(<$resp>::default())
        }
    };

    // stub — the unbacked contract method.
    (@body stub $method:ident($req:ty) -> $resp:ty,
        client: $client:ty, behaviors: $behaviors:ident,
        state: $state:expr, ctx: $ctx:expr, msg: $msg:expr) => {
        {
            let _ = ($msg, $state);
            Err($crate::state::internal_error("not implemented"))
        }
    };

    // hand — the delegation into the behavior module.
    (@body hand $method:ident($req:ty) -> $resp:ty => $target:ident,
        client: $client:ty, behaviors: $behaviors:ident,
        state: $state:expr, ctx: $ctx:expr, msg: $msg:expr) => {
        { $crate::services::$behaviors::$target($state, $ctx, $msg).await }
    };
}
