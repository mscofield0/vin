use vin::*;

#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin::actor]
#[vin::handles(Msg, bounded(size = 1, report))]
struct MyActor {
    pub number: u32,
}

#[async_trait]
impl vin::LifecycleHook for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) -> anyhow::Result<()> {
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
        let actor = MyActor::new("test", ctx).start().await.unwrap();
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