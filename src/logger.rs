extern crate simplelog;

use simplelog::*;

arg_enum! {
    #[derive(Debug)]
    pub enum LogLevel {
        Info,
        Warn,
        Debug
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Warn
    }
}

pub fn init(log_level: &LogLevel) {
    let level_filter = match log_level {
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Debug => LevelFilter::Debug,
    };

    let mut logger_array = vec![];
    match TermLogger::new(level_filter, Config::default(), TerminalMode::Mixed) {
        Some(logger) => logger_array.push(logger as Box<dyn SharedLogger>),
        None => logger_array.push(SimpleLogger::new(level_filter, Config::default())),
    }

    CombinedLogger::init(logger_array).expect("No logger should be already set");
    debug!("Initialized logging")
}
