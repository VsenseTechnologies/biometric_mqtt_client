use chrono::Local;
use fern::Dispatch;

pub fn init_logger() -> Result<(), fern::InitError> {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} : [{}] {}",
                Local::now().format("%y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Error)
        .chain(fern::log_file("app.log")?)
        .apply()?;

    Ok(())
}
