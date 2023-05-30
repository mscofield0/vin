use vin::prelude::*;
use std::time::{Instant, Duration};
use tracing::Level;

// Messages
#[vin::message]
#[derive(Debug, Clone)]
struct Ping;

#[vin::message]
#[derive(Debug, Clone)]
struct Pong;

#[vin::message]
#[derive(Debug, Clone)]
struct SendPing;

// Actor impl
#[vin::actor]
#[vin::handles(Ping, max = 128)]
#[vin::handles(Pong, max = 128)]
#[vin::handles(SendPing, max = 128)]
struct PingActor {
    /// Actor id to communicate with
    other_id: vin::ActorId,

    /// Time of the sent ping
    time_of_ping: Option<Instant>,
}

impl vin::Hooks for PingActor {}

#[async_trait]
impl vin::Handler<Ping> for PingActor {
    async fn handle(&self, _: Ping) -> Result<(), ()> {
        let ctx = self.ctx().await;
        vin::send::<PingActor, _, _>(ctx.other_id.clone(), Pong).await.expect("shouldn't have closed yet");
        Ok(())
    }
}

#[async_trait]
impl vin::Handler<Pong> for PingActor {
    async fn handle(&self, _: Pong) -> Result<(), ()> {
        let now = Instant::now();
        let mut ctx = self.ctx_mut().await;
        match ctx.time_of_ping {
            Some(start) => log::info!("actor '{}' responded after {}ms.", ctx.other_id, (now - start).as_millis()),
            None => log::warn!("Other side responded with a pong message to no ping message."),
        }
        ctx.time_of_ping = None;
        Ok(())
    }
}

#[async_trait]
impl vin::Handler<SendPing> for PingActor {
    async fn handle(&self, _: SendPing) -> Result<(), ()> {
        let mut ctx = self.ctx_mut().await;
        if let Some(_) = ctx.time_of_ping {
            log::info!("Cancelled previous ping.");
        }
        ctx.time_of_ping = Some(Instant::now());

        vin::send::<PingActor, _, _>(ctx.other_id.clone(), Ping).await.expect("shouldn't have closed yet");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let a1 = PingActor::start("test1", VinContextPingActor { other_id: "test2".into(), time_of_ping: Default::default() }).unwrap();
    let a2 = PingActor::start("test2", VinContextPingActor { other_id: "test1".into(), time_of_ping: Default::default() }).unwrap();

    a1.send(SendPing).await;
    a2.send(SendPing).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
    vin::shutdown();
    vin::wait_for_shutdowns().await;
}