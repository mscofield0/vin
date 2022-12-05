use std::time::Duration;

use vin::*;

#[vin::actor]
#[derive(Debug, Clone)]
struct MyActor {
    pub i: u32,
}

#[async_trait]
impl vin::LifecycleHook for MyActor {
    async fn on_run(&self) {
        let mut ctx = self.ctx_mut().await;
        tracing::info!("i is {}", ctx.i);
        ctx.i += 1;
        tokio::time::sleep(Duration::from_millis(10)).await;
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

        let ctx = VinContextMyActor { i: 0 };
        let _actor = MyActor::new("test", ctx).start().await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}