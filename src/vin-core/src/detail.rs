use std::{sync::atomic::AtomicUsize, collections::HashMap};
use tokio::sync::Notify;
use crate::{ActorId, WeakErasedAddr};
use lazy_static::lazy_static;

pub use super::{Forwarder, WrappedMessage};

/// Global actor shutdown signal.
pub static SHUTDOWN_SIGNAL: Notify = Notify::const_new();

/// Number of actors alive.
pub static ACTORS_ALIVE: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    /// An actor registry that is queriable via `query_actor()` and `query_actor_erased()`.
    pub static ref REGISTRY: std::sync::Mutex<HashMap<ActorId, WeakErasedAddr>> = Default::default();
}
