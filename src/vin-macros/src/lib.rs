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
/// ## Additional arguments
/// Currently there is only one additional argument and it's 'bounded'.
/// 
/// ### `bounded`
/// The `bounded` argument allows you to set an upper limit to the amount of messages a mailbox 
/// can take in. It also allows you to set a strategy for handling a full mailbox. Current 
/// available strategies are: 'wait' (awaits until the mailbox is available), 'report' (reports a failure) 
/// and 'silent' (silently discards the message).
/// 
/// ## Example
/// ```ignore
/// #[vin::actor]
/// #[vin::handles(MyMsg, bounded(size = 1024, report))]
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
///     async fn task(&mut self) -> anyhow::Result<()> {
///         loop {
///             match self.ws.recv().await {
///                 Ok(msg) => {}, // e.g. send the message to some other actor
///                 Err(err) => return Err(err), // and maybe even notify some "manager" actor
///             }
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
#[proc_macro_attribute]
pub fn message(args: TokenStream, input: TokenStream) -> TokenStream {
    message_impl(args, input)
}