use vin::*;

#[vin::message]
#[derive(Debug, Clone)]
pub enum MsgA {
    Foo,
    Bar,
    Baz,
}

#[vin::message]
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
impl vin::Hooks for MyActor {}

#[async_trait]
impl vin::Handler<MsgA> for MyActor {
    async fn handle(&self, msg: MsgA) -> Result<(), ()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Err(())
    }
}

#[async_trait]
impl vin::Handler<MsgB> for MyActor {
    async fn handle(&self, msg: MsgB) -> Result<(), ()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Err(())
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
        let actor = MyActor::start("test", ctx).unwrap();
        vin::send::<MyActor, _, _>("test", MsgA::Bar).await.unwrap();
        
        vin::send_and_wait::<MyActor, _, _>("test", MsgA::Bar).await.unwrap().expect_err("expected error");
        actor.send(MsgA::Bar).await;
        actor.send(MsgB::Bar).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // sometimes, an actor will close before the message is sent
        actor.send_and_wait(MsgA::Bar).await.expect_err("expected error");

        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}