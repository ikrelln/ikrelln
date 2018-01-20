//#![deny(warnings)]

extern crate chrono;
extern crate fern;
#[macro_use]
extern crate log;
extern crate mime;
extern crate time;

extern crate clap;

#[macro_use]
extern crate lazy_static;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate futures;

use clap::{App, Arg};

mod build_info;
mod engine;
mod http;

fn main() {
    let version: String = format!("v{}", build_info::BUILD_INFO.version);

    let matches = App::new("Krelln")
        .version(version.as_str())
        .about("Start Krelln server")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .value_name("PORT")
                .help("Listen to the specified port"),
        )
        .get_matches();

    fern::Dispatch::new()
        .level(log::LogLevelFilter::Trace)
        .level_for("krelln", log::LogLevelFilter::Trace)
        .level_for("tokio_core", log::LogLevelFilter::Error)
        .level_for("mio", log::LogLevelFilter::Error)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                chrono::UTC::now().format("%Y-%m-%d %H:%M:%S%.9f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();

    let port = matches
        .value_of("port")
        .and_then(|it| it.parse::<u16>().ok())
        .unwrap_or(9999);

    info!("Hello, world!");

    http::serve(port);
}
