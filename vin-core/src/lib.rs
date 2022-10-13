use async_trait::async_trait;

pub trait Message {}

impl<T> Message for T {}

pub trait Forwarder<M: Message> {
    fn forward(&self, msg: M);
}

#[async_trait]
pub trait Handler<M: Message> {
    async fn handle(&self, msg: M);
}

#[async_trait]
pub trait Actor {
    type Context;

    // impl'd by derive macro
    fn send<M: Message>(&self, msg: M) where Self: Forwarder<M>;
    fn state(&self) -> State;
    fn close(&self);
    async fn start(self) -> Addr<Self>;
    async fn ctx(&self) -> tokio::sync::RwLockReadGuard<Self::Context>;
    async fn ctx_mut(&self) -> tokio::sync::RwLockWriteGuard<Self::Context>;

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
