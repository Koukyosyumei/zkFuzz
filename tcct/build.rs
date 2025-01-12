use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_hash = String::from_utf8(output.stdout).unwrap().trim().to_string();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
