use log::{debug, error, info, LevelFilter, warn};

pub fn log_to_file(message: String, level: LevelFilter) {
    match level {
        LevelFilter::Error => error!("{}", message),
        LevelFilter::Warn => warn!("{}", message),
        LevelFilter::Info => info!("{}", message),
        LevelFilter::Debug => debug!("{}", message),
        _ => {}
    }
}