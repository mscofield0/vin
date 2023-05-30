use std::{sync::atomic::AtomicUsize, collections::HashMap};
use async_trait::async_trait;
use tokio::sync::{oneshot, Notify};
use crate::{Message, ActorId, WeakErasedAddr};
use lazy_static::lazy_static;

/// Forwards the message to the respective mailbox.
#[async_trait]
pub trait Forwarder<M: Message> {
    async fn forward(&self, msg: WrappedMessage<M>);
}

/// Global actor shutdown signal.
pub static SHUTDOWN_SIGNAL: Notify = Notify::const_new();

/// Number of actors alive.
pub static ACTORS_ALIVE: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// An actor registry that is queriable via `query_actor()` and `query_actor_erased()`.
    pub static ref REGISTRY: std::sync::Mutex<HashMap<ActorId, WeakErasedAddr>> = Default::default();
}

/// A message wrapper type that enables returning results through the result channel packed inside it.
pub struct WrappedMessage<M: Message> {
    /// The actual message to be sent.
    pub msg: M,

    /// The result channel used to send the result, if used with `send_and_wait()`.
    pub result_channel: Option<oneshot::Sender<Result<M::Result, M::Error>>>,
}
