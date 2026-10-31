# User guide

`musl-compat-test` is a diagnostic executable with 15 runtime checks. It exercises environment access, filesystem operations, threads, TCP loopback, timing, process metadata and selected syscalls. A clean run is evidence about those checks on that host, not certification of other applications.

## Build and verify

Follow the [README source setup and artifact checks](README.md). Build tools belong on the build machine; a target host only needs the runtime facilities described there. The configured target is `x86_64-unknown-linux-musl`; other targets are not established by these instructions.

From the repository root after prerequisites are installed:

```sh
cargo build --release --locked
file target/x86_64-unknown-linux-musl/release/musl-compat-test
ldd target/x86_64-unknown-linux-musl/release/musl-compat-test
readelf -d target/x86_64-unknown-linux-musl/release/musl-compat-test
```

Inspect all three outputs: a static x86-64 ELF, the expected non-dynamic `ldd` diagnostic, and no `NEEDED` dependencies. A missing tool or failed `readelf` invocation is not proof of static linkage. The expected `ldd` diagnostic may return nonzero, so these are individual inspection steps rather than a chain of success-status assertions.

## Run locally

```sh
./target/x86_64-unknown-linux-musl/release/musl-compat-test
```

No arguments runs the complete suite in registry order. `--version` or `-V` prints build information and exits without running checks:

```sh
./target/x86_64-unknown-linux-musl/release/musl-compat-test --version
```

These are the only implemented CLI modes. There is no help, filtering, JSON or self-check mode. Unknown arguments currently fall through to running the suite unless `--version` or `-V` is also present; do not use `--help` expecting a harmless usage display.

## Copy to another host

Replace `user@host` with your SSH destination. These commands copy to that user's home directory and execute there:

```sh
scp target/x86_64-unknown-linux-musl/release/musl-compat-test user@host:
ssh user@host './musl-compat-test --version'
ssh user@host './musl-compat-test'
```

Confirm that the destination has the expected architecture and permits execution. Neither a home directory nor `/tmp` is guaranteed executable under mount or security policy. If execution is denied, use an approved executable location and consult [troubleshooting](TROUBLESHOOT.md). Do not assume relabeling a file is an appropriate universal fix.

## Read results and exit status

On normal completion the process returns `0` if there are no failed checks, including when checks are skipped, and `1` if any check fails. Version mode returns `0`. A crash, signal or failure to launch does not follow that summary contract. For local inspection, capture the status immediately:

```sh
./target/x86_64-unknown-linux-musl/release/musl-compat-test
probe_status=$?
printf 'Exit status: %s\n' "$probe_status"
```

A remote SSH command also exposes the remote command's status, but connection or launch failures must be distinguished from a completed probe run. There is no structured output mode.

The [test matrix](TEST_MATRIX.md) is the canonical list of expected profiles. Kernel gating happens before security gating. `renameat2` requires 3.15; `getrandom` and `memfd_create` require 3.17. These thresholds do not establish a minimum supported kernel for the whole binary.

Illustrative excerpts below omit other output; they are not recorded validation results or promises about a distribution:

```text
# Host meeting the matrix assumptions, kernel >= 3.17, no detected confinement
  PASS  syscall: getrandom(2)
  PASS  syscall: memfd_create(2)
  PASS  syscall: renameat2(2)
results: 15 passed, 0 skipped, 0 failed

# Host meeting the matrix assumptions, kernel 3.12.x, no detected confinement
  SKIP  syscall: getrandom(2) (needs 3.17: SYS_getrandom added in 3.17)
  SKIP  syscall: memfd_create(2) (needs 3.17: SYS_memfd_create added in 3.17)
  SKIP  syscall: renameat2(2) (needs 3.15: SYS_renameat2 added in 3.15)
results: 12 passed, 3 skipped, 0 failed
```

Spacing is abbreviated. Distribution names, WSL usage and kernel strings alone do not guarantee a particular security configuration or result. Expected skips are successful handling of a limitation, not evidence that the skipped operation works.

## Build provenance

The run header includes kernel, architecture, security summary, UID, compiler information and a musl sysroot field. Version mode additionally displays package version, linker-wrapper information, build target and Unix timestamp.

These build fields are captured by `build.rs`; paths refer to the build machine even when the binary runs elsewhere. The musl field reports a sysroot where `libc.a` was found. It does not extract the musl version or independently establish which archive the linker used. The wrapper version field can be `unknown` because the script reads standard error from `musl-gcc --version`. Treat these fields as diagnostic context, alongside artifact inspection, not full provenance verification.

## Security scenarios

The probe detects SELinux via `/sys/fs/selinux/enforce` and otherwise checks for AppArmor under `/sys/kernel/security/apparmor`. It reads `/proc/self/attr/current` for context. Detection is limited by what these paths expose; “none detected” does not mean unrestricted execution. In particular, it does not detect seccomp restrictions.

- SELinux enforcing with a context containing `unconfined_t` is treated as unconfined. Other contexts under enforcing mode trigger the sensitive-test skips.
- AppArmor's exact profile string `unconfined` is treated as unconfined; other profile strings trigger skips. An unfamiliar executable is not necessarily unrestricted.
- Under detected confinement, `memfd_create` and `renameat2` are proactively skipped when their kernel gates allow them. `getrandom` is not security-gated. Other checks may still fail because of policy.

For a host meeting the matrix assumptions with kernel >= 3.17 and detected confinement, illustrative output includes:

```text
  SKIP  syscall: memfd_create(2) (security: test uses syscalls that are commonly denied in confined domains)
  SKIP  syscall: renameat2(2) (security: test uses syscalls that are commonly denied in confined domains)
results: 13 passed, 2 skipped, 0 failed
```

With SELinux detected, an executable path starting with `/tmp` triggers a warning about possible execution restrictions. The warning does not inspect the actual label or prove a denial. A binary blocked before startup cannot print it. Use an execution location allowed by the host policy; a warning itself is not a failed check.

## Current limitations

- The “monotonic clock” check reads `CLOCK_MONOTONIC` before and after a short sleep; it does not depend on wall-clock adjustments.
- Each filesystem/rename check creates a private, unique directory under `std::env::temp_dir()` (including `TMPDIR` on Linux). Normal success and error paths clean it up. Cleanup failures are reported without discarding the original test error; process aborts/signals can still leave directories. Use an existing writable temporary base.
- There is no built-in timeout; I/O or synchronization can hang. Release builds abort on panic.
- Direct syscall failures include operation context, OS error text and numeric errno captured before cleanup. Kernel availability, filesystem semantics and policy restrictions can differ even when a version gate passes.
- DNS, TLS trust, general application behavior and comprehensive libc compatibility are outside current coverage.

See [scope and architecture](DESIGN_NOTES.md), [planned fixes](ROADMAP.md) and [troubleshooting](TROUBLESHOOT.md). The [host compatibility checklist](MUSL_HOST_COMPATIBILITY.md) covers broader application concerns that this executable does not test.
