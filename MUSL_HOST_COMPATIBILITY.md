# MUSL Host Compatibility

This is background for application authors, not a list of checks implemented by `musl-compat-test`. DNS, TLS, locale and other application concerns below are outside the current probe coverage; see the [user guide](USER_GUIDE.md) for actual workflows.

Use this document as an instruction sheet and pre-flight checklist when you build an application with musl and plan to run it on different Linux hosts with different kernel versions.

## What musl does and does not solve

The main thing to keep straight is that musl solves user-space libc portability, not kernel compatibility. A statically linked musl binary can avoid host glibc/version problems, but it still runs on the host kernel and host filesystem/security environment.

## Issues you may face

The issues you usually hit are:

- Newer syscalls on older kernels. If your binary or a dependency uses `getrandom`, `memfd_create`, `renameat2`, `statx`, `pidfd_*`, `copy_file_range`, `io_uring`, or similar, an older kernel may return `ENOSYS` or behave differently.
- Different syscall semantics across kernel versions. Sometimes the syscall exists but flags or edge-case behavior differ, so code that works on a modern kernel can fail or misbehave on an older one.
- DNS and name resolution surprises. musl’s resolver behavior differs from glibc/NSS expectations. If your app assumes glibc-style NSS modules, LDAP-backed users/groups, mDNS plugins, or certain resolver quirks, it may not behave the same.
- Host config dependencies. Even a static binary still depends on runtime files like `/etc/resolv.conf`, `/etc/hosts`, `/etc/passwd`, CA bundles, timezone data, locale data, and `/proc` or `/sys` if the app uses them.
- TLS/certificate paths. HTTPS clients often fail on some hosts because the CA bundle path differs or the host lacks expected cert files.
- Locale and i18n differences. musl’s locale support is simpler than glibc’s. Programs expecting rich locale behavior, collation, or glibc locale databases can see regressions.
- Threads, signals, and low-level runtime assumptions. Most code is fine, but software with tight coupling to pthread internals, signal stacks, or glibc-specific behavior can break.
- `dlopen`/plugin expectations. Fully static musl binaries do not mix well with ecosystems that expect dynamic plugins or glibc-linked shared objects.
- Security policy differences. SELinux, AppArmor, seccomp, container runtimes, and mount options can block syscalls, execution from `/tmp`, `memfd_create`, unusual renames, or other operations even when the kernel is new enough.
- Filesystem and mount differences. `noexec`, `nodev`, `nosuid`, overlayfs quirks, older procfs/sysfs behavior, and network filesystem differences can all affect runtime behavior.
- CPU/ABI assumptions. musl does not protect you from building for an instruction set not available on the target CPU.
- Tooling assumptions. Some third-party crates or native dependencies quietly assume glibc behavior and will compile but fail subtly under musl.

## Practical rule

- musl helps with libc portability across distros
- it does not guarantee portability across kernel versions or host environments

## User checklist

Use this checklist before you call a musl build portable across hosts:

- Check whether your application or any dependency relies on newer syscalls.
- Verify that your code gracefully handles `ENOSYS`, `EPERM`, and `EINVAL`.
- Test on your oldest supported kernel, not just your build host.
- Test under different security contexts and mount layouts.
- Be explicit about runtime file dependencies like certs, tzdata, and resolver config.

## Minimum host checks

Run these on the target host before you trust the result:

```bash
uname -r
cat /etc/os-release
mount
getenforce 2>/dev/null || true
aa-status 2>/dev/null || true
file ./your-binary
ldd ./your-binary
readelf -d ./your-binary
```

What these checks tell you:

- `uname -r`: the kernel version you are actually testing against
- `cat /etc/os-release`: the distro family and release
- `mount`: whether `noexec`, overlay, or other mount flags may affect behavior
- `getenforce`: whether SELinux is enforcing
- `aa-status`: whether AppArmor is active
- `file`: whether the binary is actually statically linked for the expected target
- `ldd`: whether the binary is unexpectedly dynamic
- `readelf -d`: inspect for dynamic `NEEDED` entries and check for inspection errors

## Pre-flight review

Before shipping to another host, review these areas:

- Kernel compatibility:
  Confirm the oldest kernel version you intend to support and review whether your code path depends on syscalls added after that version.
- Runtime environment:
  Confirm the target host has the runtime files your application expects, including resolver config, CA bundles, timezone data, and any `/proc` or `/sys` data you read.
- Security policy:
  Confirm whether SELinux, AppArmor, seccomp, container policy, or mount options may block execution or individual syscalls.
- CPU target:
  Confirm the binary was built for an instruction set the destination CPU actually supports.
- Third-party code:
  Confirm any native libraries, crates, or plugins do not quietly depend on glibc-specific behavior.

## What to validate on each target host

- The binary starts successfully.
- Expected syscalls are available or correctly handled when unavailable.
- Name resolution works the way the application expects.
- TLS connections can find and use the expected CA bundle.
- Filesystem behavior matches assumptions around temp files, mounts, and permissions.
- Signals, threads, and process execution paths behave correctly for your workload.

## Runtime dependency checklist

Even a static musl binary may still depend on host files and services. Check these explicitly:

- `/etc/resolv.conf`: required for DNS resolution
- `/etc/hosts`: required if you expect `localhost` or local overrides to resolve correctly
- `/etc/passwd`: required if your application resolves users or home directories
- `/etc/group`: required if your application resolves group information
- CA bundle path: required for outbound HTTPS or TLS peer verification
- timezone data: required if your application depends on local time zones rather than UTC only
- `/proc`: required if your application reads process, memory, mount, or kernel metadata
- `/sys`: required if your application reads device, kernel, or security state from sysfs

## Common failure signatures

Watch for these patterns when validating across hosts:

- `ENOSYS`: the kernel is too old for the syscall or feature being used
- `EPERM`: the syscall exists but security policy or container restrictions blocked it
- `EINVAL`: the syscall exists, but flags or semantics differ from what the application expected
- `ENOENT`: a runtime file dependency is missing, or a path assumption is wrong
- certificate verification failures: the CA bundle is missing, misplaced, or different from what the application expects
- DNS resolution failures: the resolver configuration, `/etc/hosts`, or libc behavior is not matching the application’s assumptions
- execution denied from `/tmp`: mount flags or SELinux labeling prevent running the binary from that location

## Do not assume

- Do not assume musl makes the binary compatible with every Linux kernel.
- Do not assume glibc-style NSS behavior will exist on the target host.
- Do not assume plugin loading or `dlopen`-based ecosystems will work with a fully static binary.
- Do not assume locale behavior will match glibc.
- Do not assume `/tmp` is executable.
- Do not assume the target CPU supports the same instruction set as the build machine.

## Suggested test matrix

Validate the application in more than one environment:

- build host: confirms the binary at least works where it was produced
- oldest supported kernel: confirms your real minimum compatibility target
- SELinux host: confirms policy does not break startup or required syscalls
- AppArmor host: confirms confinement or profile state does not change behavior unexpectedly
- containerized host: confirms seccomp, mount layout, and namespace constraints do not break the application

## Release checklist

Before calling the build ready for distribution:

- Confirm the binary is statically linked.
- Confirm no unexpected dynamic dependencies appear in the ELF metadata.
- Run the application on the oldest supported kernel.
- Run the application on at least one host with a security framework enabled.
- Verify expected degraded behavior on older or more restricted hosts.
- Verify runtime dependencies like cert bundles, resolver config, and timezone data.
- Verify the binary is placed in an executable location on the target host.

## Decision standard

Do not treat “it is statically linked with musl” as the success condition.

Treat this as the success condition:

- the binary starts on the target host
- required kernel features are present or cleanly degraded
- required host configuration files exist
- security policy and mount layout do not break the application
- the application behaves correctly on the oldest host you claim to support
