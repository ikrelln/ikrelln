#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate diesel;

#[cfg(feature = "python")]
extern crate cpython;

use actix::prelude::*;

pub mod api;
mod build_info;
mod config;
mod db;
pub mod engine;
pub mod opentracing;

lazy_static! {
    static ref DB_EXECUTOR_POOL: actix::Addr<db::update::DbExecutor> = {
        let config = crate::config::Config::load();
        actix::SyncArbiter::start(1, move || {
            if let Ok(connection) = db::update::establish_connection(&config.db_url) {
                db::update::DbExecutor(Some(connection))
            } else {
                error!("error opening connection to DB");
                db::update::DbExecutor(None)
            }
        })
    };
}

lazy_static! {
    static ref DB_READ_EXECUTOR_POOL: actix::Addr<db::read::DbReadExecutor> = {
        let config = crate::config::Config::load();
        actix::SyncArbiter::start(3, move || {
            if let Ok(connection) = db::read::establish_connection(&config.db_url) {
                db::read::DbReadExecutor(Some(connection))
            } else {
                error!("error opening read connection to DB");
                db::read::DbReadExecutor(None)
            }
        })
    };
}

lazy_static! {
    #[derive(Debug)]
    static ref CONFIG: config::Config = config::Config::load();
}

pub fn start_server() {
    info!("Starting i'Krelln with config: {:?}", *CONFIG);

    let system = actix::System::new("i'Krelln");

    match std::env::var("LISTEN_FD") {
        Ok(fd) => api::serve_from_fd(&fd),
        _ => api::serve(&CONFIG.host, CONFIG.port),
    }

    //actix::System::current().registry()
    actix::System::current()
        .registry()
        .get::<crate::engine::streams::Streamer>()
        .do_send(crate::engine::streams::LoadScripts);

    let _: Addr<_> = db::cleanup::CleanUpTimer.start();

    system.run();
}
