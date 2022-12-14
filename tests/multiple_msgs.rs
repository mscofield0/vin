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
#[derive(Debug, Clone)]
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
    use vin_core::StrongErasedAddr;
    use super::*;

    #[tokio::test]
    async fn test1() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();

        let ctx = VinContextMyActor { number: 42 };
        let actor = MyActor::new("test", ctx).start().await.unwrap();
        vin::send_to("test", MsgA::Bar).await;
        vin::send(actor.clone(), MsgA::Bar).await;
        vin::send_erased(actor.clone() as StrongErasedAddr, MsgA::Bar).await;
        actor.send(MsgA::Bar).await;
        actor.send(MsgB::Bar).await;
        actor.send_erased(Box::new(MsgA::Bar)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}