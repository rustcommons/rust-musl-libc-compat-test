# Troubleshoot

Use this flow in order. Start at the top and stop once you have narrowed the problem to one category.

## 1. The binary will not start at all

Check what kind of binary you actually copied:

```bash
file ./musl-compat-test
ldd ./musl-compat-test
readelf -d ./musl-compat-test
```

Expected:

- `file` reports a statically linked ELF of the expected architecture (it does not establish the libc version)
- `ldd` reports `not a dynamic executable`
- `readelf -d` shows no `NEEDED` entries

Inspect command errors as well as output. The expected `ldd` diagnostic can return nonzero; missing tools are not successful verification.

If execution is denied from `/tmp`, choose a location allowed by host policy. For example, if your home directory permits execution:

```bash
cp ./musl-compat-test ~/musl-compat-test
~/musl-compat-test
```

What to suspect here:

- SELinux denying execution from a `tmp_t` label
- `noexec` mount options on `/tmp`
- wrong architecture or CPU feature level for the host
- a binary that is not actually fully static

Helpful checks:

```bash
uname -r
mount | grep ' /tmp '
getenforce 2>/dev/null || true
aa-status 2>/dev/null || true
```

## 2. The binary starts, but crashes immediately

An optional diagnostic is to check whether a trivial static musl binary works on the same host. The recipe below requires a C/musl toolchain; deployment hosts need not have one. If necessary, build on a suitable machine and transfer the resulting binary using the [user guide](USER_GUIDE.md). Use a scratch directory on the build machine and an execution location permitted by the test host. The following assumes the current scratch directory permits execution:

```bash
cat > ./hello.c << 'EOF'
#include <stdio.h>
int main() { puts("hello"); return 0; }
EOF
musl-gcc -static ./hello.c -o ./hello-musl
file ./hello-musl
./hello-musl
```

If the trivial musl binary also fails, investigate shared environment and build assumptions; its toolchain may differ from the Rust binary.

Check whether a core file was written:

```bash
ls -la core* /tmp/core* 2>/dev/null
```

If `systemd-coredump` is active:

```bash
coredumpctl list 2>/dev/null
```

Get the most recent kernel message:

```bash
dmesg | tail -20
```

If you need a local core dump:

```bash
ulimit -c unlimited
./musl-compat-test
ls -la core*
```

What to suspect here:

- host/kernel incompatibility severe enough to prevent startup
- wrong CPU target
- host policy or mount restrictions causing abnormal termination
- a bug in the binary itself rather than a gated test failure

## 3. The binary starts and prints output, but shows a warning

Read the warning literally first.

Current expected warning class:

- execution from `/tmp` on SELinux hosts

If you see the exec-from-`/tmp` warning, use an approved executable location. For example, only if your home directory permits execution:

```bash
cp ./musl-compat-test ~/musl-compat-test
~/musl-compat-test
```

Interpretation:

- a warning is not a failed test
- it is telling you the launch location may be the real problem if execution or specific behavior is blocked

## 4. The binary runs, but one or more tests fail

Start with the summary line:

- `passed`
- `skipped`
- `failed`

Then classify the outcome:

- `SKIP` on an older kernel can be expected
- `SKIP (security)` under confinement can be expected
- `FAIL` is the thing to investigate

Use [`TEST_MATRIX.md`](TEST_MATRIX.md) to determine whether the result is expected for that kernel and security context.

## 5. A kernel-gated syscall test fails on an old host

This should usually be a `SKIP`, not a `FAIL`.

Check the reported kernel:

```bash
uname -r
```

Expected gates in the current suite:

- `renameat2(2)` requires kernel `3.15+`
- `getrandom(2)` requires kernel `3.17+`
- `memfd_create(2)` requires kernel `3.17+`

What to suspect if you got `FAIL` instead of `SKIP`:

- incorrect kernel parsing
- the test was called without the intended gate
- the host kernel reports something unexpected

## 6. A syscall test fails on a modern host

If a modern kernel reports a failure for `getrandom`, `memfd_create`, or `renameat2`, check whether security policy is the real cause:

```bash
getenforce 2>/dev/null || true
cat /proc/self/attr/current 2>/dev/null || true
aa-status 2>/dev/null || true
```

What to suspect here:

- SELinux confinement
- AppArmor confinement
- seccomp or container restrictions
- a real bug in the test implementation

Syscall failures now include operation context, OS error text and numeric errno captured before cleanup. Use that errno to distinguish policy denial from unavailable functionality; the kernel version alone does not establish the cause.

Current security-sensitive tests in this suite:

- `syscall: memfd_create(2)`
- `syscall: renameat2(2)`

## 7. Filesystem tests fail

Check the actual temporary directory (`TMPDIR` when set, otherwise normally `/tmp`). The following `/tmp` checks apply only when it is the selected directory; substitute your actual path otherwise.

```bash
echo "$TMPDIR"
mount
df -h /tmp
touch /tmp/musl-test-write && rm -f /tmp/musl-test-write
```

What to suspect:

- unwritable temp directory
- unusual mount flags
- leftover directories after aborts/signals or cleanup failures (new runs use unique private directories)
- policy restrictions on rename or symlink operations

Affected test classes:

- `fs: create / write / read`
- `fs: symlink creation`
- `fs: directory traversal`
- `syscall: renameat2(2)`

## 8. Network tests fail

The suite only uses loopback TCP right now, so start by validating loopback on the host.

What to suspect:

- loopback is down or restricted
- container/network namespace policy interferes with localhost communication
- unusual firewall or sandbox policy

Relevant current test:

- `net: TCP loopback`

## 9. Process or `/proc` tests fail

If `/proc/self/exe` checks fail, verify that `/proc` is mounted and readable:

```bash
mount | grep proc
ls -l /proc/self/exe
```

What to suspect:

- broken or missing `/proc`
- heavy container isolation
- path-resolution oddities on the target host

Relevant current test:

- `proc: PID / /proc/self/exe`

## 10. Threading or timing tests fail

These should normally be stable on supported Linux systems.

What to suspect:

- a genuine runtime bug
- very constrained or unusual container behavior
- unexpected libc or kernel interaction

Relevant current tests:

- `thread: spawn + join`
- `thread: TLS isolation`
- `thread: barrier sync`
- `time: monotonic clock`
- `time: CLOCK_BOOTTIME`

## 11. DNS, TLS, or host-config problems appear outside the current suite

This project does not yet test every real application dependency.

If your actual app fails but this suite passes, check host dependencies such as:

- `/etc/resolv.conf`
- `/etc/hosts`
- CA bundle location
- timezone data
- locale expectations
- plugin or `dlopen` assumptions

That class of problem is consistent with a musl binary being broadly runnable while your specific application still depends on host configuration the suite does not cover.
