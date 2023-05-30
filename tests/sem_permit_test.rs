use std::{sync::{atomic::{AtomicU32, Ordering}, Arc}, time::Duration};

use vin::*;

#[vin::message]
#[derive(Debug, Clone)]
pub struct Msg;

#[vin::actor]
#[vin::handles(Msg, max = 5)]
struct MyActor;

impl vin::Hooks for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, _: Msg) -> Result<(), ()> {
        log::info!("Going into 500ms sleep...");
        tokio::time::sleep(Duration::from_millis(500)).await;
        log::info!("Awake");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tracing::Level;
    use super::*;

    #[tokio::test]
    async fn test1() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();

        let ctx = VinContextMyActor;
        let actor = MyActor::start("test", ctx).unwrap();
        actor.send(Msg).await;
        actor.send(Msg).await;
        actor.send(Msg).await;
        actor.send(Msg).await;
        actor.send(Msg).await;

        // Should be blocked
        actor.send(Msg).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}