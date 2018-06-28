#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
#[macro_use]
extern crate log;
extern crate mime;

extern crate clap;
#[macro_use]
extern crate structopt;
extern crate toml;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_urlencoded;
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

use actix::prelude::*;

pub mod api;
mod build_info;
mod config;
mod db;
pub mod engine;
pub mod opentracing;

lazy_static! {
    static ref DB_EXECUTOR_POOL: actix::Addr<actix::Syn, db::DbExecutor> = {
        let config = ::config::Config::load();
        actix::SyncArbiter::start(config.db_nb_connection, move || {
            if let Ok(connection) = db::establish_connection(&config.db_url) {
                return db::DbExecutor(Some(connection));
            } else {
                error!("error opening connection to DB");
                return db::DbExecutor(None);
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

    actix::Arbiter::system_registry()
        .get::<::engine::streams::Streamer>()
        .do_send(::engine::streams::LoadScripts);

    let _: Addr<Syn, _> = db::cleanup::CleanUpTimer.start();

    system.run();
}
