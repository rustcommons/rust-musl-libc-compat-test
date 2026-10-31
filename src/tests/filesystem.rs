use std::fs::{self, File};
use std::io::{Read, Write};

use super::support::with_temp_dir;

pub fn read_write() -> Result<(), String> {
    with_temp_dir(|dir| {
        let path = dir.join("data.bin");

        File::create(&path)
            .and_then(|mut f| f.write_all(b"hello musl"))
            .map_err(|e| e.to_string())?;

        let mut buf = String::new();
        File::open(&path)
            .and_then(|mut f| f.read_to_string(&mut buf))
            .map_err(|e| e.to_string())?;

        if buf != "hello musl" {
            return Err(format!("read back '{}', expected 'hello musl'", buf));
        }

        Ok(())
    })
}

pub fn symlink() -> Result<(), String> {
    with_temp_dir(|dir| {
        let src = dir.join("src.txt");
        let link = dir.join("link.txt");

        fs::write(&src, b"target").map_err(|e| e.to_string())?;
        std::os::unix::fs::symlink(&src, &link).map_err(|e| e.to_string())?;

        let meta = fs::symlink_metadata(&link).map_err(|e| e.to_string())?;
        if !meta.file_type().is_symlink() {
            return Err("symlink_metadata did not report symlink type".into());
        }

        // Follow the link and read through it
        let contents = fs::read_to_string(&link).map_err(|e| e.to_string())?;
        if contents != "target" {
            return Err(format!(
                "read through symlink got '{}', expected 'target'",
                contents
            ));
        }

        Ok(())
    })
}

pub fn dir_walk() -> Result<(), String> {
    with_temp_dir(|dir| {
        for i in 0..5 {
            fs::write(dir.join(format!("{}.txt", i)), b"x").map_err(|e| e.to_string())?;
        }

        let count = fs::read_dir(dir).map_err(|e| e.to_string())?.count();

        if count != 5 {
            return Err(format!("expected 5 entries, found {}", count));
        }

        Ok(())
    })
}
