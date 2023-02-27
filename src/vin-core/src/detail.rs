use std::{sync::atomic::AtomicUsize, collections::HashMap};
use async_trait::async_trait;
use tokio::sync::{oneshot, Notify};
use crate::{Message, BoxedMessage, ActorId, WeakErasedAddr};
use lazy_static::lazy_static;

/// Forwards the message to the respective mailbox.
#[async_trait]
pub trait Forwarder<M: Message> {
    async fn forward(&self, msg: WrappedMessage<M>);
}

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

/// A message wrapper type that enables returning results through the result channel packed inside it.
pub struct WrappedMessage<M: Message> {
    /// The actual message to be sent.
    pub msg: M,

    /// The result channel used to send the result, if the result type isn't `()`.
    pub result_channel: Option<oneshot::Sender<anyhow::Result<M::Result>>>,
}

/// Helper wrapper that provides message name information alongside the handler error.
pub struct HandlerError {
    pub inner: anyhow::Error,
    pub msg_name: &'static str,
}

impl HandlerError {
    /// Creates a new `HandlerError`.
    pub fn new(msg_name: &'static str, err: anyhow::Error) -> Self {
        Self { msg_name, inner: err }
    }

    /// Returns the message type name.
    pub fn msg_name(&self) -> &'static str {
        self.msg_name
    }
}

impl ::core::fmt::Debug for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

/// Used in the attribute macro to implement an erased -> typed message sending.
#[async_trait]
pub trait ErasedTx: Send + Sync {
    async fn erased_send(&self, msg: BoxedMessage<()>);
}