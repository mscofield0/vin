use vin::*;

#[vin::message(result = u32)]
#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin::actor]
#[vin::handles(Msg)]
struct MyActor {
    pub number: u32,
}

#[async_trait]
impl vin::Hooks for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) -> Result<u32, ()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Ok(ctx.number)
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
        tokio::join!(
            actor.send(Msg::Bar),
            actor.send(Msg::Baz),
            actor.send(Msg::Foo),
            actor.send(Msg::Bar),
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}