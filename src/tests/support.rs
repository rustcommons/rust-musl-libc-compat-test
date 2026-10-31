use std::ffi::{CStr, CString, OsStr};
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

// Capture errno before formatting, cleanup, or any other operation can replace it.
pub fn syscall_error(operation: &str) -> String {
    let error = std::io::Error::last_os_error();
    format!("{operation}: {error}")
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<Self, String> {
        let base = std::env::temp_dir();
        let base = fs::canonicalize(&base).map_err(|e| format!("temporary directory: {e}"))?;
        let template = base.join("musl-compat-XXXXXX");
        let mut bytes = CString::new(template.as_os_str().as_bytes())
            .map_err(|e| format!("temporary path: {e}"))?
            .into_bytes_with_nul();
        // mkdtemp atomically creates a private directory and replaces XXXXXX.
        let result = unsafe { libc::mkdtemp(bytes.as_mut_ptr().cast()) };
        if result.is_null() {
            return Err(syscall_error("mkdtemp"));
        }
        let path = CStr::from_bytes_with_nul(&bytes).expect("mkdtemp preserves terminator");
        Ok(Self(PathBuf::from(OsStr::from_bytes(path.to_bytes()))))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Best effort on early return or unwind; aborts/signals cannot run Drop.
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn with_temp_dir(test: impl FnOnce(&Path) -> Result<(), String>) -> Result<(), String> {
    let dir = TempDir::new()?;
    let result = test(&dir.0);
    // Preserve the original test error, and report cleanup failures as well.
    match (result, fs::remove_dir_all(&dir.0)) {
        (result, Ok(())) => result,
        (Ok(()), Err(error)) => Err(format!("temporary directory cleanup: {error}")),
        (Err(test), Err(error)) => Err(format!("{test}; temporary directory cleanup: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    #[test]
    fn concurrent_directories_are_isolated_and_removed() {
        let barrier = Arc::new(Barrier::new(4));
        let threads: Vec<_> = (0..4)
            .map(|i| {
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    let mut path = PathBuf::new();
                    with_temp_dir(|dir| {
                        path = dir.to_owned();
                        fs::write(dir.join("same-name"), i.to_string()).unwrap();
                        barrier.wait();
                        assert_eq!(
                            fs::read_to_string(dir.join("same-name")).unwrap(),
                            i.to_string()
                        );
                        Ok(())
                    })
                    .unwrap();
                    assert!(!path.exists());
                    path
                })
            })
            .collect();
        let paths: std::collections::HashSet<_> =
            threads.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(paths.len(), 4);
    }

    #[test]
    fn early_error_removes_nested_resources_and_preserves_diagnostic() {
        let mut path = PathBuf::new();
        let error = with_temp_dir(|dir| {
            path = dir.to_owned();
            fs::create_dir(dir.join("nested")).unwrap();
            fs::write(dir.join("nested/file"), b"data").unwrap();
            Err("original failure".into())
        })
        .unwrap_err();
        assert_eq!(error, "original failure");
        assert!(!path.exists());
    }

    #[test]
    fn errno_diagnostic_survives_cleanup() {
        let error = with_temp_dir(|_| {
            assert_eq!(unsafe { libc::close(-1) }, -1);
            let error = syscall_error("close(-1)");
            // Simulate another syscall replacing errno before resource cleanup.
            let mut ts = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            assert_eq!(unsafe { libc::clock_gettime(-1, &mut ts) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EINVAL)
            );
            Err(error)
        })
        .unwrap_err();
        let expected = std::io::Error::from_raw_os_error(libc::EBADF);
        assert_eq!(error, format!("close(-1): {expected}"));
        assert!(error.contains(&format!("os error {}", libc::EBADF)));
    }
}
