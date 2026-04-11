use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming};

fn resolved_log_spec(config_level: String) -> String {
    if let Ok(env) = std::env::var("RUST_LOG") {
        let t = env.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    let c = config_level.trim();
    if c.eq_ignore_ascii_case("none") {
        if cfg!(debug_assertions) {
            return "debug".to_string();
        }
        return "off".to_string();
    }
    c.to_string()
}

fn log_directory() -> String {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/Library/Logs/screen-buoy", home);
        }
    }
    "logs".to_string()
}

pub fn init_logger(log_level: String) -> Result<(), Box<dyn std::error::Error>> {
    let spec = resolved_log_spec(log_level);
    let log_dir = log_directory();
    Logger::try_with_str(&spec)?
        .log_to_stdout()
        .log_to_file(FileSpec::default().directory(&log_dir).basename("screen-buoy"))
        .rotate(
            Criterion::Size(3_000_000),
            Naming::Numbers,
            Cleanup::KeepLogFiles(15),
        )
        .format(|writer, now, record| {
            write!(
                writer,
                "[{}][{}][{}:{}] {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.line().unwrap_or(0),
                &record.args()
            )
        })
        .start()?;
    Ok(())
}