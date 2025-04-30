use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming};

pub fn init_logger(log_level: String) -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str(log_level)?
        .log_to_stdout()
        .log_to_file(FileSpec::default().directory("./logs").basename("screen-buoy"))
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