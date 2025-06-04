//! cargo-gpu build script.
fn cache_dir() -> std::path::PathBuf {
    let dir = directories::BaseDirs::new()
        .unwrap()
        .cache_dir()
        .join("rust-gpu");

    if cfg!(test) {
        let thread_id = std::thread::current().id();
        let id = format!("{thread_id:?}").replace('(', "-").replace(')', "");
        dir.join("tests").join(id)
    } else {
        dir
    }
}

fn main() {
    std::fs::create_dir_all(cache_dir()).unwrap();

    assert!(cache_dir().exists());

    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_or_else(
            |_| "unknown".to_owned(),
            |output| String::from_utf8(output.stdout).unwrap_or_else(|_| "unknown".to_owned()),
        );
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
