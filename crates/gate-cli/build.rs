// Capture the build target triple and rustc version at compile time so the
// harness can report them in --json without touching std::env / std::process at
// runtime (kept clean for the wasm builds and the covenant posture). Build
// scripts run on the host and may use std freely.
use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());
    println!("cargo:rustc-env=GATE_TARGET={target}");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let version = Command::new(rustc)
        .arg("-V")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=GATE_RUSTC={version}");

    println!("cargo:rerun-if-changed=build.rs");
}
