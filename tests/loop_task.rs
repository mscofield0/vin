use vin::*;

#[vin::task]
#[derive(Debug, Clone, PartialEq, Eq)]
struct MyTaskActor {
    pub number: u32,
}

#[async_trait]
impl vin::Task for MyTaskActor {
    async fn task(self) -> anyhow::Result<()> {
        for i in 0..self.number {
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

        let close_handle = MyTaskActor{ number: 5 }.start("test_task").await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        close_handle.close();
        vin::shutdown();
        vin::wait_for_shutdowns().await;
    }
}