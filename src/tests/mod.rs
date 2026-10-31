/// src/tests/mod.rs
///
pub mod env;
pub mod filesystem;
pub mod network;
pub mod process;
pub mod syscalls;
mod support;
pub mod threading;

use crate::runner::Test;

/// Macro to reduce boilerplate in test table declarations.
///
/// always:
///   test!(name, always, fn_ptr)
///
/// kernel-gated:
///   test!(name, since major.minor, "reason string", fn_ptr)
macro_rules! test {
    ($name:expr, always, $f:expr) => {
        Test { name: $name, min_kernel: None, reason: None, run: $f }
    };
    // Use comma between major and minor — dot gets eaten as a float literal
    ($name:expr, since $maj:literal, $min:literal, $why:expr, $f:expr) => {
        Test {
            name:       $name,
            min_kernel: Some(($maj, $min)),
            reason:     Some($why),
            run:        $f,
        }
    };
}

/// Returns the full ordered test suite.
pub fn all() -> Vec<Test> {
    vec![
        test!("env: set/get round-trip",     always, env::round_trip),
        test!("env: iterate environment",    always, env::iterate),
        test!("fs: create / write / read",   always, filesystem::read_write),
        test!("fs: symlink creation",        always, filesystem::symlink),
        test!("fs: directory traversal",     always, filesystem::dir_walk),
        test!("thread: spawn + join",        always, threading::spawn_join),
        test!("thread: TLS isolation",       always, threading::tls_isolation),
        test!("thread: barrier sync",        always, threading::barrier),
        test!("net: TCP loopback",           always, network::tcp_loopback),
        test!("time: monotonic clock",       always, process::monotonic_clock),
        test!("time: CLOCK_BOOTTIME",        always, process::clock_boottime),
        test!("proc: PID / /proc/self/exe",  always, process::proc_self),

        test!("syscall: getrandom(2)",
              since 3, 17, "SYS_getrandom added in 3.17",
              syscalls::getrandom),

        test!("syscall: memfd_create(2)",
              since 3, 17, "SYS_memfd_create added in 3.17",
              syscalls::memfd_create),

        test!("syscall: renameat2(2)",
              since 3, 15, "SYS_renameat2 added in 3.15",
              syscalls::renameat2),
    ]
}

