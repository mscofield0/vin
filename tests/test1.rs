use vin::*;

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
impl vin::LifecycleHook for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);
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
        actor.send(Msg::Bar).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        vin_core::shutdown();
        vin_core::wait_for_shutdowns().await;
    }
}