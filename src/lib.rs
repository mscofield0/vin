//! # vin
//! A lightweight, ergonomic and unconventional actor crate.
//! 
//! ## Overview
//! 
//! Vin's goal is to be an ergonomic, unconventional actor library. Vin doesn't follow the conventional implementations for actor libraries, but tries to be as simple as possible, while still providing an ergonomic and rich interface by integrating itself with [`tokio`] as much as possible. Each actor gets its own task to poll messages and execute handlers on. Its address is shared by a simple `Arc`. Vin also provides a way to gracefully shutdown all actors without having to do the manual labour yourself. Actor data is stored in its actor context and is retrievable for reading with `Actor::ctx()` and for writing with `Actor::ctx_mut()` which acquire a `RwLock` to the data. Vin also provides a "task actor" which is simply a [`tokio`] task spun up and synchronized with Vin's shutdown system.
//! 
//! Vin completely relies on [`tokio`](https://github.com/tokio-rs/tokio) (for the async runtime), [`log`](https://github.com/rust-lang/log) (for diagnostics) and [`async_trait`](https://github.com/dtolnay/async-trait).
//! 
//! ## Examples
//! 
//! ### Regular actors
//! Basic usage of [`vin`].
//! 
//! ```rust
//! use vin::prelude::*;
//! use std::time::Duration;
//! use tracing::Level;
//! 
//! #[vin::message]
//! #[derive(Debug, Clone)]
//! pub enum Msg {
//!     Foo,
//!     Bar,
//!     Baz,
//! }
//! 
//! #[vin::message(result = u32, error = String)]
//! struct MsgWithResult(bool);
//! 
//! #[vin::actor]
//! #[vin::handles(Msg)]
//! #[vin::handles(MsgWithResult)]
//! struct MyActor {
//!     pub number: u32,
//! }
//! 
//! #[async_trait]
//! impl vin::Hooks for MyActor {}
//! 
//! #[async_trait]
//! impl vin::Handler<Msg> for MyActor {
//!     async fn handle(&self, msg: Msg) -> Result<(), ()> {
//!         let ctx = self.ctx().await;
//!         println!("The message is: {:?} and the number is {}", msg, ctx.number);
//! 
//!         Ok(())
//!     }
//! }
//! 
//! #[async_trait]
//! impl vin::Handler<MsgWithResult> for MyActor {
//!     async fn handle(&self, MsgWithResult(should_err): MsgWithResult) -> Result<u32, String> {
//! 		if should_err { Err("error".to_string()) }
//!         else { Ok(42) }
//!     }
//! }
//! 
//! #[tokio::main]
//! async fn main() {
//!     tracing_subscriber::fmt()
//!         .with_max_level(Level::TRACE)
//!         .init();
//! 
//!     let ctx = VinContextMyActor { number: 42 };
//!     let actor = MyActor::start("test", ctx).unwrap();
//!     actor.send(Msg::Bar).await;
//!     tokio::time::sleep(Duration::from_millis(500)).await;
//! 	let res = actor.send_and_wait(MsgWithResult(false)).await.unwrap();
//! 	assert_eq!(res, 42);
//!     vin::shutdown();
//!     vin::wait_for_shutdowns().await;
//! }
//! ```
//! 
//! ### Task actors
//! Basic usage of task actors in [`vin`].
//! 
//! ```rust
//! use vin::*;
//! use std::time::Duration;
//! use tracing::Level;
//! 
//! #[vin::task]
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct MyTaskActor {
//!     pub number: u32,
//! }

//! #[async_trait]
//! impl vin::Task for MyTaskActor {
//!     async fn task(&self, ctx: Self::Context) -> anyhow::Result<()> {
//!         for i in 0..ctx.number {
//!             log::info!("{}. iteration", i);
//!         }
//! 
//!         Ok(())
//!     }
//! }
//! 
//! #[tokio::main]
//! async fn main() {
//!     tracing_subscriber::fmt()
//!         .with_max_level(Level::TRACE)
//!         .init();
//! 
//! 	let ctx = VinContextMyTaskActor { number: 5 };
//!     let actor = MyTaskActor::start("test_task", ctx);
//!     tokio::time::sleep(Duration::from_millis(100)).await;
//! 	actor.close();
//!     vin::shutdown();
//!     vin::wait_for_shutdowns().await;
//! }
//! ```

pub use vin_core::{
	self, State, Message, Addr, Actor, ActorId, Handler,
	StrongAddr, WeakAddr, StrongErasedAddr, WeakErasedAddr,
	shutdown, shutdown_future, add_actor, remove_actor,
	wait_for_shutdowns, ActorQueryError, ActorStartError, query_actor, query_actor_erased,
	send, send_and_wait, Hooks, TaskActor, Task, TaskAddr,
};
pub use vin_macros::{actor, handles, task, message};

#[doc(hidden)] pub use ::vin_core::anyhow;
#[doc(hidden)] pub use ::vin_core::async_channel;
#[doc(hidden)] pub use ::vin_core::async_trait::{self, async_trait};
#[doc(hidden)] pub use ::vin_core::crossbeam;
#[doc(hidden)] pub use ::vin_core::downcast_rs;
#[doc(hidden)] pub use ::vin_core::futures;
#[doc(hidden)] pub use ::vin_core::tokio;
#[doc(hidden)] pub use ::vin_core::log;

#[doc(hidden)]
pub mod prelude {
	pub use crate::{Addr, Actor, StrongAddr, WeakAddr, StrongErasedAddr, WeakErasedAddr, TaskActor, TaskAddr};
	
	pub use crate::anyhow;
	pub use crate::async_channel;
	pub use crate::async_trait::{self, async_trait};
	pub use crate::crossbeam;
	pub use crate::downcast_rs;
	pub use crate::futures;
	pub use crate::tokio;
	pub use crate::log;
}