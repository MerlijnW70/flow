// crates/utoipa-swagger-ui/build.rs
use std::{env, fs, path::PathBuf};

fn main() {
    // Define output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("embed.rs");

    // This file will be compiled-in during build.
    // In the official crate this is auto-generated via rust-embed macros.
    // For local builds we generate a safe placeholder to avoid missing OUT_DIR.
    let content = r#"
// Auto-generated stub embed file for local development.
// The real utoipa-swagger-ui build script would embed Swagger UI static assets here.

#[allow(dead_code)]
pub struct SwaggerUiDist;

impl SwaggerUiDist {
    pub fn get(_path: &str) -> Option<Vec<u8>> {
        None
    }
}
"#;

    fs::write(&dest, content).expect("failed to write embed.rs");

    // Ensure cargo rebuilds when this script changes
    println!("cargo:rerun-if-changed=build.rs");
    // Silence doc_cfg warnings
    println!("cargo::rustc-check-cfg=cfg(doc_cfg)");
}
