use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("exampleservice_descriptor.bin"))
        .compile(&["proto/exampleservice.proto"], &["proto"])
        .unwrap();

    println!("cargo:rerun-if-changed=proto/exampleservice.proto");

    println!("cargo:rustc-cfg=tokio_unstable");

    Ok(())
}
