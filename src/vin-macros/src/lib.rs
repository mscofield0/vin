mod actor;
mod task;
mod message;

use actor::actor_impl;
use task::task_impl;
use message::message_impl;

use proc_macro::TokenStream;
use quote::quote;

/// Generates the actor impls and forms necessary fields.
/// 
/// # `vin::handles()` proc macro
/// Specifies which message the actor handles, then generates code to handle the message.
/// 
/// ## Arguments
/// - message type
/// - maximum messages to handle concurrently (optional)
/// 
/// # Example
/// ```ignore
/// #[vin::actor]
/// #[vin::handles(MyMsg)]
/// struct MyActor;
/// 
/// /* or */
/// 
/// #[vin::actor]
/// #[vin::handles(MyMsg, max = 1024)]
/// struct MyActor;
/// ```
#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
    actor_impl(args, input)
}

/// A noop macro attribute used to specify messages to handle. Check [`actor`] for more information.
#[proc_macro_attribute]
pub fn handles(_args: TokenStream, _input: TokenStream) -> TokenStream {
    quote! {}.into()
}

/// Generates a [`vin`]-managed [`tokio`] task. It's a specialized actor with no handler that runs some 
/// piece of code until it's completed or [`vin`] has shutdown.
/// 
/// # Example
/// ```no_run
/// # use vin::*;
/// # struct WebSocket;
/// 
/// # impl WebSocket {
/// #     async fn recv(&mut self) -> Vec<u8> {
/// #         Vec::new()
/// #     }
/// # }
/// 
/// #[vin::task]
/// struct WebsocketSession {
///     ws: WebSocket,
/// }
/// 
/// #[async_trait]
/// impl TaskActor for WebsocketSession {
///     async fn task(&self, ctx: Self::Context) -> anyhow::Result<()> {
///         loop {
///             let msg = self.ws.recv().await?;
///             /* do something */
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn task(args: TokenStream, input: TokenStream) -> TokenStream {
    task_impl(args, input)
}

/// Implements the [`Message`] trait for a type. Enables specifying the result type easily.
/// 
/// Shorthand for:
/// ```ignore
/// struct MyMsg;
/// 
/// impl Message for MyMsg {
///     type Result = u32;
/// }
/// ```
/// 
/// # Example
/// ```ignore
/// #[vin::message] // No result type (result = ())
/// struct MyMsg;
/// ```
/// or
/// ```ignore
/// #[vin::message(result = u32)]
/// struct MyMsg;
/// ```
/// or
/// ```ignore
/// #[vin::message(result = u32, error = MyError)]
/// struct MyMsg;
/// ```
#[proc_macro_attribute]
pub fn message(args: TokenStream, input: TokenStream) -> TokenStream {
    message_impl(args, input)
}