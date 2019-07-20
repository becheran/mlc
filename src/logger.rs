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

pub fn init(log_level: &LogLevel) {
    let level_filter =
        match log_level {
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Debug => LevelFilter::Debug,
        };

    CombinedLogger::init(
        vec![
            TermLogger::new(level_filter,
                            Config::default(),
                            TerminalMode::Mixed).unwrap(),
        ]
    ).unwrap();
    debug!("Initialized logging")
}