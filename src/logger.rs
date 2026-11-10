use log::{Level, LevelFilter, debug};
use std::time::SystemTime;

/// Logs to stdout, or to stderr if stdout is reserved for the result
pub fn init(log_level: LevelFilter, use_stderr: bool) -> Result<(), fern::InitError> {
    let output: fern::Output = if use_stderr {
        std::io::stderr().into()
    } else {
        std::io::stdout().into()
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "\x1B[{}m[{} {} {}] {}\x1B[0m",
                match record.level() {
                    Level::Error => "31", // Red
                    Level::Warn => "33",  // Yellow
                    Level::Info => "32",  // Green
                    Level::Debug => "34", // Blue
                    Level::Trace => "37", // White
                },
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(output)
        .apply()?;
    debug!("Initialized logging");
    Ok(())
}
