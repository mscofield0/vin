#[doc(hidden)] pub use ::tokio;
#[doc(hidden)] pub use ::futures;
#[doc(hidden)] pub use ::log;
#[doc(hidden)] pub use ::crossbeam;
#[doc(hidden)] pub use ::async_trait;
#[doc(hidden)] pub use ::async_channel;
#[doc(hidden)] pub use ::anyhow;
#[doc(hidden)] pub use ::downcast_rs;

// only visible to other vin crates
pub mod detail;
use detail::*;

use downcast_rs::{Downcast, DowncastSync};
use async_trait::async_trait;
use std::{
    sync::atomic::Ordering,
    time::Duration
};
use tokio::sync::{
    RwLockReadGuard, 
    RwLockWriteGuard, 
    futures::Notified,
};

/// Actor lifecycle states.
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

/// Trait indicating that the type is a message.
pub trait Message: Downcast + Send {
    type Result;
}
downcast_rs::impl_downcast!(Message assoc Result);

/// Handler for specifying message handling logic.
#[async_trait]
pub trait Handler<M: Message> {
    async fn handle(&self, msg: M) -> anyhow::Result<M::Result>;
}

/// A restricted interface of `Actor` that provides send mechanics and state reads.
#[async_trait]
pub trait Addr: DowncastSync + Sync {

    /// Sends a typed message to the actor.
    async fn send<M: Message>(&self, msg: M)
        where
            Self: Sized + Forwarder<M>,
            M::Result: Send;

    /// Sends a typed message to the actor and awaits the result.
    async fn send_and_wait<M: Message>(&self, msg: M) -> anyhow::Result<M::Result>
        where
            Self: Sized + Forwarder<M>,
            M::Result: Send;
            
    /// Sends an erased message to the actor.
    async fn erased_send(&self, msg: BoxedMessage<()>);

    /// Returns the current state of the actor.
    /// 
    /// # Warning
    /// Beware of potential races, given that the state can be changed after you fetched it.
    fn state(&self) -> State;

    /// Closes the actor and gives the actor time to process the already-queued up 
    /// messages.
    fn close(&self);
    
    /// Returns this actor's close future.
    fn close_future(&self) -> Notified<'_>;

    /// Returns the id of the actor.
    fn id(&self) -> String;

    /// Checks if the actor has been started.
    fn is_started(&self) -> bool {
        self.state() != State::Pending && self.state() != State::Closed
    }

    /// Checks if the actor is closed.
    fn is_closed(&self) -> bool {
        self.state() == State::Closed
    }
}
downcast_rs::impl_downcast!(sync Addr);

/// Actor trait that all generic (non-specialized) actors must implement.
/// 
/// # Usage
/// Implementation is generated by the [`vin::actor`] proc macro.
/// 
/// # Behind the scenes
/// The way actors work in [`vin`] is that they live in their own [`tokio`] task, polling messages sent 
/// and starting handler tasks for each message received. Shutdown is also synchronized and actors 
/// will be given time to gracefully shutdown before being forcibly closed.
#[async_trait]
pub trait Actor: Addr {
    type Context;

    /// Returns a read lock to the underlying actor context.
    async fn ctx(&self) -> RwLockReadGuard<Self::Context>;

    /// Returns a write lock to the underlying actor context.
    async fn ctx_mut(&self) -> RwLockWriteGuard<Self::Context>;

    /// Creates and starts an actor with the given id (if available) and context.
    async fn start<Id: Into<ActorId> + Send>(id: Id, ctx: Self::Context) -> anyhow::Result<StrongAddr<Self>>;
}

/// Actor id type for the actor registry.
pub type ActorId = std::borrow::Cow<'static, str>;

/// A strong typed reference to the spawned actor.
pub type StrongAddr<A> = std::sync::Arc<A>;

/// A weak typed reference to the spawned actor.
pub type WeakAddr<A> = std::sync::Weak<A>;

/// A strong erased reference to the spawned actor.
pub type StrongErasedAddr = std::sync::Arc<dyn Addr>;

/// A weak erased reference to the spawned actor.
pub type WeakErasedAddr = std::sync::Weak<dyn Addr>;

/// A boxed Message.
pub type BoxedMessage<R> = Box<dyn Message<Result = R>>;

/// Sends a shutdown signal to all actors.
pub fn shutdown() {
    SHUTDOWN_SIGNAL.notify_waiters();
}

/// Registers a shutdown future.
/// 
/// Useful in loops aside from the main actor loops to cancel activities.
/// 
/// # Example
/// Pseudo Rust code to get the point across.
/// 
/// ```ignore
/// let shutdown = vin::shutdown_future();
/// tokio::pin!(shutdown);
/// 
/// loop {
///     tokio::select! {
///         msg = tcp_stream.read() => {
///             ...
///         },
///         ...
///         _ = &mut shutdown => {
///             info!("Received shutdown signal.");
///             break;
///         },
///     }
/// }
/// ```
pub fn shutdown_future<'a>() -> Notified<'a> {
    SHUTDOWN_SIGNAL.notified()
}

/// Registers an actor in the actor counter.
pub fn add_actor() {
    ACTORS_ALIVE.fetch_add(1, Ordering::Release);
}

/// Unregisters an actor in the actor counter.
pub fn remove_actor() {
    ACTORS_ALIVE.fetch_sub(1, Ordering::Release);
}

/// Waits for all actors to shutdown gracefully.
pub async fn wait_for_shutdowns() {
    while ACTORS_ALIVE.load(Ordering::Acquire) != 0 {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Error returned by the actor query functions.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum ActorQueryError {
    #[error("actor has expired and is either closing or closed")]
    Expired,

    #[error("actor is not found in the registry")]
    NotFound,

    #[error("invalid type of actor queried")]
    InvalidType,
}

/// Queries a typed actor from the registry.
pub async fn query_actor<A: Addr, Id: AsRef<str>>(id: Id) -> Result<StrongAddr<A>, ActorQueryError> {
    let addr = query_actor_erased(id).await?;
    let addr = if let Ok(addr) = addr.downcast_arc::<A>() {
        addr
    } else {
        return Err(ActorQueryError::InvalidType);
    };

    Ok(addr)
}

/// Queries an erased actor from the registry.
pub async fn query_actor_erased<Id: AsRef<str>>(id: Id) -> Result<StrongErasedAddr, ActorQueryError> {
    let reg = REGISTRY.lock().await;
    let addr = reg.get(id.as_ref());
    let addr = match addr {
        Some(addr) => if let Some(addr) = addr.upgrade() {
            if addr.is_closed() {
                return Err(ActorQueryError::Expired);
            } else {
                addr
            }
        } else {
            return Err(ActorQueryError::Expired);
        },
        None => {
            return Err(ActorQueryError::NotFound);
        },
    };

    Ok(addr)
}

/// Sends a typed message to an actor with the corresponding id.
pub async fn send_at<Id: AsRef<str>, M: Message<Result = ()>>(actor_id: Id, msg: M) {
    let addr = query_actor_erased(actor_id).await.unwrap(); // TODO add logging
    addr.erased_send(Box::new(msg)).await;
}

/// Sends an erased message to an actor with the corresponding id.
pub async fn erased_send_at<Id: AsRef<str>>(actor_id: Id, msg: BoxedMessage<()>) {
    let addr = query_actor_erased(actor_id).await.unwrap(); // TODO add logging
    addr.erased_send(msg).await;
}

/// Used to call arbitrary code on state changes.
/// 
/// ## Example
/// ```rust
/// #[derive(Debug, Clone)]
/// enum Message {
///     Foo,
///     Bar,
/// }
/// 
/// #[vin::actor]
/// #[vin::handles(Message)]
/// struct MyActor;
/// 
/// #[async_trait]
/// impl vin::Hooks for MyActor {
///     async fn on_started(&self) {
///         println!("Started!");
///     }
/// }
/// ```
#[async_trait]
pub trait Hooks {
    async fn on_started(&self) {}
    async fn on_closed(&self) {}
    async fn on_error(&self, _err: anyhow::Error) {}
}

/// Actor trait that all generic (non-specialized) actors must implement.
#[async_trait]
pub trait TaskActor: Task + TaskAddr {
    /// Creates and starts a task actor with the given id (if available) and context.
    async fn start<Id: Into<ActorId> + Send>(id: Id, ctx: Self::Context) -> StrongAddr<Self>;
}

/// A restricted interface of `TaskActor` that provides closing and state reads.
pub trait TaskAddr {
    /// Sends a close signal to the task actor.
    fn close(&self);
    
    /// Returns this task actor's close future.
    fn close_future(&self) -> Notified<'_>;

    /// Returns the state of the task actor.
    fn state(&self) -> State;

    /// Returns the id of the task actor.
    fn id(&self) -> String;

    /// Returns if the task actor is closed.
    fn is_closed(&self) -> bool {
        self.state() == State::Closed
    }
}

/// Used to call arbitrary code on a task actor.
#[async_trait]
pub trait Task: TaskContextTrait {
    /// Task function being executed.
    async fn task(&self, ctx: Self::Context) -> anyhow::Result<()>;
}

/// Private trait used by [`Task`] implementations to have access 
/// to the context associated type.
pub trait TaskContextTrait {
    /// The task's context type.
    type Context;
}
