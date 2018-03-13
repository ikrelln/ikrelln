extern crate chrono;
extern crate fern;
extern crate log;

extern crate ikrelln;

fn main() {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .level_for("ikrelln", log::LevelFilter::Trace)
        .level_for("tokio_core", log::LevelFilter::Error)
        .level_for("mio", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.9f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();

    ikrelln::start_server();
}
