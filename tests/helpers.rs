extern crate actix_web;
extern crate serde_json;

extern crate ikrelln;

use actix_web::test::TestServer;

use ikrelln::api::http_application;

pub fn setup_server() -> TestServer {
    TestServer::with_factory(http_application)
}
