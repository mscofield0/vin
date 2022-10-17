use vin_macros;

#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin_macros::actor]
#[vin_macros::handles(Msg)]
struct MyActor {
    pub number: u32,
}

#[async_trait::async_trait]
impl vin_core::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) {
        println!("The message is: {:?}", msg);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tracing::Level;
    use vin_core::*;
    use super::*;

    #[tokio::test]
    async fn test1() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();

        let ctx = VinContextMyActor { number: 42 };
        let actor = MyActor::new(ctx).start().await;
        actor.send(Msg::Bar);
        tokio::time::sleep(Duration::from_millis(250)).await;
        actor.close();
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}