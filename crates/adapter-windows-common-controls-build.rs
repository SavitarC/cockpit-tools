use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../windows-common-controls-v6.rc");
    println!("cargo:rerun-if-changed=../windows-common-controls-v6.manifest");

    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is required"));
    let shared_dir = manifest_dir
        .parent()
        .expect("adapter crates must live under the shared crates directory");
    let resource_path = shared_dir.join("windows-common-controls-v6.rc");

    embed_resource::compile(
        resource_path,
        embed_resource::ParamsIncludeDirs([shared_dir.as_os_str()]),
    )
    .manifest_required()
    .expect("embed Windows Common Controls v6 manifest");
}
