# Rust Musl Libc Compat Test

A focused Rust runtime probe for static musl compatibility across Linux kernels and host security contexts. The package `rust-musl-libc-compat-test` builds the `musl-compat-test` executable, not a library API.

A successful run validates only the operations exercised by this probe. It does not certify arbitrary musl applications, complete libc coverage, or DNS/TLS behavior.

## Build and run from source

Use a Linux build environment for the configured `x86_64-unknown-linux-musl` target. Install Git, Rust/Cargo supporting edition 2024, rustup, and a musl C toolchain providing `musl-gcc` on `PATH`. Use your distribution's package manager for the native toolchain; package names vary. Static verification also needs `file`, `ldd`, and `readelf` (usually supplied by file, libc tools, and binutils packages).

```sh
git clone https://github.com/rustcommons/rust-musl-libc-compat-test.git
cd rust-musl-libc-compat-test
rustup target add x86_64-unknown-linux-musl
cargo build --release --locked
./target/x86_64-unknown-linux-musl/release/musl-compat-test --version
./target/x86_64-unknown-linux-musl/release/musl-compat-test
```

[.cargo/config.toml](.cargo/config.toml) selects the target, `musl-gcc` linker and static flags. Cargo obtains the `libc` crate using the committed lockfile. No manual Rust dependency installation is needed.

## Verify the artifact

Run these inspection commands separately on the binary you built:

```sh
file target/x86_64-unknown-linux-musl/release/musl-compat-test
ldd target/x86_64-unknown-linux-musl/release/musl-compat-test
readelf -d target/x86_64-unknown-linux-musl/release/musl-compat-test
```

Expect an x86-64 statically linked ELF, `ldd` reporting “not a dynamic executable,” and no dynamic `NEEDED` entries. That `ldd` diagnostic may have a nonzero exit status. Inspect `readelf` output and errors directly; filtering through `grep NEEDED` returns nonzero for an expected no-match and can hide an inspection failure. These checks establish linkage properties, not universal host compatibility.

## Runtime requirements

The destination needs a compatible x86-64 Linux environment, an executable launch location, writable temporary storage, working loopback networking and `/proc`. The probe needs no root privileges or installed Rust/musl development toolchain. Host policy can still deny operations. Run on the actual target hosts and interpret expected skips rather than requiring every check to pass on every kernel.

## Documentation

This repository is not published on crates.io. The manifest version is not a published-package or host-qualification claim.

- [User guide](USER_GUIDE.md): build, run, transfer, provenance, results and security scenarios.
- [Test matrix](TEST_MATRIX.md): expected kernel and security skips.
- [Troubleshooting](TROUBLESHOOT.md): investigate startup and runtime failures.
- [Design notes](DESIGN_NOTES.md): scope, architecture and contributor guidance.
- [Musl host compatibility](MUSL_HOST_COMPATIBILITY.md): broader guidance for application authors, beyond this probe's coverage.
- [Roadmap](ROADMAP.md): planned code fixes and features.
- [Repository instructions](AGENTS.md): authoritative working conventions and validation.
- [Requirements and documentation map](docs/REQUIREMENTS.md): confirmed constraints, bounded evidence, and remaining gaps.
