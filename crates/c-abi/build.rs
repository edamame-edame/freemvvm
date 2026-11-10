use std::env;
use std::fs;
use std::path::PathBuf;

const WINDOWS_FEATURE: &str = "presentation-backend-windows-win32-d2d";
const LINUX_FEATURE: &str = "presentation-backend-linux-drm-gbm-egl-gles";

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_PRESENTATION_BACKEND_WINDOWS_WIN32_D2D");
    println!(
        "cargo:rerun-if-env-changed=CARGO_FEATURE_PRESENTATION_BACKEND_LINUX_DRM_GBM_EGL_GLES"
    );
    println!("cargo:rerun-if-env-changed=FREEMVVM_TARGET_PROFILE");

    let target = env::var("TARGET").expect("Cargo must provide TARGET");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo must provide target OS");
    let target_arch =
        env::var("CARGO_CFG_TARGET_ARCH").expect("Cargo must provide target architecture");
    let windows_enabled =
        env::var_os("CARGO_FEATURE_PRESENTATION_BACKEND_WINDOWS_WIN32_D2D").is_some();
    let linux_enabled =
        env::var_os("CARGO_FEATURE_PRESENTATION_BACKEND_LINUX_DRM_GBM_EGL_GLES").is_some();
    let default_profile = match (target_os.as_str(), target_arch.as_str()) {
        ("windows", "x86_64") => "windows-11-x64",
        ("linux", "x86_64") => "ubuntu-24.04-x64",
        _ => "unsupported-target",
    };
    let profile = env::var("FREEMVVM_TARGET_PROFILE").unwrap_or_else(|_| default_profile.into());
    let profile_is_compatible = match profile.as_str() {
        "windows-11-x64" => target_os == "windows" && target_arch == "x86_64" && windows_enabled,
        "ubuntu-24.04-x64" | "debian-12-vm-x64" => {
            target_os == "linux" && target_arch == "x86_64" && linux_enabled
        }
        _ => false,
    };
    if !profile_is_compatible {
        panic!(
            "unsupported FreeMvvm UI artifact profile `{profile}` for target `{target}` and enabled backend features"
        );
    }
    println!("cargo:rustc-env=FREEMVVM_TARGET_PROFILE={profile}");

    let requested_features = [
        (WINDOWS_FEATURE, windows_enabled),
        (LINUX_FEATURE, linux_enabled),
    ]
    .into_iter()
    .filter_map(|(name, enabled)| enabled.then_some(name))
    .collect::<Vec<_>>();
    let (backend_id, backend_feature) = match target_os.as_str() {
        "windows" if windows_enabled => (Some("windows.win32-d2d"), Some(WINDOWS_FEATURE)),
        "linux" if linux_enabled => (Some("linux.drm-gbm-egl-gles"), Some(LINUX_FEATURE)),
        _ => (None, None),
    };
    let active_features = backend_feature.into_iter().collect::<Vec<_>>();
    let identity = format!(
        "0.0.0-abi1-{profile}-{target}+{}",
        requested_features.join("+")
    );
    let requested_json = json_array(&requested_features);
    let active_json = json_array(&active_features);
    let backend_json = backend_id.map_or_else(|| "[]".to_owned(), |id| json_array(&[id]));
    let metadata = format!(
        "{{\"schema\":1,\"runtime_version\":\"0.0.0\",\"abi_major\":1,\"target_triple\":\"{target}\",\"target_profile\":\"{profile}\",\"requested_backend_features\":{requested_json},\"active_backend_features\":{active_json},\"presentation_backends\":{backend_json},\"artifact_identity\":\"{identity}\"}}\n"
    );

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must provide OUT_DIR"));
    fs::write(out_dir.join("presentation-backend-artifact.json"), metadata)
        .expect("write presentation backend artifact metadata");
}

fn json_array(values: &[&str]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}
