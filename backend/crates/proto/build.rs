//! cms-proto build.
//!
//! Compiles the synced CMS contract tree (`backend/api/protos`, checksummed
//! by `backend/api/sync-protos.sh`) together with its vendored dependencies
//! (`backend/api/third_party`, see its PROVENANCE.md) into BOTH faces of the
//! contract:
//!
//! * the annotated descriptor set — the FULL compile closure including the
//!   annotation declarations (google.api.http, errors.code, redact,
//!   validate, gnostic) — produced by `buf build` (api/buf.yaml workspace),
//!   NOT protox: protox's serializer drops custom-option bytes, which are
//!   the entire point of this set. `rushwind-gen-http` parses it for
//!   routes/error tables/binding plans; the runtime pool decodes it as the
//!   schema surface for protojson serialization and form binding. Requires
//!   `buf` on PATH.
//! * the Rust types — prost + pbjson, from a FILTERED descriptor set built
//!   by protox: only data-carrying files (the contract files, the
//!   pagination messages, the well-known types referenced as field
//!   types). Annotation-only files are removed so prost/pbjson never emit
//!   types for them (and so `google/protobuf/descriptor.proto` — pulled in
//!   by the gnostic annotations — never enters type generation, where the
//!   `.google.protobuf` extern would produce unresolvable references).
//!
//! Kept files additionally have their `dependency` lists pruned to kept
//! files: option-bearing imports carry no type references, so the dangling
//! edges are harmless for codegen while keeping the synced protos untouched.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use protox::prost::Message as _;

// The deployments' whitelist union, single-sourced from src/auth_free.rs —
// the same file the lib re-exports (included at build-script scope so the
// generator config can consume it).
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/auth_free.rs"));

/// Vendored data-carrying files (the paging envelope). These are real
/// runtime types (List request inputs) and participate in type generation.
const VENDORED_DATA_FILES: [&str; 2] = [
    "pagination/v1/pagination.proto",
    "google/api/httpbody.proto",
];

/// Well-known types referenced as field types by the contract tree.
/// Externed to `pbjson_types` by the prost config below; kept in the filtered
/// descriptor set so references resolve, excluded from pbjson output.
const WELL_KNOWN_PREFIX: &str = "google/protobuf/";

fn is_well_known(name: &str) -> bool {
    name.starts_with(WELL_KNOWN_PREFIX)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // The crate sits at backend/crates/proto — two levels up is backend/.
    let backend_root = manifest_dir.join("../..");
    let proto_root = backend_root.join("api/protos");
    let third_party_root = backend_root.join("api/third_party");

    // CMS contract files + vendored data files: the codegen request set.
    let mut compile_files: Vec<PathBuf> = Vec::new();
    collect_protos(&proto_root, &mut compile_files)?;
    if compile_files.is_empty() {
        return Err("no CMS protos found under backend/api/protos".into());
    }
    for rel in VENDORED_DATA_FILES {
        let p = third_party_root.join(rel);
        if !p.is_file() {
            return Err(format!("vendored data file missing: {}", p.display()).into());
        }
        compile_files.push(p);
    }

    // Canonical input order: directory enumeration order is filesystem-defined
    // (NTFS yields name order, ext4 hash order); the sort keeps the protox
    // types face deterministic across platforms. (The annotated closure no
    // longer depends on this list at all — buf build emits the whole
    // workspace in its own sorted order.)
    compile_files.sort();

    // The contract tree's own top-level modules — derived from the tree
    // itself so a new module directory needs no whitelist edit. The types
    // filter keeps a file when its top-level segment is one of these
    // modules, or it is an explicitly vendored data file, or a well-known
    // import.
    let mut cms_tops: Vec<String> = fs::read_dir(&proto_root)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    cms_tops.sort();
    let is_types_kept = |name: &str| -> bool {
        match name.split_once('/') {
            Some((top, _)) => {
                cms_tops.iter().any(|t| t == top)
                    || VENDORED_DATA_FILES.contains(&name)
                    || name.starts_with(WELL_KNOWN_PREFIX)
            }
            None => VENDORED_DATA_FILES.contains(&name) || name.starts_with(WELL_KNOWN_PREFIX),
        }
    };

    let includes = [proto_root.as_path(), third_party_root.as_path()];

    // 1. The annotated full closure via buf build (option bytes preserved,
    //    imports resolved through the api/buf.yaml workspace). Output
    //    ordering is buf-determined (sorted), keeping the generated ROUTES
    //    table platform-independent.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let annotated_path = out_dir.join("annotated_descriptor.bin");
    let mut cmd = Command::new("buf");
    cmd.current_dir(backend_root.join("api"))
        .args(["build", "--exclude-source-info", "--output"])
        .arg(&annotated_path);
    let output = cmd.output().map_err(|e| format!("buf: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "buf build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let annotated_size = fs::metadata(&annotated_path).map(|m| m.len()).unwrap_or(0);

    // 2. The types face via protox: the full closure (option bytes unused
    //    here) filtered to data-carrying files.
    let full_fds = protox::compile(&compile_files, includes)?;

    // Filtered set for type generation: drop annotation-only files, prune
    // dependency edges to kept files.
    let mut types_fds = full_fds.clone();
    types_fds.file.retain(|f| is_types_kept(f.name()));
    for f in &mut types_fds.file {
        let deps: Vec<String> = f.dependency.to_vec();
        f.dependency = deps
            .into_iter()
            .filter(|d| is_types_kept(d.as_str()))
            .collect();
    }

    // Filtered closure → types_descriptor.bin (prost + pbjson input).
    let types_path = out_dir.join("types_descriptor.bin");
    fs::write(&types_path, types_fds.encode_to_vec())?;

    // Sweep stale codegen artifacts (earlier runs may have written packages
    // that the filter has since dropped).
    for entry in fs::read_dir(&out_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let _ = fs::remove_file(&path);
        }
    }

    eprintln!(
        "[cms-proto] annotated closure: {annotated_size} bytes (buf build); types set: {} files (annotation declarations dropped)",
        types_fds.file.len()
    );

    // prost type generation, reading the FILTERED descriptor set verbatim.
    // skip_protoc_run is essential: without it prost-build would invoke
    // protoc on the request files and OVERWRITE types_descriptor.bin with
    // the full closure, resurrecting the annotation packages.
    //
    // The tonic service generator rides the SAME pass: the domain gRPC
    // services (and, harmlessly, the BFF services) get their tonic
    // server/client faces emitted beside their message types — one type
    // tree, zero duplication. The core deployment serves these; the BFF
    // deployments dial them as clients.
    let mut config = prost_build::Config::new();
    config
        .file_descriptor_set_path(&types_path)
        .skip_protoc_run()
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .include_file("proto_include.rs");
    config.service_generator(
        tonic_prost_build::configure()
            .build_server(true)
            .build_client(true)
            // The trait methods default to Unimplemented — the core
            // deployment overrides them service by service.
            .generate_default_stubs(true)
            .service_generator(),
    );
    config.compile_protos(&compile_files, &includes)?;

    // pbjson serde impls: data packages of the filtered set only. The package
    // list is derived from the set itself (dir names do not map 1:1 to
    // package names); well-knowns are externed and excluded.
    let mut packages: Vec<String> = types_fds
        .file
        .iter()
        .filter(|f| !is_well_known(f.name()))
        .map(|f| format!(".{}", f.package()))
        .collect();
    packages.sort();
    packages.dedup();
    let pkg_refs: Vec<&str> = packages.iter().map(|s| s.as_str()).collect();
    pbjson_build::Builder::new()
        .register_descriptors(&fs::read(&types_path)?)?
        .build(&pkg_refs)?;

    // 3. The generated route/binding/trait/mount surface: the framework
    //    generator (rushwind-gen-http) over the annotated closure — TWICE,
    //    once per BFF face. The generator names service traits by short
    //    name, and the admin and the app BFF both carry CategoryService,
    //    CommentService &c., so the two faces cannot share one module.
    //    Each face compiles from the sub-closure reachable from its own
    //    BFF tree (the BFF files plus their transitive imports); the
    //    emitted self-references (`crate::gen::` — the generator's
    //    hardcoded module path) are rewritten to the face's own module.
    //    Generated types still resolve through `crate::proto` and the
    //    pool through `crate::pool()`.
    let annotated = fs::read(&annotated_path)?;
    let cfg = rushwind_gen_http::CodegenConfig {
        proto_module_path: "crate::proto",
        pool_expr: "crate::pool()",
        auth_free: AUTH_FREE,
    };
    for (face, roots) in [("admin", "admin/service/v1/"), ("app", "app/service/v1/")] {
        let sub = sub_closure(&annotated, roots)?;
        eprintln!(
            "[cms-proto] {face} face sub-closure: {} bytes (full: {})",
            sub.len(),
            annotated.len()
        );
        let src = match rushwind_gen_http::generate_from_bytes(&sub, &cfg) {
            Ok(src) => src,
            Err(e) => panic!("gen-rust code generation failed ({face} face): {e}"),
        };
        let module = format!("crate::gen_{face}::");
        let src = src.replace("crate::gen::", &module);
        fs::write(out_dir.join(format!("{face}_gen.rs")), src)?;
    }

    // Re-run on any contract change (and on the auth-free table).
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("src/auth_free.rs").display()
    );
    println!("cargo:rerun-if-changed={}", third_party_root.display());
    println!("cargo:rerun-if-changed={}", proto_root.display());

    Ok(())
}

fn collect_protos(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_protos(&path, out)?;
        } else if path.extension().map(|e| e == "proto").unwrap_or(false) {
            out.push(path);
        }
    }
    Ok(())
}

/// Slices the annotated closure to the files reachable (transitively
/// through `dependency` edges) from the files whose names start with
/// `root_prefix` — one BFF face's compile sub-closure.
///
/// The slice works at the BYTE level and never re-encodes a
/// FileDescriptorProto: prost's decoder discards unknown fields, which is
/// exactly where the custom-option bytes (google.api.http / errors.code)
/// ride, so a decode+re-encode round-trip would strip every annotation.
/// Each file record is copied verbatim; only the top-level record set is
/// filtered. Dependencies of retained files are retained by construction
/// (the reachability walk adds them), so no dependency-list rewrite is
/// needed.
fn sub_closure(annotated: &[u8], root_prefix: &str) -> Result<Vec<u8>, String> {
    // Split the FileDescriptorSet into per-file records, harvesting each
    // record's name (field 1) and dependencies (field 3) with a minimal
    // walk that leaves every other field's bytes untouched.
    let mut files: Vec<(Vec<u8>, String, Vec<String>)> = Vec::new();
    let mut top = records(annotated)?;
    for (field, wire, payload) in top.drain(..) {
        if field != 1 || wire != 2 {
            continue;
        }
        let mut name = String::new();
        let mut deps = Vec::new();
        for (field, wire, value) in records(payload)? {
            match (field, wire) {
                (1, 2) if name.is_empty() => {
                    name = String::from_utf8_lossy(value).into_owned();
                }
                (3, 2) => deps.push(String::from_utf8_lossy(value).into_owned()),
                _ => {}
            }
        }
        files.push((payload.to_vec(), name, deps));
    }

    // Reachability from the root prefix.
    let mut keep = std::collections::HashSet::new();
    let mut queue: Vec<String> = files
        .iter()
        .map(|(_, name, _)| name.clone())
        .filter(|n| n.starts_with(root_prefix))
        .collect();
    let deps_of = |name: &str| -> &[String] {
        files
            .iter()
            .find(|(_, n, _)| n == name)
            .map(|(_, _, d)| d.as_slice())
            .unwrap_or(&[])
    };
    while let Some(name) = queue.pop() {
        if keep.insert(name.clone()) {
            queue.extend(deps_of(&name).iter().cloned());
        }
    }

    // Re-emit the retained records verbatim (original order, field-1
    // length-delimited framing).
    let mut out = Vec::with_capacity(annotated.len());
    for (blob, name, _) in &files {
        if keep.contains(name) {
            write_tag(&mut out, 1, 2);
            write_varint(&mut out, blob.len() as u64);
            out.extend_from_slice(blob);
        }
    }
    Ok(out)
}

/// One wire-level record: (field number, wire type, payload slice).
type Record<'a> = (u32, u8, &'a [u8]);

/// The wire-level record walk: yields `(field number, wire type, payload)`
/// for length-delimited records, or empty payloads after consuming
/// scalar records.
fn records(buf: &[u8]) -> Result<Vec<Record<'_>>, String> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < buf.len() {
        let key = read_varint(buf, &mut pos)?;
        let field = (key >> 3) as u32;
        let wire = (key & 7) as u8;
        match wire {
            0 => {
                read_varint(buf, &mut pos)?;
                out.push((field, wire, &[][..]));
            }
            1 => {
                let end = pos.checked_add(8).filter(|e| *e <= buf.len());
                out.push((field, wire, buf.get(pos..end.unwrap_or(pos)).unwrap_or(&[])));
                pos = end.ok_or("truncated fixed64")?;
            }
            2 => {
                let len = read_varint(buf, &mut pos)? as usize;
                let end = pos
                    .checked_add(len)
                    .filter(|e| *e <= buf.len())
                    .ok_or("truncated length-delimited record")?;
                out.push((field, wire, &buf[pos..end]));
                pos = end;
            }
            5 => {
                let end = pos.checked_add(4).filter(|e| *e <= buf.len());
                out.push((field, wire, buf.get(pos..end.unwrap_or(pos)).unwrap_or(&[])));
                pos = end.ok_or("truncated fixed32")?;
            }
            other => return Err(format!("unsupported wire type {other}")),
        }
    }
    Ok(out)
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let b = *buf.get(*pos).ok_or("truncated varint")?;
        *pos += 1;
        result |= u64::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err("varint too long".into());
        }
    }
}

fn write_tag(out: &mut Vec<u8>, field: u32, wire: u8) {
    write_varint(out, (u64::from(field) << 3) | u64::from(wire));
}

fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}
