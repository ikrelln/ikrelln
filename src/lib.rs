#![deny(warnings)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
#[macro_use]
extern crate log;
extern crate mime;

extern crate clap;
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

pub mod opentracing;
mod build_info;
mod config;
pub mod engine;
pub mod api;
mod db;

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

pub fn start_server() {
    let config = config::Config::load();

    info!("Starting i'Krelln with config: {:?}", config);

    let system = actix::System::new("i'Krelln");

    match std::env::var("LISTEN_FD") {
        Ok(fd) => api::serve_from_fd(fd),
        _ => api::serve(&config.host, config.port),
    }

    actix::Arbiter::system_registry()
        .get::<::engine::streams::Streamer>()
        .do_send(::engine::streams::LoadScripts);

    system.run();
}
