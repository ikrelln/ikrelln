#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate fern;
#[macro_use]
extern crate log;
extern crate mime;

extern crate clap;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate failure;
extern crate futures;

#[macro_use]
extern crate diesel;

#[cfg(feature = "python")]
extern crate cpython;

mod build_info;
mod config;
mod engine;
mod api;
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

    let system = actix::System::new("i'Krelln");

    api::serve(&config.host, config.port);

    actix::Arbiter::system_registry()
        .get::<::engine::streams::Streamer>()
        .do_send(::engine::streams::LoadScripts);

    system.run();
}

lazy_static! {
    static ref DB_EXECUTOR_POOL: actix::Addr<actix::Syn, db::DbExecutor> = {
        let config = ::config::Config::load();
        actix::SyncArbiter::start(config.db_nb_connection, move || {
            db::DbExecutor(db::establish_connection(&config.db_url))
        })
    };
}
