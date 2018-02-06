use actix::prelude::*;
use std::time::Duration;

#[derive(Default)]
pub struct Batcher;

impl Actor for Batcher {
    type Context = Context<Self>;
}
impl actix::Supervised for Batcher {}

impl actix::ArbiterService for Batcher {
    fn service_started(&mut self, ctx: &mut Context<Self>) {
        info!("Batcher started");
        ctx.notify_later(Batch, Duration::new(5 * 60, 0));
    }
}

#[derive(Message)]
pub struct Register(pub String);
impl Handler<Register> for Batcher {
    type Result = ();

    fn handle(&mut self, msg: Register, _ctx: &mut Context<Self>) {
        info!("adding trace {}", msg.0);
    }
}

#[derive(Message)]
struct Batch;
impl Handler<Batch> for Batcher {
    type Result = ();

    fn handle(&mut self, _: Batch, ctx: &mut Context<Self>) {
        info!("batch");
        ctx.notify_later(Batch, Duration::new(5 * 60, 0));
    }
}
