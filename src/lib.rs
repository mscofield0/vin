pub use vin_core::{Actor, Forwarder, Handler, Message, Addr, shutdown, wait_for_shutdowns, State};
pub use vin_macros::{
    self,
    vin_proc_macros::{actor, handles}
};

pub mod prelude {
    pub use super::*;
    #[doc(hidden)] pub use ::async_trait::async_trait;
}