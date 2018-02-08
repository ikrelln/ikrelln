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

    let system_and_actors = SystemAndActors::setup();

    api::serve(config.host, config.port, system_and_actors.ingestor);
    system_and_actors.system.run();
}


struct SystemAndActors {
    system: actix::SystemRunner,
    ingestor: actix::SyncAddress<engine::ingestor::Ingestor>,
}
impl SystemAndActors {
    fn setup() -> SystemAndActors {
        let system = actix::System::new("i'Krelln");
        let ingestor_actor: actix::SyncAddress<_> = engine::ingestor::Ingestor.start();

        SystemAndActors {
            system: system,
            ingestor: ingestor_actor,
        }
    }
}

lazy_static! {
    static ref DB_EXECUTOR_POOL: actix::SyncAddress<db::DbExecutor> = {
        let config = ::config::Config::load();
        actix::SyncArbiter::start(config.db_nb_connection, move || {
            db::DbExecutor(db::establish_connection(config.db_url.clone()))
        })
    };
}
