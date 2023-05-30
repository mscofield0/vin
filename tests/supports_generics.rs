use vin::*;
use std::fmt::Debug;

#[vin::message]
#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin::actor]
#[vin::handles(Msg, max = 5)]
struct MyActor<T: Debug + Clone + Send + Sync + 'static> {
    value: T,
}

#[async_trait]
impl<T: Debug + Clone + Send + Sync> vin::Handler<Msg> for MyActor<T> {
    async fn handle(&self, msg: Msg) -> Result<(), ()> {
        log::info!("The message is: {:?}", msg);
        let ctx = self.ctx().await;
        log::info!("Value is: {:?}", ctx.value);

        Ok(())
    }
}

#[async_trait]
impl<T: Debug + Clone + Send + Sync> vin::Hooks for MyActor<T> {}

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

        let ctx = VinContextMyActor { value: 8 };
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