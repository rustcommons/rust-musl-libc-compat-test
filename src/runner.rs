use crate::kernel::KernelVersion;
use crate::security::{MacFramework, check_exec_location};

/// A single compatibility test entry.
pub struct Test {
    pub name: &'static str,
    /// Minimum (major, minor) required to run this test.
    /// None means the test is safe on any kernel we care about.
    pub min_kernel: Option<(u32, u32)>,
    /// Human-readable explanation of why the minimum exists.
    pub reason: Option<&'static str>,
    pub run: fn() -> Result<(), String>,
}

/// Outcome of a single test execution.
enum Outcome {
    Pass,
    Skip {
        requires: (u32, u32),
        reason: &'static str,
    },
    SkipSecurity(String),
    Fail(String),
}

/// Run all registered tests against `kernel`, print results, return exit code.
pub fn run_all(
    tests: &[Test],
    kernel: &KernelVersion,
    build: &crate::BuildInfo,
    output: &mut crate::output::Output<impl std::io::Write>,
) -> i32 {
    let mac = MacFramework::detect();

    output.line(format_args!("=== musl ABI compatibility test ==="));
    output.line(format_args!("kernel   : {}", kernel));
    output.line(format_args!("arch     : {}", std::env::consts::ARCH));
    output.line(format_args!("security : {}", mac.summary()));
    output.line(format_args!("uid      : {}", unsafe { libc::getuid() }));
    output.line(format_args!("rustc    : {}", build.rustc_version));
    output.line(format_args!("musl libc: {}", build.musl_libc_version));

    // Warn about exec-from-tmp before anything else
    if let Some(warn) = check_exec_location() {
        output.line(format_args!("\n  WARNING: {}\n", warn));
    }

    if mac.may_restrict() {
        output.line(format_args!(
            "\n  NOTE: running under a confined security context — some \
                  tests may fail or be skipped due to MAC policy.\n"
        ));
    }

    output.line(format_args!(""));

    let (mut passed, mut skipped, mut failed) = (0u32, 0u32, 0u32);

    for t in tests {
        let outcome = evaluate(t, kernel, &mac);

        match &outcome {
            Outcome::Pass => {
                output.line(format_args!("  PASS  {}", t.name));
                passed += 1;
            }
            Outcome::Skip { requires, reason } => {
                output.line(format_args!(
                    "  SKIP  {:<45} (needs {}.{}: {})",
                    t.name, requires.0, requires.1, reason
                ));
                skipped += 1;
            }
            Outcome::SkipSecurity(reason) => {
                output.line(format_args!(
                    "  SKIP  {:<45} (security: {})",
                    t.name, reason
                ));
                skipped += 1;
            }
            Outcome::Fail(msg) => {
                output.line(format_args!("  FAIL  {:<45}  →  {}", t.name, msg));
                failed += 1;
            }
        }
    }

    output.line(format_args!(
        "\nresults: {} passed, {} skipped, {} failed",
        passed, skipped, failed
    ));

    if failed > 0 { 1 } else { 0 }
}

fn evaluate(t: &Test, kernel: &KernelVersion, mac: &MacFramework) -> Outcome {
    // Kernel gate first
    if let Some((maj, min)) = t.min_kernel {
        if !kernel.at_least(maj, min) {
            return Outcome::Skip {
                requires: (maj, min),
                reason: t.reason.unwrap_or("kernel version too old"),
            };
        }
    }

    // Security gate: if we're confined, skip tests known to be policy-sensitive
    if mac.may_restrict() && is_mac_sensitive(t.name) {
        return Outcome::SkipSecurity(
            "test uses syscalls that are commonly denied in confined domains".into(),
        );
    }

    match (t.run)() {
        Ok(()) => Outcome::Pass,
        Err(msg) => Outcome::Fail(msg),
    }
}

/// Tests we proactively skip under a confined MAC context rather than letting
/// them fail with a confusing error.
fn is_mac_sensitive(name: &str) -> bool {
    matches!(name, "syscall: memfd_create(2)" | "syscall: renameat2(2)")
}
