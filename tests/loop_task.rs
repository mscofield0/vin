use vin::*;

#[vin::task]
#[derive(Debug, Clone, PartialEq, Eq)]
struct MyTaskActor {
    pub number: u32,
}

#[async_trait]
impl vin::Task for MyTaskActor {
    async fn task(&self, ctx: Self::Context) -> anyhow::Result<()> {
        for i in 0..ctx.number {
            log::info!("{}. iteration", i);
        }

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

        let ctx = VinContextMyTaskActor {
            number: 5,
        };
        let actor = MyTaskActor::start("test_task", ctx).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        actor.close();
        vin::wait_for_shutdowns().await;
        assert_eq!(actor.state(), State::Closed);
        assert_eq!(actor.is_closed(), true);
    }
}