# vin
A decentralized, ergonomic actor crate.

```toml
[dependencies]
vin = "0.1"
```

## Overview

Vin's goal is to be an ergonomic actor library. The reason many actor libraries are so tedious to use is because they rely on a central `System` that needs to be shared among all actors. Vin ditches that burden by making every actor instance completely independent. Each actor gets its own task to poll messages and execute handlers on. Its address is shared by a simple `Arc`. Vin also provides a way to gracefully shutdown all actors without having to do the manual labour yourself. Actor data is stored in its actor context and is retrievable for reading with `Actor::ctx()` and for writing with `Actor::ctx_mut()` which acquire a `RwLock` to the data.

Vin completely relies on [`tokio`](https://github.com/tokio-rs/tokio) (for the async runtime), [`tracing`](https://github.com/tokio-rs/tracing) (for diagnostics) and [`async_trait`](https://github.com/dtolnay/async-trait).

## Example

Basic usage of `vin`.

```rust
use std::time::Duration;
use tracing::Level;
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let ctx = VinContextMyActor { number: 42 };
    let actor = MyActor::new(ctx).start().await;
    actor.send(Msg::Bar).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    vin::shutdown();
    vin::wait_for_shutdowns().await;
}
```

## License

This project is licensed under the [MIT license](https://github.com/mscofield0/vin/blob/master/LICENSE).
