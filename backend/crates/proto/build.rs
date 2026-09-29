//! cms-proto build — the config face. The engine (the buf compile of
//! the api workspace yielding both the source-free annotated closure
//! and, by filtered decode, the prost+pbjson types with the tonic
//! service generator, plus the per-BFF gen-http route surfaces) lives
//! in `rushwind-proto-build`; this file carries the deployment's
//! knobs: the two dependency data files, the whitelist union, and the
//! two BFF faces — each sliced from the annotated closure by its own
//! root prefix and re-pointed to its own module (the admin and app
//! BFFs carry same-named services, so the faces cannot share one
//! module).

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/auth_free.rs"));

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rushwind_proto_build::run(rushwind_proto_build::Build {
        manifest_dir: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        dep_data_files: &[
            "pagination/v1/pagination.proto",
            "google/api/httpbody.proto",
        ],
        auth_free: AUTH_FREE,
        redact_plan_expr: Some("crate::redact_plan()"),
        proto_module_path: "crate::proto",
        pool_expr: "crate::pool()",
        tonic: true,
        faces: &[
            rushwind_proto_build::Face {
                label: "admin",
                module_file: "admin_gen.rs",
                root_prefix: Some("admin/service/v1/"),
                module_path: Some("crate::gen_admin::"),
            },
            rushwind_proto_build::Face {
                label: "app",
                module_file: "app_gen.rs",
                root_prefix: Some("app/service/v1/"),
                module_path: Some("crate::gen_app::"),
            },
        ],
    })
}
