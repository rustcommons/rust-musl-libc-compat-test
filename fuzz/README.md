# Kernel parser fuzzing

This development-only cargo-fuzz target compiles the actual [kernel module](../src/kernel.rs) directly. It exercises `KernelVersion::parse`, `Display` and `at_least`, without calling `uname`, security detection or runtime probes. No public library target or product behavior changes are needed.

The byte-input harness applies the same lossy UTF-8 conversion as the running-kernel path. It checks the documented first-three-ASCII-numeric-token behavior against an independent checked-arithmetic byte scanner, including overflow-to-zero, missing components and ignored suffixes. Display/parse round trips and the 3.17 gate/order relationship are also checked. Seeds include real RHEL/SUSE release strings, a gate boundary, overflow, empty input and malformed UTF-8/NUL bytes.

## Run locally

Use Linux with a GNU Rust target and a C++ compiler (`g++` on the hosted CI image). The following commands run from the repository root on x86-64 Linux. The explicit GNU target overrides the product's default static-musl target. On an AArch64 Linux host substitute `aarch64-unknown-linux-gnu` in both target arguments.

```sh
rustup toolchain install nightly-2026-11-15 --profile minimal
cargo +nightly-2026-11-15 install cargo-fuzz --version 0.13.2 --locked --target x86_64-unknown-linux-gnu --jobs 2
cargo +nightly-2026-11-15 fetch --manifest-path fuzz/Cargo.toml --locked --target x86_64-unknown-linux-gnu
```

Create a private writable corpus so mutations do not change the committed seeds:

```sh
fuzz_scratch=$(mktemp -d)
trap 'rm -rf "$fuzz_scratch"' EXIT
mkdir "$fuzz_scratch/corpus"
cp -R fuzz/seeds/kernel_version/. "$fuzz_scratch/corpus/"
cp fuzz/Cargo.lock "$fuzz_scratch/Cargo.lock"
CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true cargo +nightly-2026-11-15 fuzz run kernel_version "$fuzz_scratch/corpus" --target x86_64-unknown-linux-gnu --sanitizer address -- -max_total_time=30 -max_len=4096 -timeout=5 -rss_limit_mb=1024
cmp fuzz/Cargo.lock "$fuzz_scratch/Cargo.lock"
```

Cargo-fuzz 0.13.2 does not accept `--locked`. The prefetch validates the committed graph with Cargo's `--locked`, fuzzing runs offline, and the final comparison rejects lockfile drift. Both the dated nightly and cargo-fuzz version are explicit. The separate unpublished package and checksum-bearing [Cargo.lock](Cargo.lock) do not alter the product manifest or lockfile. Root [cargo-deny policy](../deny.toml) also audits this graph using the GNU target; the NCSA license exception is scoped to libfuzzer-sys 0.4.13, whose libFuzzer code includes LLVM licensing. It does not authorize product redistribution.

## CI and failures

[Parser fuzzing](../.github/workflows/fuzz.yml) runs a 30-second smoke budget on non-Dependabot PRs and a 300-second budget on main pushes, weekly and manual runs. There is one fuzz worker and at most two build workers. Each input is limited to 4096 bytes, five seconds and a 1024 MiB process RSS limit; the whole job has a 15-minute timeout. Actual runtime may slightly exceed the time budget. Per-PR concurrency cancels superseded runs. Dependabot PRs retain the existing core checks, including the fuzz graph dependency audit, and defer fuzz execution to main/weekly runs.

Failures save ignored reproductions under `fuzz/artifacts/kernel_version/`; CI uploads available reproductions for seven days. After downloading one, replay it with the pinned tools:

```sh
CARGO_NET_OFFLINE=true cargo +nightly-2026-11-15 fuzz run kernel_version /path/to/reproduction --target x86_64-unknown-linux-gnu --sanitizer address
```

Treat a crash, assertion, sanitizer finding or timeout as a finding to reproduce and investigate. A bounded pass is not exhaustive input coverage, old-host qualification or static-musl runtime evidence. Do not fuzz the syscall probes or suppress failures merely to improve a score. Scorecard can recognize the cargo-fuzz integration after a main scan; no score increase is claimed before observing that report. Keep CodeQL and normal CI enabled on commits and merges; historical SAST coverage may take time to improve as the sampled commit window advances.
