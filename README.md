# vin
A lightweight, ergonomic and unconventional actor crate.

[![Crates.io][crates-badge]][crates-url] [![Documentation][docs-badge]][docs-url] [![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/d/vin
[crates-url]: https://crates.io/crates/vin
[docs-badge]: https://img.shields.io/docsrs/vin
[docs-url]: https://docs.rs/vin
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE

```toml
[dependencies]
vin = "6.0"
```

## Overview

Vin's goal is to be an ergonomic, unconventional actor library. Vin doesn't follow the conventional implementations for actor libraries, but tries to be as simple as possible, while still providing an ergonomic and rich interface by integrating itself with [`tokio`](https://github.com/tokio-rs/tokio) as much as possible. Each actor gets its own task to poll messages and execute handlers on. Its address is shared by a simple `Arc`. Vin also provides a way to gracefully shutdown all actors without having to do the manual labour yourself. Actor data is stored in its actor context and is retrievable for reading with `Actor::ctx()` and for writing with `Actor::ctx_mut()` which acquire a `RwLock` to the data. Vin also provides a "task actor" which is simply a [`tokio`](https://github.com/tokio-rs/tokio) task spun up and synchronized with Vin's shutdown system.

Vin completely relies on [`tokio`](https://github.com/tokio-rs/tokio) (for the async runtime), [`log`](https://github.com/rust-lang/log) (for diagnostics), [`async_trait`](https://github.com/dtolnay/async-trait) and [`anyhow`](https://github.com/dtolnay/anyhow) (as the handler error type).

## Examples

### Regular actors
Basic usage of [`vin`](https://github.com/mscofield0/vin).

```rust
use vin::*;
use std::time::Duration;
use tracing::Level;

#[vin::message]
#[derive(Debug, Clone)]
pub enum Msg {
    Foo,
    Bar,
    Baz,
}

#[vin::message(result = u32)]
#[derive(Debug, Clone)]
pub struct MsgWithResult;

#[vin::actor]
#[vin::handles(Msg)]
struct MyActor {
    pub number: u32,
}

#[async_trait]
impl vin::Hooks for MyActor {}

#[async_trait]
impl vin::Handler<Msg> for MyActor {
    async fn handle(&self, msg: Msg) -> anyhow::Result<()> {
        let ctx = self.ctx().await;
        println!("The message is: {:?} and the number is {}", msg, ctx.number);

        Ok(())
    }
}

#[async_trait]
impl vin::Handler<MsgWithResult> for MyActor {
    async fn handle(&self, _: MsgWithResult) -> anyhow::Result<<MsgWithResult as Message>::Result> {
        Ok(42)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let ctx = VinContextMyActor { number: 42 };
    let actor = MyActor::start("test", ctx).await.unwrap();
    actor.send(Msg::Bar).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let res = actor.send_and_wait(MsgWithResult).await.unwrap();
    assert_eq!(res, 42);
    vin::shutdown();
    vin::wait_for_shutdowns().await;
}
```

### Task actors
Basic usage of task actors in [`vin`](https://github.com/mscofield0/vin).

```rust
use vin::*;
use std::time::Duration;
use tracing::Level;

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

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let ctx = VinContextMyTaskActor { number: 5 };
    let actor = MyTaskActor::start("test_task", ctx).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    actor.close();
    vin::shutdown();
    vin::wait_for_shutdowns().await;
}
```

## License

This project is licensed under the [MIT license](https://github.com/mscofield0/vin/blob/master/LICENSE).
