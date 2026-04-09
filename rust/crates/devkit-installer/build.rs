use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=DEVKIT_INSTALLER_PAYLOAD");
    println!("cargo:rerun-if-env-changed=DEVKIT_CLEANUP_HELPER");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));
    let generated = out_dir.join("embedded_payload.rs");
    let payload = embedded_const(
        "DEVKIT_INSTALLER_PAYLOAD",
        "devkit-payload.exe",
        "EMBEDDED_PAYLOAD",
        &out_dir,
    );
    let helper = embedded_const(
        "DEVKIT_CLEANUP_HELPER",
        "devkit-cleanup-helper.exe",
        "EMBEDDED_CLEANUP_HELPER",
        &out_dir,
    );
    fs::write(generated, format!("{payload}{helper}"))
        .expect("failed to write embedded payload module");
}

fn embedded_const(env_name: &str, copied_name: &str, const_name: &str, out_dir: &Path) -> String {
    match env::var_os(env_name) {
        Some(value) => {
            let source = PathBuf::from(value);
            println!("cargo:rerun-if-changed={}", source.display());

            if !source.is_file() {
                panic!("{env_name} does not point to a file: {}", source.display());
            }

            let copied = out_dir.join(copied_name);
            copy_payload(&source, &copied);
            format!(
                "pub const {const_name}: Option<&[u8]> = Some(include_bytes!(r#\"{}\"#));\n",
                copied.display()
            )
        }
        None => format!("pub const {const_name}: Option<&[u8]> = None;\n"),
    }
}

fn copy_payload(source: &Path, destination: &Path) {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).expect("failed to create payload output directory");
    }

    fs::copy(source, destination).expect("failed to copy installer payload");
}
