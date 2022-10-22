use vin::*;

#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin::actor]
#[vin::handles(Msg, bounded(size = 1, wait))]
struct MyActor {
    pub number: u32,
}

#[async_trait]
impl vin::LifecycleHook for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    type Error = String;

    async fn handle(&self, msg: Msg) -> Result<(), HandlerError<Msg, Self::Error>> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Err(HandlerError::new(String::from("hi, i am error")))
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
        tokio::join!(
            actor.send(Msg::Bar),
            actor.send(Msg::Baz),
            actor.send(Msg::Foo),
            actor.send(Msg::Bar),
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
        vin_core::shutdown();
        vin_core::wait_for_shutdowns().await;
    }
}