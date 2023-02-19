use std::env;

const ENV_PROFILE: &str = "MICROCHASSIS_BUILD_PROFILE";
const ENV_MICROCHASSIS_BUILD_PROFILE: &str = "MICROCHASSIS_BUILD_PROFILE";

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed={ENV_PROFILE}");
    println!("cargo:rerun-if-env-changed={ENV_MICROCHASSIS_BUILD_PROFILE}");

    let cargo_profile = env::var(ENV_PROFILE).ok();
    let microchassis_build_profile = env::var(ENV_MICROCHASSIS_BUILD_PROFILE).ok();

    let profile = match microchassis_build_profile.as_deref() {
        Some(profile) => profile,
        None => match cargo_profile.as_deref() {
            Some("release") => "release",
            _ => "debug",
        },
    };

    println!("cargo:rustc-env={ENV_MICROCHASSIS_BUILD_PROFILE}={profile}");

    // TODO: configurable
    println!("cargo:rustc-cfg=tokio_unstable");

    Ok(())
}
