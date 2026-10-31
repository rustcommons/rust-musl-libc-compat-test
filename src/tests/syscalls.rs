/// Tests in this module are all kernel-gated in the test registry.
/// They should never be called directly on an old kernel.
use super::support::{syscall_error, with_temp_dir};
use std::os::unix::ffi::OsStrExt;

pub fn getrandom() -> Result<(), String> {
    let mut buf = [0u8; 32];
    let ret = unsafe { libc::syscall(libc::SYS_getrandom, buf.as_mut_ptr(), buf.len(), 0u32) };
    if ret < 0 {
        return Err(syscall_error("getrandom"));
    }
    // Extremely unlikely to be all zeros if working correctly
    if buf == [0u8; 32] {
        return Err("getrandom filled buffer with all zeros".into());
    }
    Ok(())
}

pub fn memfd_create() -> Result<(), String> {
    let name = std::ffi::CString::new("musl_compat_test").unwrap();
    let fd = unsafe { libc::syscall(libc::SYS_memfd_create, name.as_ptr(), 0u32) };
    if fd < 0 {
        return Err(syscall_error("memfd_create"));
    }
    unsafe { libc::close(fd as i32) };
    Ok(())
}

pub fn renameat2() -> Result<(), String> {
    use std::ffi::CString;
    use std::fs;

    with_temp_dir(|dir| {
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        fs::write(&a, b"a").map_err(|e| e.to_string())?;
        fs::write(&b, b"b").map_err(|e| e.to_string())?;

        let a_cstr = CString::new(a.as_os_str().as_bytes()).unwrap();
        let b_cstr = CString::new(b.as_os_str().as_bytes()).unwrap();

        // RENAME_EXCHANGE = 2 — atomically swap the two names
        let ret = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                libc::AT_FDCWD,
                a_cstr.as_ptr(),
                libc::AT_FDCWD,
                b_cstr.as_ptr(),
                2u32, // RENAME_EXCHANGE
            )
        };

        if ret != 0 {
            return Err(syscall_error("renameat2(RENAME_EXCHANGE)"));
        }

        let a_contents = fs::read_to_string(&a).map_err(|e| e.to_string())?;
        let b_contents = fs::read_to_string(&b).map_err(|e| e.to_string())?;

        if a_contents != "b" || b_contents != "a" {
            return Err(format!(
                "RENAME_EXCHANGE failed: a='{}' b='{}'",
                a_contents, b_contents
            ));
        }
        Ok(())
    })
}
