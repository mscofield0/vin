use std::time::Duration;
use vin::*;

#[derive(Debug, Clone)]
enum Msg {
    A,
    B,
    C,
}

#[vin::actor]
#[vin::handles(Msg, bounded(size = 1024, wait))]
#[derive(Debug, Clone)]
struct ActorA {
    pub rx: vin::async_channel::Receiver<()>,
}

#[vin::actor]
#[vin::handles(Msg, bounded(size = 1024, wait))]
#[derive(Debug, Clone)]
struct ActorB {
    pub tx: vin::async_channel::Sender<()>,
    pub i: usize,
}

#[async_trait]
impl vin::LifecycleHook for ActorA {
    async fn on_run(&self) {
        let ctx = self.ctx().await;
        if let Ok(_) = ctx.rx.recv().await {
            tracing::info!("recv: received msg");
        } else {
            self.close();
        }
    }
}

#[async_trait]
impl vin::LifecycleHook for ActorB {
    async fn on_run(&self) {
        let mut ctx = self.ctx_mut().await;
        if ctx.i >= 5 { 
            self.close();
            return;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        ctx.tx.send(()).await.unwrap();
        ctx.i += 1;
        tracing::info!("send: i is {}", ctx.i);
    }
}

#[async_trait]
impl vin::Handler<Msg> for ActorA {
    async fn handle(&self, _msg: Msg) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl vin::Handler<Msg> for ActorB {
    async fn handle(&self, _msg: Msg) -> anyhow::Result<()> {
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

        let (tx, rx) = vin::async_channel::bounded(1024);
        let ctx_a = VinContextActorA { rx };
        let ctx_b = VinContextActorB { tx, i: 0 };
        let actor_a = ActorA::new("a", ctx_a).start().await.unwrap();
        let actor_b = ActorB::new("b", ctx_b).start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(150)).await;
        actor_a.send(Msg::A).await;
        actor_b.send(Msg::B).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}