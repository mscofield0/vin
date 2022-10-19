use async_trait::async_trait;
use lazy_static::lazy_static;
use tokio::sync::Notify;
use std::{sync::atomic::{AtomicUsize, Ordering::*}, time::Duration};

pub trait Message {}

impl<T> Message for T {}

#[async_trait]
pub trait Forwarder<M: Message> {
    async fn forward(&self, msg: M);
}

#[async_trait]
pub trait Handler<M: Message> {
    async fn handle(&self, msg: M);
}

#[async_trait]
pub trait Actor {
    type Context;

    // impl'd by derive macro
    fn new(ctx: Self::Context) -> Self;
    fn state(&self) -> State;
    fn close(&self);
    async fn start(self) -> Addr<Self>;
    async fn ctx(&self) -> tokio::sync::RwLockReadGuard<Self::Context>;
    async fn ctx_mut(&self) -> tokio::sync::RwLockWriteGuard<Self::Context>;
    async fn send<M: Message + Send>(&self, msg: M) where Self: Forwarder<M>;

    // user defined
    async fn on_started(&self) {}
    async fn on_closing(&self) {}
    async fn on_closed(&self) {}

    // blanket
    fn is_started(&self) -> bool {
        self.state() != State::Pending
    }

    fn is_closed(&self) -> bool {
        self.state() == State::Closing || self.state() == State::Closed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pending,
    Starting,
    Running,
    Closing,
    Closed,
}

impl Default for State {
    fn default() -> Self {
        Self::Pending
    }
}

pub type Addr<A> = std::sync::Arc<A>;

pub static SHUTDOWN_SIGNAL: Notify = Notify::const_new();
pub static ACTORS_ALIVE: AtomicUsize = AtomicUsize::new(0);

pub fn shutdown() {
    SHUTDOWN_SIGNAL.notify_waiters();
}

pub fn add_actor() {
    ACTORS_ALIVE.fetch_add(1, Release);
}

pub fn remove_actor() {
    ACTORS_ALIVE.fetch_sub(1, Release);
}

pub async fn wait_for_shutdowns() {
    while ACTORS_ALIVE.load(Acquire) != 0 {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
