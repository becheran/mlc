use std::time::SystemTime;

pub fn init(log_level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "\x1B[{}m[{} {} {}] {}\x1B[0m",
                match record.level() {
                    log::Level::Error => "31", // Red
                    log::Level::Warn => "33",  // Yellow
                    log::Level::Info => "32",  // Green
                    log::Level::Debug => "34", // Blue
                    log::Level::Trace => "37", // White
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
        .chain(std::io::stdout())
        .apply()?;
    debug!("Initialized logging");
    Ok(())
}
