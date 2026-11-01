# Roadmap

This file tracks planned documentation work, code fixes, and runtime test additions. Documentation tasks below describe current behavior; they do not authorize or imply implementation of planned features.

## Documentation tasks (prioritized)

The documentation checklist records delivered work; unchecked items remain future work. P1 addresses setup and command correctness; P2 addresses organization, accuracy and contributor guidance. Existing code fixes and feature proposals remain in their own sections below.

### P1 — Setup and command correctness

- [x] Expand `README.md` “Build & Verify” with source installation prerequisites: Linux build environment, Rust/Cargo supporting edition 2024, the `x86_64-unknown-linux-musl` Rust target, and `musl-gcc`. Include clone, `cd`, `rustup target add x86_64-unknown-linux-musl`, `cargo build --release --locked`, and the resulting binary path. Explain `.cargo/config.toml`, Cargo-managed `libc`, and the distinction between build tools and runtime-host requirements. Do not invent a crates.io install or release-download workflow.
- [x] Keep static verification in the README and explain the required `file`, `ldd`, and `readelf` tools: expect static linkage and no `NEEDED` entries. Explain that the expected `ldd` diagnostic and a no-match `grep NEEDED` need not return zero; avoid hiding a missing tool or failed inspection as a successful check.
- [x] Fix README transfer instructions to include remote execution and an explicit destination: `scp target/x86_64-unknown-linux-musl/release/musl-compat-test user@host:` followed by `ssh user@host './musl-compat-test'`. Explain that the destination must allow execution; neither `/tmp` nor a home directory is guaranteed executable by policy.
- [x] Replace the absolute `/home/joey/GIT/.../TEST_MATRIX.md` link in `TROUBLESHOOT.md` section 4 with `[TEST_MATRIX.md](TEST_MATRIX.md)`.
- [x] In `TROUBLESHOOT.md` section 2, label the C smoke-test recipe as requiring a C/musl toolchain, which deployment hosts need not have. Build it on a suitable build host if necessary and run it from a location permitting execution rather than assuming `/tmp` does.

### P2 — Reader paths and implemented workflows

- [x] Refocus `README.md` on purpose, limited scope, prerequisites, quick start, static checks, no-root runtime requirements and relative documentation links. State that the package `rust-musl-libc-compat-test` builds the executable `musl-compat-test`, a focused runtime probe rather than a library or application-certification suite.
- [x] Add `USER_GUIDE.md` for detailed source build/static verification, local execution, remote transfer/execution, `--version`/`-V`, output interpretation, kernel/security skips and exit status (normal completion: 0 without failures, 1 with failures). Move lengthy README output/security examples there. Document that unknown arguments currently run the suite; `--help`, filtering, JSON and self-check are not implemented. This is a documentation task, not a CLI behavior change.
- [x] Link the guide, `TEST_MATRIX.md`, `TROUBLESHOOT.md`, `DESIGN_NOTES.md`, `MUSL_HOST_COMPATIBILITY.md` and `ROADMAP.md` from the README using relative links. Preserve the separate matrix, troubleshooting flow and scope exclusions. Label host-compatibility guidance as background for application authors: DNS/TLS and other application checks there are not implemented probe workflows. Keep planned features in the roadmap.
- [x] Move README “Architecture” into `DESIGN_NOTES.md`; explain registry wiring and the exact-name security classification in `runner.rs`, so adding or renaming a test includes reviewing both. Add contributor guidance distinguishing `cargo test --locked` (parser/ordering and resource/diagnostic regression tests) from executing the runtime suite; keep authoritative validation instructions in `AGENTS.md`.

### P2 — Accuracy and current limitations

- [x] Correct the README provenance explanation: the musl field records a build-time sysroot where `libc.a` exists, not an extracted musl version or independent proof of which archive was linked. Keep `--version` provenance documentation aligned with `build.rs`.
- [x] Replace blanket security claims in README/guide with actual SELinux/AppArmor detection and proactive skip behavior. Do not imply an unfamiliar AppArmor executable is necessarily unrestricted, that no detected framework means no restrictions, or that detection covers seccomp. Avoid presenting relabeling or a destination directory as a universal policy fix.
- [x] Label sample output illustrative. Do not infer a distribution, guaranteed kernel version or security configuration from the sample kernel string; remove blanket WSL security claims. Preserve useful example pass/skip profiles without implying universal compatibility.
- [x] In the guide and timing coverage documentation, document the actual clock source and link the correctness work below; the original `SystemTime` limitation has now been fixed with `CLOCK_MONOTONIC`.
- [x] Document temporary storage under `std::env::temp_dir()` (not always `/tmp`) and the absence of built-in timeouts. The earlier fixed-name limitation has now been resolved by private per-invocation directories; document remaining cleanup limits for aborts/signals. Timeout work remains planned.

## Recommended Additions

### `proc: fork/exec self`

Why add it:
The current suite checks `/proc/self/exe` readability, but it does not prove that `execve` works correctly for this static musl binary. Spawning the current binary with `--version` or a small `--self-check` mode would exercise process startup more directly.

Linux functionality actually being tested:
- `fork`/`clone` + `execve` process creation path
- ELF binary startup for the produced static musl executable
- argv handling across exec
- environment inheritance across exec
- stdout/stderr pipe handling through `std::process::Command`
- `/proc/self/exe` usability as an execution target, if that route is used

Why it matters for this project:
This is closer to the real success condition than a pure metadata check. A compatibility binary that starts once but cannot be re-executed cleanly on the target host is still suspect.

### `io: pipe read/write`

Why add it:
The suite currently exercises filesystem I/O and TCP sockets, but not anonymous file descriptors. A simple pipe round-trip is cheap and hits a different class of libc and kernel interactions.

Linux functionality actually being tested:
- `pipe` or `pipe2`
- blocking file descriptor reads and writes
- anonymous kernel-managed byte streams
- close semantics and EOF behavior
- libc wrappers around unbuffered descriptor I/O

Why it matters for this project:
It gives coverage to core Unix I/O primitives that are independent of both the filesystem and the network stack.

### `net: UDP loopback`

Why add it:
The current network coverage only exercises TCP loopback. UDP uses a different socket behavior model and is worth validating separately.

Linux functionality actually being tested:
- `socket` creation for datagram sockets
- `bind`, `sendto`, `recvfrom` or connected UDP send/recv
- loopback IPv4 routing
- datagram delivery semantics without stream state

Why it matters for this project:
It broadens network coverage without introducing outside dependencies or requiring privileged ports.

### `fs: rename + unlink while open`

Why add it:
The suite covers basic create/read/write, symlinks, and directory traversal, but not common Unix pathname semantics around open handles, renames, and unlinks.

Linux functionality actually being tested:
- `open`
- `rename`
- `unlink`
- continued file access through an already-open descriptor after pathname changes
- VFS behavior for inode-backed files independent of directory entry lifetime

Why it matters for this project:
This is a stronger Unix compatibility signal than another basic file read. It validates behavior many real tools rely on for temp files, atomic updates, and log rotation patterns.

### `net: DNS/localhost resolution`

Why add it:
Name resolution is one of the more common musl-versus-host-environment friction points. A `ToSocketAddrs("localhost:0")` or `getaddrinfo("localhost")` test would catch bad `/etc/hosts` or resolver assumptions.

Linux functionality actually being tested:
- libc name resolution path via `getaddrinfo`
- `/etc/hosts` handling for `localhost`
- resolver configuration parsing where relevant
- address family and socket address construction

Why it matters for this project:
This checks a real-world integration edge that often fails differently from raw socket creation.

Operational note:
This one is more environment-sensitive than the other recommendations, so it should likely be optional or skippable rather than treated as a hard failure everywhere.

### `thread: condvar or channel wakeup`

Why add it:
The suite currently covers thread spawn/join, TLS isolation, and barriers, but not blocking wake/sleep coordination. A `Condvar` test would exercise futex-backed synchronization more directly.

Linux functionality actually being tested:
- pthread-backed thread synchronization
- futex-based wait/wake behavior underneath Rust synchronization primitives
- scheduler wakeups and handoff between blocked threads
- mutex + condition variable coordination correctness

Why it matters for this project:
It expands concurrency coverage from “threads exist” to “threads can block and be woken correctly,” which is a more meaningful runtime check.

### `fs: unix domain socket pair`

Why add it:
The suite currently covers TCP loopback and regular filesystem operations, but not local IPC through Unix sockets. This is a common Linux primitive used by many real-world tools and exercises a different path than TCP.

Linux functionality actually being tested:
- `socketpair` or `AF_UNIX` socket creation
- local bidirectional stream semantics without the IP stack
- file descriptor passing readiness, if extended later
- libc wrappers around Unix-domain socket I/O

Why it matters for this project:
It broadens local IPC coverage and helps distinguish “network stack works” from “general socket and descriptor behavior works.”

### `proc: signal handling round-trip`

Why add it:
The current suite does not exercise signal delivery at all. A small self-signal test using `SIGUSR1` would validate a core Unix runtime behavior without requiring privileges.

Linux functionality actually being tested:
- `sigaction`
- signal mask and handler installation
- signal delivery to the current process or thread
- libc signal trampoline behavior

Why it matters for this project:
Signals are foundational Unix behavior and are often involved in supervisor shutdown, timeouts, and diagnostics. A static binary that mishandles them is not fully trustworthy.

### `fs: metadata and permissions sanity`

Why add it:
The suite verifies file contents, but not ownership and mode bit handling. A basic chmod/stat round-trip would validate another layer of libc and VFS behavior.

Linux functionality actually being tested:
- `stat`/`fstat`
- `chmod`
- Unix permission bits and metadata reporting
- libc path-to-metadata translation

Why it matters for this project:
This is a cheap way to validate file metadata correctness, which matters for packaging, temp files, and execution behavior on target hosts.

## Fixes To Make First

- [x] **Real monotonic timing:** `process::monotonic_clock` now reads `CLOCK_MONOTONIC` before and after its sleep, checks valid timespec values and rejects backwards/too-short elapsed time. The test name is unchanged. The original use of `SystemTime` was confirmed in source without changing the host clock.
- [x] **Isolated temporary resources and cleanup:** filesystem and rename checks now use private directories atomically created by `mkdtemp` under the configured temporary base. Each invocation owns its directory; success and error paths clean it up, and a drop guard provides best-effort cleanup on unwind. Cleanup failures retain the original test error. Aborts/signals can still leave directories. Baseline reproduction found a stale symlink failure and one failure in 12 concurrent runs; after the fix, all 12 concurrent runs passed with no new leftovers and with seeded stale files preserved.
- [x] **Immediate syscall diagnostics:** failures of `getrandom`, `memfd_create`, `renameat2` and clock reads capture `last_os_error` before formatting or cleanup. Messages include operation context, OS error text and numeric errno. A regression test verifies that a later syscall changing errno and directory cleanup do not replace the original diagnostic.

Historical validation (2026-10-31, revision `5ef07a1ac968bd7b25728dd3862eec2b559e11cc`): [x86-64 Linux static musl CI run](https://github.com/rustcommons/rust-musl-libc-compat-test/actions/runs/41238463704) passed six unit tests, formatting of changed modules, Clippy (two existing style warnings in untouched modules), static ELF inspection, all 15 runtime checks and 12 concurrent runs. The static executable had no dynamic section/`NEEDED` entries. Native ARM64 GNU tests and runs also passed as supplementary evidence, not as a substitute for the musl gate. This does not certify older kernels or different host security policies.

For the later F6 validation and the current evidence index, see [requirements and evidence](docs/REQUIREMENTS.md#scope-and-sources). The preceding outcomes describe that revision, not a new run or a universal compatibility result.

## Independent review follow-up

- [x] **F6: fallible stdout:** replace panic-on-write output in suite/version modes; ignore only `BrokenPipe`, continue checks/cleanup, retain failed-probe status, and surface other write/final-flush errors through best-effort stderr plus exit 1. Regression coverage includes injected flush errors, closed/early-closing pipes, `/dev/full`, and real probe failures.
- [ ] **Compiler provenance (separate from F6):** `build.rs` queries `rustc` through `PATH` for version and sysroot instead of Cargo's `RUSTC`. Use Cargo's selected compiler for these queries, handle unsuccessful commands, and test a PATH/RUSTC mismatch plus wrapper behavior. Determine explicitly whether wrappers should participate in metadata queries rather than treating a wrapper executable as the compiler or naively concatenating `RUSTC_WRAPPER`/`RUSTC_WORKSPACE_WRAPPER`. Keep compiler identity separate from wrapper metadata. This warrants a separate tested change; it must not claim to identify the actual musl version or linked archive.

## Broader Improvements

### Add machine-readable output mode

What to add:
- A `--json` flag or similar output mode
- Structured per-test fields for `name`, `status`, `reason`, `kernel_gate`, and `error`
- Structured build and host metadata in the header section

Why it matters:
Right now the output is human-readable only. JSON would make it much easier to archive results from multiple hosts and compare behavior across kernels or distributions.

### Add per-test timeouts or hang protection

What to add:
- A small timeout mechanism around tests that could block on I/O or synchronization
- A clear `TIMEOUT` or `FAIL` mode distinct from ordinary assertion failures

Why it matters:
Compatibility probes should fail fast. If a target host wedges on one primitive, the suite should still produce actionable output instead of hanging indefinitely.

Linux functionality actually being protected:
- Blocking socket operations
- synchronization waits
- child process execution paths

### Add a focused self-check CLI mode

What to add:
- A hidden or documented `--self-check` mode intended only for the suite's own `fork/exec self` test
- Minimal output and deterministic exit codes

Why it matters:
This makes the process execution test cleaner than reusing the human-facing main output path, and it provides a stable target for future smoke tests.

### Expand unit tests for non-runtime parsing and detection logic

What to add:
- More unit tests for kernel version parsing in `kernel.rs`
- Unit tests for security-context parsing in `security.rs`
- Unit tests for test registry invariants, such as all names being unique and kernel-gated tests having reasons

Why it matters:
Not every regression requires a target Linux host to catch. The parsing and wiring code should be protected by fast local tests.

### Extend CI checks for static-linkage invariants

The focused static-musl workflow now builds and verifies `file`, `ldd` and `readelf`, runs unit tests and the probe, and checks concurrent resource isolation. It prints version metadata. Broader metadata assertions remain future work.

Remaining work:
- A check that the binary still reports the expected build metadata fields

Why it matters:
This project’s central claim is “static musl binary with predictable behavior.” Static-linkage regressions should be caught automatically before anyone copies an artifact to a test host.

### Complete documentation around expected skip behavior

`TEST_MATRIX.md` already contains pass/skip profiles and distinguishes skips from failures. Preserve that delivered work rather than planning another matrix.

- [x] **P2, documentation:** Qualify the “Kernel < 3.15” column and its baseline assumptions so it does not claim support for arbitrarily old kernels; the ungated `CLOCK_BOOTTIME` check has a prerequisite noted in `src/tests/process.rs`. Keep expected outcomes conditional on host facilities and security context, and reference this matrix from the user guide instead of duplicating it.
