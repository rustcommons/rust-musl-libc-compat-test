/// src/main.rs
///
mod kernel;
mod output;
mod runner;
mod security;
mod tests;

/// Build provenance baked in at compile time.
pub struct BuildInfo {
    pub rustc_version: &'static str,
    pub musl_gcc_version: &'static str,
    pub musl_libc_version: &'static str,
    pub build_timestamp: &'static str,
    pub build_target: &'static str,
}

pub const BUILD: BuildInfo = BuildInfo {
    rustc_version: env!("BUILD_RUSTC_VERSION"),
    musl_gcc_version: env!("BUILD_MUSL_GCC_VERSION"),
    musl_libc_version: env!("BUILD_MUSL_LIBC_VERSION"),
    build_timestamp: env!("BUILD_TIMESTAMP"),
    build_target: env!("BUILD_TARGET"),
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let stdout = std::io::stdout();
    let mut output = output::Output::new(stdout.lock());
    let code = if args.iter().any(|a| a == "--version" || a == "-V") {
        print_version(&mut output);
        0
    } else {
        let kernel = kernel::KernelVersion::current();
        runner::run_all(&tests::all(), &kernel, &BUILD, &mut output)
    };
    let code = output.finish(code, &mut std::io::stderr().lock());
    std::process::exit(code);
}

fn print_version(output: &mut output::Output<impl std::io::Write>) {
    output.line(format_args!(
        "musl-compat-test {}",
        env!("CARGO_PKG_VERSION")
    ));
    output.line(format_args!("  rustc          : {}", BUILD.rustc_version));
    output.line(format_args!(
        "  musl-gcc       : {}",
        BUILD.musl_gcc_version
    ));
    output.line(format_args!(
        "  musl libc      : {}",
        BUILD.musl_libc_version
    ));
    output.line(format_args!("  build target   : {}", BUILD.build_target));
    output.line(format_args!("  build unix ts  : {}", BUILD.build_timestamp));
}

#[cfg(test)]
mod output_tests {
    use super::*;
    use std::io::{self, Write};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FailedWriter(io::ErrorKind);
    impl Write for FailedWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(self.0.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn suite_continues_after_write_errors_and_keeps_failed_probe_status() {
        static CALLED: AtomicUsize = AtomicUsize::new(0);
        let kernel = kernel::KernelVersion::parse("6.0.0");
        for kind in [io::ErrorKind::BrokenPipe, io::ErrorKind::StorageFull] {
            for fail in [false, true] {
                let probes = [runner::Test {
                    name: "output regression",
                    min_kernel: None,
                    reason: None,
                    run: if fail {
                        || {
                            CALLED.fetch_add(1, Ordering::SeqCst);
                            Err("probe failure".into())
                        }
                    } else {
                        || {
                            CALLED.fetch_add(1, Ordering::SeqCst);
                            Ok(())
                        }
                    },
                }];
                let before = CALLED.load(Ordering::SeqCst);
                let mut output = output::Output::new(FailedWriter(kind));
                let code = runner::run_all(&probes, &kernel, &BUILD, &mut output);
                assert_eq!(CALLED.load(Ordering::SeqCst), before + 1);
                assert_eq!(code, i32::from(fail));
                let mut diagnostics = Vec::new();
                assert_eq!(
                    output.finish(code, &mut diagnostics),
                    i32::from(fail || kind != io::ErrorKind::BrokenPipe)
                );
            }
        }
    }

    #[test]
    fn version_uses_fallible_output() {
        let mut bytes = Vec::new();
        let mut output = output::Output::new(&mut bytes);
        print_version(&mut output);
        assert_eq!(output.finish(0, &mut Vec::new()), 0);
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .starts_with("musl-compat-test ")
        );
        for kind in [io::ErrorKind::BrokenPipe, io::ErrorKind::StorageFull] {
            let mut output = output::Output::new(FailedWriter(kind));
            print_version(&mut output);
            assert_eq!(
                output.finish(0, &mut Vec::new()),
                i32::from(kind != io::ErrorKind::BrokenPipe)
            );
        }
    }
}
