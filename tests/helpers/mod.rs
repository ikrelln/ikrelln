#![allow(dead_code)]

extern crate actix_web;
extern crate chrono;
extern crate fern;
extern crate log;
extern crate serde_json;

extern crate ikrelln;

use std as TheStd;

use actix_web::test::TestServer;

use ikrelln::api::http_application;

pub const DELAY_SPAN_SAVED_MILLISECONDS: u64 = 200;
pub const DELAY_RESULT_SAVED_MILLISECONDS: u64 = 200;
pub const DELAY_REPORT_SAVED_MILLISECONDS: u64 = 500;
pub const DELAY_SCRIPT_SAVED_MILLISECONDS: u64 = 100;
pub const DELAY_FINISH: u64 = 500;

pub fn setup_server() -> TestServer {
    TestServer::with_factory(http_application)
}

static mut LOGGER_SET_UP: bool = false;
pub fn setup_logger() {
    if unsafe { !LOGGER_SET_UP } {
        unsafe { LOGGER_SET_UP = true };
        fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Debug)
            .level_for("tokio_core", log::LevelFilter::Error)
            .level_for("tokio_reactor", log::LevelFilter::Error)
            .chain(TheStd::io::stdout())
            .apply()
            .unwrap();
    }
}
