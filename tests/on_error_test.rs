use vin::*;

#[vin::message]
#[derive(Debug, Clone)]
pub struct Msg;

#[vin::actor]
#[vin::handles(Msg, bounded(size = 1024, wait))]
struct MyActor;

#[async_trait]
impl vin::Hooks for MyActor {
    async fn on_error(&self, err: anyhow::Error) {
        log::error!("on_error says: {}", err);
    }
}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, _: Msg) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("hello world"))
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
        let actor = MyActor::start("test", ctx).await.unwrap();
        actor.send(Msg).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}