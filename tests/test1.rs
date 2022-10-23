use vin::*;

#[derive(Debug, Clone)]
pub enum MsgA {
    Foo,
    Bar,
    Baz,
}

#[derive(Debug, Clone)]
pub enum MsgB {
    Foo,
    Bar,
    Baz,
}

#[vin::actor]
#[vin::handles(MsgA)]
#[vin::handles(MsgB)]
struct MyActor {
    pub number: u32,
}

#[async_trait]
impl vin::LifecycleHook for MyActor {}

#[async_trait]
impl vin::Handler<MsgA> for MyActor {
    async fn handle(&self, msg: MsgA) -> anyhow::Result<()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Err(anyhow::anyhow!("hi, i am error"))
    }
}

#[async_trait]
impl vin::Handler<MsgB> for MyActor {
    async fn handle(&self, msg: MsgB) -> anyhow::Result<()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Err(anyhow::anyhow!("hi, i am error"))
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

        let ctx = VinContextMyActor { number: 42 };
        let actor = MyActor::new(ctx).start().await;
        actor.send(MsgA::Bar).await;
        actor.send(MsgB::Bar).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        vin_core::shutdown();
        vin_core::wait_for_shutdowns().await;
    }
}