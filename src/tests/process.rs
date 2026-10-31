use super::support::syscall_error;
use std::fs;
use std::time::Duration;

pub fn monotonic_clock() -> Result<(), String> {
    let before = monotonic_now()?;
    std::thread::sleep(Duration::from_millis(20));
    let elapsed = monotonic_now()?
        .checked_sub(before)
        .ok_or("CLOCK_MONOTONIC moved backwards")?;
    if elapsed < Duration::from_millis(20) {
        return Err(format!("elapsed {:?} is less than 20ms sleep", elapsed));
    }
    Ok(())
}

fn monotonic_now() -> Result<Duration, String> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) } != 0 {
        return Err(syscall_error("clock_gettime(CLOCK_MONOTONIC)"));
    }
    if ts.tv_sec < 0 || !(0..1_000_000_000).contains(&ts.tv_nsec) {
        return Err("CLOCK_MONOTONIC returned an invalid timespec".into());
    }
    Ok(Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32))
}

pub fn clock_boottime() -> Result<(), String> {
    // CLOCK_BOOTTIME available since 2.6.39 — safe on 3.12
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ret = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
    if ret != 0 {
        return Err(syscall_error("clock_gettime(CLOCK_BOOTTIME)"));
    }
    if ts.tv_sec < 0 {
        return Err(format!("bogus uptime: {} seconds", ts.tv_sec));
    }
    Ok(())
}

pub fn proc_self() -> Result<(), String> {
    let pid = std::process::id();
    if pid == 0 {
        return Err("getpid() returned 0".into());
    }

    let exe =
        fs::read_link("/proc/self/exe").map_err(|e| format!("/proc/self/exe unreadable: {}", e))?;

    if !exe.exists() {
        return Err(format!("/proc/self/exe -> {:?} does not exist", exe));
    }

    Ok(())
}
