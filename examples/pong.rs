use std::time::{Instant, Duration};
use vin::{*, Actor, tracing::Level};

// Messages
#[derive(Debug, Clone)]
struct Ping;

#[derive(Debug, Clone)]
struct Pong;

#[derive(Debug, Clone)]
struct SendPing;

// Actor impl
#[vin::actor]
#[vin::handles(Ping, bounded(size = 128, wait))]
#[vin::handles(Pong, bounded(size = 128, wait))]
#[vin::handles(SendPing, bounded(size = 128, wait))]
struct PingActor {
    /// The address of the counterpart actor
    addr: Option<vin::Addr<PingActor>>, // notice how we didn't specify `Self`, this is because the fields get wrapped into a context struct
                                        // which in turn means that `Self` becomes the context type and not the actor type 

    /// Time of the sent ping
    time_of_ping: Option<Instant>,
}

impl vin::LifecycleHook for PingActor {}

#[async_trait]
impl vin::Handler<Ping> for PingActor {
    async fn handle(&self, _: Ping) -> anyhow::Result<()> {
        let ctx = self.ctx().await;
        ctx.addr.as_ref().unwrap().send(Pong).await;
        Ok(())
    }
}

#[async_trait]
impl vin::Handler<Pong> for PingActor {
    async fn handle(&self, _: Pong) -> anyhow::Result<()> {
        let now = Instant::now();
        let mut ctx = self.ctx_mut().await;
        match ctx.time_of_ping {
            Some(start) => tracing::info!("Other side responded after {}ms.", (now - start).as_millis()),
            None => tracing::warn!("Other side responded with a pong message to no ping message."),
        }
        ctx.time_of_ping = None;
        Ok(())
    }
}

#[async_trait]
impl vin::Handler<SendPing> for PingActor {
    async fn handle(&self, _: SendPing) -> anyhow::Result<()> {
        let mut ctx = self.ctx_mut().await;
        if let Some(_) = ctx.time_of_ping {
            tracing::info!("Cancelled previous ping.");
        }
        ctx.time_of_ping = Some(Instant::now());

        ctx.addr.as_ref().unwrap().send(Ping).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let a1 = PingActor::new(VinContextPingActor { addr: None, time_of_ping: Default::default() });
    let a2 = PingActor::new(VinContextPingActor { addr: Some(a1.clone()), time_of_ping: Default::default() });
    a1.ctx_mut().await.addr = Some(a2.clone());

    a1.start().await;
    a2.start().await;

    a1.send(SendPing).await;
    a2.send(SendPing).await;

    tokio::time::sleep(Duration::from_millis(100)).await;
    vin::shutdown();
    vin::wait_for_shutdowns().await;
}