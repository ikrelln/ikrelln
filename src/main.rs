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

#[macro_use]
extern crate diesel;

use actix::Actor;

mod build_info;
mod config;
mod engine;
mod http;
mod db;

fn main() {
    // log set up
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

    let config = config::Config::load();

    info!("Starting i'Krelln with config: {:?}", config);

    info!(
        "you can connect to the UI at http://{}:{}",
        config.host,
        config.port
    );

    let system = actix::System::new("i'Krelln");
    let db_actor = {
        let db_url = config.db_url;
        actix::SyncArbiter::start(config.db_nb_connection, move || {
            db::DbExecutor(db::establish_connection(db_url.clone()))
        })
    };
    let ingestor_actor: actix::SyncAddress<_> = engine::ingestor::Ingestor(db_actor).start();
    http::serve(config.host, config.port, ingestor_actor);
    system.run();
}
