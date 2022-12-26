#[doc(hidden)] pub use ::tokio;
#[doc(hidden)] pub use ::futures;
#[doc(hidden)] pub use ::tracing;
#[doc(hidden)] pub use ::crossbeam;
#[doc(hidden)] pub use ::async_trait;
#[doc(hidden)] pub use ::async_channel;
#[doc(hidden)] pub use ::anyhow;
#[doc(hidden)] pub use ::downcast_rs;

use async_trait::async_trait;
use lazy_static::lazy_static;
use downcast_rs::{Downcast, DowncastSync};
use tokio::sync::{Notify, futures::Notified};
use std::{
    sync::{atomic::{AtomicUsize, Ordering::*}, Arc}, 
    time::Duration, 
    fmt::Debug, 
    collections::HashMap,
};

/// Marker trait indicating that the type is a message.
pub trait Message: Downcast + Send {}

impl<T: Send + 'static> Message for T {}
downcast_rs::impl_downcast!(Message);

/// Forwards the message to the respective mailbox.
#[async_trait]
pub trait Forwarder<M: Message> {
    async fn forward(&self, msg: M);
}

/// Handler for specifying message handling logic.
#[async_trait]
pub trait Handler<M: Message> {
    async fn handle(&self, msg: M) -> anyhow::Result<()>;
}

/// Helper wrapper that provides message name information alongside the handler error.
pub struct HandlerError {
    inner: anyhow::Error,
    msg_name: &'static str,
}

impl HandlerError {
    pub fn new(msg_name: &'static str, err: anyhow::Error) -> Self {
        Self { msg_name, inner: err }
    }

    pub fn msg_name(&self) -> &'static str {
        self.msg_name
    }
}

impl ::core::fmt::Debug for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// Error returned on actor start.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum StartError {
    #[error("the actor has already been started")]
    AlreadyStarted,
    
    #[error("the id ({0}) is already taken")]
    AlreadyTakenId(ActorId),
}

/// Actor trait that all generic (non-specialized) actors must implement.
/// 
/// Implementation is generated by the [`vin::actor`] proc macro.
/// 
/// The way actors work in [`vin`] is that they live in their own [`tokio`] task, polling messages sent 
/// and starting handler tasks for each message received. Shutdown is also synchronized and actors 
/// will be given time to gracefully shutdown before being forcibly closed.
#[async_trait]
pub trait Actor: Addr + LifecycleHook {
    type Context;

    // impl'd by attr macro

    /// Creates a new actor with the given id and context.
    fn new<Id: Into<ActorId> + Send>(id: Id, ctx: Self::Context) -> StrongAddr<Self>;

    /// Returns a read lock to the underlying actor context.
    async fn ctx(&self) -> tokio::sync::RwLockReadGuard<Self::Context>;

    /// Returns a write lock to the underlying actor context.
    async fn ctx_mut(&self) -> tokio::sync::RwLockWriteGuard<Self::Context>;

    /// Starts the actor if:
    /// - The ID is not taken
    /// - Hasn't already been started
    async fn start(self: &StrongAddr<Self>) -> Result<StrongAddr<Self>, StartError>
        where Self: Sized;
}

/// A restricted interface of `Actor` that provides send mechanics and state reads.
#[async_trait]
pub trait Addr: DowncastSync + Sync {
    // impl'd by attr macro

    /// Sends a typed message to the actor.
    /// 
    /// # Note
    /// Requires that the actor type it's being called on be `Sized`.
    async fn send<M: Message>(&self, msg: M) where Self: Forwarder<M> + Sized;

    /// Sends an erased message to the actor.
    /// 
    /// The actor stores each registered message's type id, so the erased message 
    /// still ends up in the correct mailbox.
    async fn send_erased(&self, msg: BoxedMessage);

    /// Returns the current state of the actor.
    /// 
    /// # Warning
    /// Beware of potential races, given that the state can be changed after you fetched it.
    fn state(&self) -> State;

    /// Closes the actor and gives the actor time to process the already-queued up 
    /// messages.
    fn close(&self);

    /// Returns the ID of the actor.
    fn id(&self) -> &str;
    
    /// Checks if the actor has been started.
    fn is_started(&self) -> bool {
        self.state() != State::Pending
    }

    /// Checks if the actor is being or is closed.
    fn is_closed(&self) -> bool {
        self.state() == State::Closing || self.state() == State::Closed
    }
}
downcast_rs::impl_downcast!(sync Addr);

/// Sends a typed message to an actor with the corresponding id.
pub async fn send_to<Id: AsRef<str>, M: Message>(actor_id: Id, msg: M) {
    let addr = query_actor_erased(actor_id).await.unwrap(); // TODO add logging
    addr.send_erased(Box::new(msg)).await;
}

/// Sends a typed message to an actor.
pub async fn send<A: Addr + Forwarder<M>, M: Message>(addr: StrongAddr<A>, msg: M) {
    addr.send(msg).await;
}

/// Sends a typed message to an actor.
pub async fn send_erased<M: Message>(addr: StrongErasedAddr, msg: M) {
    addr.send_erased(Box::new(msg)).await;
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
/// impl vin::LifecycleHook for MyActor {
///     async fn on_started(&self) {
///         println!("Started!");
///     }
/// }
/// ```
#[async_trait]
pub trait LifecycleHook {
    async fn on_started(&self) {}
    async fn on_closed(&self) {}
}

/// A close handle with which to close a task actor.
/// 
/// # Usage
/// 
/// ```ignore
/// let close_handle = TaskCloseHandle::default();
/// let close = close_handle.close_future();
/// tokio::pin!(close);
/// 
/// loop {
///     tokio::select! {
///         msg = tcp_stream.read() => {
///             ...
///         },
///         ...
///         _ = &mut close => {
///             info!("Received close signal.");
///             break;
///         },
///     }
/// }
/// ```
pub struct TaskCloseHandle {
    notifier: Notify,
}

impl TaskCloseHandle {
    /// Registers a shutdown future.
    pub fn close_future(&self) -> Notified {
        self.notifier.notified()
    }

    /// Sends close signal to task.
    pub fn close(&self) {
        self.notifier.notify_waiters();
    }
}

impl Default for TaskCloseHandle {
    fn default() -> Self {
        Self { notifier: Notify::const_new() }
    }
}

/// Actor trait that all generic (non-specialized) actors must implement.
#[async_trait]
pub trait TaskActor: Task {
    /// Starts the task actor.
    async fn start<Id: Into<ActorId> + Send>(self, id: Id) -> Arc<TaskCloseHandle>
        where Self: Sized;
}

/// Used to call arbitrary code on a task actor.
/// 
/// # Note
/// Unlike normal [`vin`] actors, a task actor does not provide a way to hook into its lifetime 
/// with `on_started()` and `on_closed()` methods. This is because I can't find a good way to 
/// supply the task future with a `&mut self`. If anyone figures out how to do it, submit a PR.
#[async_trait]
pub trait Task {
    /// Task function being executed.
    async fn task(self) -> anyhow::Result<()>;
}

/// Used in the attribute macro to implement an erased -> typed message sending.
#[async_trait]
pub trait ErasedTx: Send + Sync {
    async fn send_erased(&self, msg: BoxedMessage);
}

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

/// Actor identifier marker trait for the actor registry.
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
pub type BoxedMessage = Box<dyn Message>;

/// A boxed ErasedTx.
pub type BoxedErasedTx = Box<dyn ErasedTx>;

/// Global actor shutdown signal.
pub static SHUTDOWN_SIGNAL: Notify = Notify::const_new();

/// Number of actors alive.
pub static ACTORS_ALIVE: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// An actor registry that is queriable via `query_actor()` and `query_actor_erased()`.
    pub static ref REGISTRY: tokio::sync::Mutex<HashMap<ActorId, WeakErasedAddr>> = Default::default();
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
    ACTORS_ALIVE.fetch_add(1, Release);
}

/// Unregisters an actor in the actor counter.
pub fn remove_actor() {
    ACTORS_ALIVE.fetch_sub(1, Release);
}

/// Waits for all actors to shutdown gracefully.
pub async fn wait_for_shutdowns() {
    while ACTORS_ALIVE.load(Acquire) != 0 {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
