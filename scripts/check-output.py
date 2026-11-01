"""Release-binary stdout regressions; accepts a native Linux executable path."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())


def check(result, expected, diagnostic=False):
    assert result.returncode == expected, (result.returncode, result.stderr)
    assert b"panicked" not in result.stderr, result.stderr
    assert (b"stdout:" in result.stderr) == diagnostic, result.stderr


with tempfile.TemporaryDirectory() as root:
    env = dict(os.environ, TMPDIR=root)
    for args in ([], ["--version"], ["-V"]):
        # Deterministic BrokenPipe: the reader is gone before the process starts.
        reader, writer = os.pipe()
        os.close(reader)
        try:
            check(subprocess.run([binary, *args], stdout=writer, stderr=subprocess.PIPE,
                                 env=env, timeout=30), 0)
        finally:
            os.close(writer)

        # Smoke test: a consumer reads a line then exits early. Short version
        # output may already be fully written; the pre-closed case above
        # guarantees BrokenPipe regardless of scheduling.
        process = subprocess.Popen([binary, *args], stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, env=env)
        assert process.stdout.readline()
        process.stdout.close()
        process.wait(timeout=30)
        check(subprocess.CompletedProcess([], process.returncode, stderr=process.stderr.read()), 0)
        process.stderr.close()

        with open('/dev/full', 'wb') as full:
            check(subprocess.run([binary, *args], stdout=full, stderr=subprocess.PIPE,
                                 env=env, timeout=30), 1, diagnostic=True)
        assert not list(Path(root).iterdir()), "probe resources leaked after output failure"

    # Force actual filesystem probes to fail, independently of stdout closure.
    env['TMPDIR'] = str(Path(root) / 'missing')
    reader, writer = os.pipe()
    os.close(reader)
    try:
        check(subprocess.run([binary], stdout=writer, stderr=subprocess.PIPE,
                             env=env, timeout=30), 1)
    finally:
        os.close(writer)

print('stdout regressions passed: version/suite, early closure, full device, cleanup, failed probes')
