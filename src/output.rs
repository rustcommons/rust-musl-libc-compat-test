use std::fmt;
use std::io::{self, Write};

/// Output failures must not interrupt probes or their cleanup.
pub struct Output<W> {
    writer: W,
    stopped: bool,
    error: Option<io::Error>,
}

impl<W: Write> Output<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            stopped: false,
            error: None,
        }
    }

    fn record(&mut self, error: io::Error) {
        self.stopped = true;
        if error.kind() != io::ErrorKind::BrokenPipe && self.error.is_none() {
            self.error = Some(error);
        }
    }

    pub fn line(&mut self, args: fmt::Arguments<'_>) {
        if self.stopped {
            return;
        }
        if let Err(error) = self.writer.write_fmt(format_args!("{args}\n")) {
            self.record(error);
        }
    }

    pub fn finish(mut self, probe_code: i32, diagnostics: &mut impl Write) -> i32 {
        // Flush explicitly before process::exit, even after a broken pipe.
        if let Err(error) = self.writer.flush() {
            self.record(error);
        }
        if let Some(error) = self.error {
            // stderr may itself be closed/full: never panic while reporting stdout.
            let _ = writeln!(diagnostics, "stdout: {error}");
            let _ = diagnostics.flush();
            1
        } else {
            probe_code
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Faults {
        pub write: Option<io::ErrorKind>,
        pub flush: Option<io::ErrorKind>,
    }

    impl Write for Faults {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.write.map_or(Ok(bytes.len()), |kind| Err(kind.into()))
        }
        fn flush(&mut self) -> io::Result<()> {
            self.flush.map_or(Ok(()), |kind| Err(kind.into()))
        }
    }

    #[test]
    fn final_flush_errors_and_broken_pipes_preserve_status() {
        for code in [0, 1] {
            for (write, flush, expected) in [
                (None, None, code),
                (Some(io::ErrorKind::BrokenPipe), None, code),
                (None, Some(io::ErrorKind::BrokenPipe), code),
                (None, Some(io::ErrorKind::StorageFull), 1),
                (
                    Some(io::ErrorKind::BrokenPipe),
                    Some(io::ErrorKind::StorageFull),
                    1,
                ),
                (
                    Some(io::ErrorKind::StorageFull),
                    Some(io::ErrorKind::BrokenPipe),
                    1,
                ),
            ] {
                let mut out = Output::new(Faults { write, flush });
                out.line(format_args!("result"));
                let mut diagnostics = Vec::new();
                assert_eq!(out.finish(code, &mut diagnostics), expected);
                assert_eq!(
                    !diagnostics.is_empty(),
                    write == Some(io::ErrorKind::StorageFull)
                        || flush == Some(io::ErrorKind::StorageFull)
                );
            }
        }
    }

    #[test]
    fn failed_diagnostic_output_does_not_panic() {
        let mut out = Output::new(Faults {
            write: Some(io::ErrorKind::StorageFull),
            flush: None,
        });
        out.line(format_args!("result"));
        let mut diagnostics = Faults {
            write: Some(io::ErrorKind::BrokenPipe),
            flush: Some(io::ErrorKind::BrokenPipe),
        };
        assert_eq!(out.finish(0, &mut diagnostics), 1);
    }
}
