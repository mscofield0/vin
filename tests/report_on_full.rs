use vin::*;

#[vin::message(result = u32)]
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
impl vin::Hooks for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) -> anyhow::Result<<Msg as Message>::Result> {
        let ctx = self.ctx().await;
        let res = match msg {
            Msg::Foo => ctx.number + 42,
            Msg::Bar => ctx.number * 2,
            Msg::Baz => ctx.number + 100,
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tracing::Level;
    use vin::futures::FutureExt;
    use super::*;

    #[tokio::test]
    async fn test1() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();

        let ctx = VinContextMyActor { number: 42 };
        let actor = MyActor::start("test", ctx).await.unwrap();
        tokio::join!(
            actor.send_and_wait(Msg::Foo).map(|x| x.map(|x| log::info!("{}", x))),
            actor.send(Msg::Bar),
            actor.send(Msg::Baz),
            actor.send(Msg::Bar),
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}