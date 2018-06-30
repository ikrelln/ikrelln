#![deny(warnings)]
#![cfg_attr(feature = "cargo-clippy", deny(option_unwrap_used))]

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
    static ref DB_EXECUTOR_POOL: actix::Addr<actix::Syn, db::update::DbExecutor> = {
        let config = ::config::Config::load();
        actix::SyncArbiter::start(1, move || {
            if let Ok(connection) = db::update::establish_connection(&config.db_url) {
                return db::update::DbExecutor(Some(connection));
            } else {
                error!("error opening connection to DB");
                return db::update::DbExecutor(None);
            }
        })
    };
}

lazy_static! {
    static ref DB_READ_EXECUTOR_POOL: actix::Addr<actix::Syn, db::read::DbReadExecutor> = {
        let config = ::config::Config::load();
        actix::SyncArbiter::start(3, move || {
            if let Ok(connection) = db::read::establish_connection(&config.db_url) {
                return db::read::DbReadExecutor(Some(connection));
            } else {
                error!("error opening read connection to DB");
                return db::read::DbReadExecutor(None);
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
