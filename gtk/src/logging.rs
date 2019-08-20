use fern::{Dispatch, InitError};
use log::{Level, LevelFilter, Record};
use std::io;
use yansi::Paint;

/// Installs the fern logger, which manages all logs created with `log` statements.
pub fn install(filter: LevelFilter) -> Result<(), InitError> {
    let location = |record: &Record| {
        let mut target = record.target();
        if let Some(pos) = target.find(':') {
            target = &target[..pos];
        }

        match (record.file(), record.line()) {
            (Some(file), Some(line)) => format!(
                "{} {}:{}",
                Paint::new(target).bold(),
                Paint::magenta(file),
                Paint::cyan(line)
            ),
            _ => String::new(),
        }
    };

    let format_level = |record: &Record| match record.level() {
        level @ Level::Trace => Paint::green(level).bold(),
        level @ Level::Warn => Paint::yellow(level).bold(),
        level @ Level::Error => Paint::red(level).bold(),
        level => Paint::new(level).bold(),
    };

    Dispatch::new()
        // Exclude logs for crates that we use
        .level(LevelFilter::Off)
        // Include only the logs for relevant crates of interest
        .level_for("firmware_manager_gtk", filter)
        .level_for("firmware_manager", filter)
        .level_for("fwupd_dbus", filter)
        .level_for("system76_firmware_daemon", filter)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{:5}] {}: {}",
                format_level(record),
                location(record),
                message
            ))
        })
        .chain(io::stderr())
        .apply()?;
    Ok(())
}
