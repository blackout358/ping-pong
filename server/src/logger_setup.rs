use crossterm::style::Stylize;
use flexi_logger::Logger;
use log::Level;

pub fn init_logger() {
    // env::set_var("RUST_LOG", "debug"); // You can control this externally too
    // let file_spec = FileSpec::default()
    //     .directory("./")
    //     .basename("Client Runtime")
    //     .suppress_timestamp()
    //     .suffix("log");

    Logger::try_with_str("debug")
        .unwrap()
        // .log_to_file(file_spec)
        .format(|writer, now, record| {
            let timestamp = now.now().format("%Y-%m-%d %H:%M:%S");
            // Colorize level
            let level = match record.level() {
                Level::Error => "ERROR".red(),
                Level::Warn => "WARN ".yellow(),
                Level::Info => "INFO ".green(),
                Level::Debug => "DEBUG".blue(),
                Level::Trace => "TRACE".blue(),
            };

            // Non-Colourized
            // let level = match record.level() {
            //     Level::Error => "ERROR",
            //     Level::Warn => "WARN ",
            //     Level::Info => "INFO ",
            //     Level::Debug => "DEBUG",
            //     Level::Trace => "TRACE",
            // };
            // Get thread name or fallback
            let binding = std::thread::current();
            let thread = binding.name().unwrap_or("unnamed");

            let line = record.line().map_or("".to_string(), |l| format!("L{}", l));
            let module = record.module_path().unwrap_or("?");

            write!(
                writer,
                "{} :: {} :: {} :: {}() :: {} :: {}",
                timestamp,
                level,
                thread,
                module,
                line,
                record.args()
            )
        })
        .start()
        .unwrap();
}
