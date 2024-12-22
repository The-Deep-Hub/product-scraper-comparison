use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub fn setup_logging() {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .pretty()
        .init();

    info!("Logging initialized");
} 