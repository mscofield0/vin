#[doc(hidden)]
pub use vin_core::{
    anyhow, query_actor, query_actor_erased, send, send_erased, send_to, shutdown,
    wait_for_shutdowns, Actor, ActorId, ActorQueryError, Addr, Handler, LifecycleHook, Message,
    StartError, State, StrongAddr, StrongErasedAddr, WeakAddr, WeakErasedAddr,
};
#[doc(hidden)]
pub use vin_macros::{
    self,
    vin_proc_macros::{actor, handles},
};

#[doc(hidden)] pub use ::vin_macros::async_trait::async_trait;
#[doc(hidden)] pub use ::vin_macros::tokio;
#[doc(hidden)] pub use ::vin_macros::tracing;
