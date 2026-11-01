use std::process::Command;
use std::env;

fn main() {
    // Rust toolchain version
    let rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    // Linker-wrapper diagnostic, not the musl libc version or linked-archive proof.
    // Only stderr is read; wrappers printing their version to stdout yield "unknown".
    let musl_gcc = Command::new("musl-gcc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stderr).lines().next()
                     .unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "not found".into());

    // Diagnostic sysroot location when libc.a exists, not an extracted version.
    let musl_sysroot_version = musl_version_from_sysroot()
        .unwrap_or_else(|| "unknown".into());

    // Build timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into());

    println!("cargo:rustc-env=BUILD_RUSTC_VERSION={}", rustc);
    println!("cargo:rustc-env=BUILD_MUSL_GCC_VERSION={}", musl_gcc);
    println!("cargo:rustc-env=BUILD_MUSL_LIBC_VERSION={}", musl_sysroot_version);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
    println!("cargo:rustc-env=BUILD_TARGET={}", env::var("TARGET").unwrap_or_default());

    // This explicit file trigger watches build.rs, not compiler or wrapper changes.
    println!("cargo:rerun-if-changed=build.rs");
}

/// Report the PATH rustc sysroot if its x86-64 musl libc.a exists.
/// This does not extract a musl version or establish which archive was linked.
fn musl_version_from_sysroot() -> Option<String> {
    let sysroot = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    // The archive's presence is diagnostic context only.
    let version_paths = [
        format!("{}/lib/rustlib/x86_64-unknown-linux-musl/lib/self-contained/libc.a", sysroot),
    ];

    // If the libc.a exists, we know the musl sysroot is present
    for p in &version_paths {
        if std::path::Path::new(p).exists() {
            // Can't easily extract version from .a without objdump/nm,
            // so record the sysroot path — correlate manually with toolchain release notes
            return Some(format!("bundled in Rust sysroot ({})", sysroot));
        }
    }

    None
}
