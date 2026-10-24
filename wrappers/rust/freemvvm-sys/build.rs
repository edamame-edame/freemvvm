use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=FREEMVVM_NATIVE_LIB_DIR");
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");

    let library_dir = match env::var_os("FREEMVVM_NATIVE_LIB_DIR") {
        Some(path) => PathBuf::from(path),
        None => default_library_dir(),
    };

    println!("cargo:rustc-link-search=native={}", library_dir.display());
}

fn default_library_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace_root = manifest_dir
        .ancestors()
        .nth(3)
        .expect("Rust wrapper is nested in the workspace")
        .to_path_buf();

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| absolutize(&workspace_root, path))
        .unwrap_or_else(|| workspace_root.join("target"));
    let profile = env::var_os("PROFILE").expect("Cargo provides PROFILE");
    let host = env::var_os("HOST").expect("Cargo provides HOST");
    let target = env::var_os("TARGET").expect("Cargo provides TARGET");

    let mut library_dir = target_dir;
    if target != host {
        library_dir.push(target);
    }
    library_dir.push(profile);
    library_dir
}

fn absolutize(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}
