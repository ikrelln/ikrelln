#![deny(warnings)]

extern crate chrono;
extern crate fern;
#[macro_use]
extern crate log;
extern crate mime;
extern crate time;

extern crate clap;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate futures;

use clap::{App, Arg};

use actix::Actor;

mod build_info;
mod engine;
mod http;

fn main() {
    let version: String = format!("v{}", build_info::BUILD_INFO.version);

    // configuration
    let matches = App::new("Krelln")
        .version(version.as_str())
        .about("Start Krelln server")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .value_name("PORT")
                .default_value("8080")
                .env("PORT")
                .help("Listen to the specified port"),
        )
        .get_matches();

    // log set up
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
        .unwrap();

    info!("Hello, world!");

    let system = actix::System::new("i'krelln");
    let ingestor_actor: actix::SyncAddress<_> = engine::ingestor::Ingestor.start();
    http::serve(port, ingestor_actor);
    system.run();
}
