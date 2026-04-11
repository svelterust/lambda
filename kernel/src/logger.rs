use log::{LevelFilter, Log, Metadata, Record};
use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

struct FileLogger(Mutex<File>);

static LOGGER: OnceLock<FileLogger> = OnceLock::new();

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Ok(mut f) = self.0.lock() {
                let _ = writeln!(f, "[{}] {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut f) = self.0.lock() {
            let _ = f.flush();
        }
    }
}

pub fn init() {
    let file = File::create("/tmp/lambda.log").expect("Cannot create /tmp/lambda.log");
    let logger = LOGGER.get_or_init(|| FileLogger(Mutex::new(file)));
    let _ = log::set_logger(logger).map(|()| log::set_max_level(LevelFilter::Info));
    std::panic::set_hook(Box::new(|info| {
        let bt = std::backtrace::Backtrace::force_capture();
        log::error!("{info}\n{bt}");
    }));
}
