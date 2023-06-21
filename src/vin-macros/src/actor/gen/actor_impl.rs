use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};

use crate::actor::names::*;
use crate::actor::handles_attr::*;

pub fn form_actor_trait(
    name: &Ident,
    handles_attrs: &Vec<HandlesAttr>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let (fulls, shorts) = form_message_names(&handles_attrs);
    let context_name = form_context_struct_name(name);
    let hidden_name = form_hidden_struct_name(name);

    let (sem_inits, msg_recv_impls): (Vec<_>, Vec<_>) = handles_attrs.iter().zip(fulls.iter().zip(shorts.iter())).map(|(attr, (full, short))| {
        let max = attr.max_messages_at_once;
        let sem_name = quote::format_ident!("sem_{}", short);
        let wrap_name = quote::format_ident!("wrap_fn_{}", short);

        (
            quote! {
                let #sem_name = ::std::sync::Arc::new(::vin::tokio::sync::Semaphore::new(#max));
                let #wrap_name = || {
                    let sem = ::std::sync::Arc::clone(&#sem_name);
                    let actor = ::std::sync::Arc::clone(&actor);
                    async move {
                        ::vin::futures::future::join(sem.acquire_owned(), actor.vin_hidden.mailbox.#short.1.recv()).await
                    }
                };
            },
            quote! {
                (permit, msg) = #wrap_name() => {
                    let permit = permit.expect("vin | semaphore shouldn't be closed while the actor is running");
                    let msg = msg.expect("vin | channel should never be closed while the actor is running");
                    ::vin::log::trace!("vin.{} | actor handling '{}'", id, stringify!(#full));

                    let id = id.clone();
                    let actor = ::std::sync::Arc::clone(&actor);
                    handler_join_set.spawn(async move {
                        let _permit = permit;
                        let res = <Self as ::vin::vin_core::Handler<#full>>::handle(actor.borrow(), msg.msg).await;
                        match &res {
                            Ok(res) => ::vin::log::trace!("vin.{} | actor handling of '{}' completed with: {:?}", id, stringify!(#full), res),
                            Err(err) => ::vin::log::trace!("vin.{} | actor handling of '{}' failed with error: {:#?}", id, stringify!(#full), err),
                        }

                        if let Some(result_channel) = msg.result_channel {
                            result_channel.send(res).unwrap(); // shouldn't ever fail
                        }
                    });
                }
            }
        )
    }).unzip();

    quote! {
        #[::vin::async_trait::async_trait]
        impl #impl_generics ::vin::vin_core::Actor for #name #ty_generics #where_clause {
            type Context = #context_name #ty_generics;

            async fn ctx(&self) -> ::vin::tokio::sync::RwLockReadGuard<Self::Context> {
                self.vin_ctx.read().await
            }

            async fn ctx_mut(&self) -> ::vin::tokio::sync::RwLockWriteGuard<Self::Context> {
                self.vin_ctx.write().await
            }

            fn start<Id: Into<::vin::vin_core::ActorId> + Send>(id: Id, ctx: Self::Context) -> ::std::result::Result<::vin::vin_core::StrongAddr<Self>, ::vin::vin_core::ActorStartError> {
                let id = id.into();

                // Add actor to global registry
                let ret = {
                    let mut reg = ::vin::vin_core::detail::REGISTRY.lock().expect("actor registry should never be poisoned");
                    if reg.contains_key(&id) {
                        return Err(::vin::vin_core::ActorStartError);
                    }

					let ret = ::std::sync::Arc::new(Self {
						vin_ctx: ::vin::tokio::sync::RwLock::new(ctx),
						vin_hidden: #hidden_name {
							id: id.clone(),
                            ..<#hidden_name>::default()
						},
					});

                    ::vin::vin_core::add_actor();
                    reg.insert(id.clone(), ::std::sync::Arc::downgrade(&ret) as ::vin::vin_core::WeakErasedAddr);
                    ret
                };

                let actor = ::std::sync::Arc::clone(&ret);
                ::vin::tokio::spawn(async move {
                    use ::core::borrow::Borrow;

                    let mut handler_join_set = ::vin::tokio::task::JoinSet::new();
                    let shutdown = ::vin::vin_core::detail::SHUTDOWN_SIGNAL.notified();
                    let close = actor.vin_hidden.close.notified();
                    ::vin::tokio::pin!(shutdown);
                    ::vin::tokio::pin!(close);

                    #(#sem_inits)*

                    ::vin::log::trace!("vin.{} | actor started", id);
                    actor.vin_hidden.state.store(::vin::vin_core::State::Running);
                    <Self as ::vin::vin_core::Hooks>::on_started(actor.borrow()).await;
                    loop {
                        ::vin::tokio::select! {
                            _ = &mut close => {
                                ::vin::log::trace!("vin.{} | actor received close signal", id);
                                break;
                            },
                            _ = &mut shutdown => {
                                ::vin::log::trace!("vin.{} | actor received shutdown signal", id);
                                break;
                            },
                            Some(res) = handler_join_set.join_next() => if let Err(join_err) = res {
                                if let Ok(reason) = join_err.try_into_panic() {
                                    ::vin::log::error!("vin.{} | actor received panic from handler: {:?}", id, reason);
                                }
                            },
                            #(#msg_recv_impls),*
                        };
                    }

                    actor.vin_hidden.close.notify_waiters();

                    // Give some time for the existing handlers to gracefully end
                    actor.vin_hidden.state.store(::vin::vin_core::State::Closing);
                    ::vin::log::trace!("vin.{} | actor is closing...", id);

                    // Either await until completion or just wait for 30 more seconds and then close
                    while let Some(res) = handler_join_set.join_next().await {
                        if let Err(join_err) = res {
                            if let Ok(reason) = join_err.try_into_panic() {
                                ::vin::log::error!("vin.{} | actor received panic from handler: {:?}", id, reason);
                            }
                        }
                    }

                    // After aborting, the join set still needs to be drained since tasks only get cancelled
                    // on an 'await' point
                    handler_join_set.abort_all();
                    while handler_join_set.join_next().await.is_some() {}

                    // Run the lifecycle on_closed hook
                    <Self as ::vin::vin_core::Hooks>::on_closed(actor.borrow()).await;
                    actor.vin_hidden.state.store(::vin::vin_core::State::Closed);
                    ::vin::log::trace!("vin.{} | actor is closed", id);

                    // Remove the actor from the registry
                    {
                        let mut reg = ::vin::vin_core::detail::REGISTRY.lock().expect("actor registry should never be poisoned");
                        reg.remove(&id);
                    }
                    ::vin::vin_core::remove_actor();
                });

                Ok(ret)
            }
        }

        #[::vin::async_trait::async_trait]
        impl #impl_generics ::vin::vin_core::Addr for #name #ty_generics #where_clause {

            async fn send<M: ::vin::vin_core::Message + Send>(&self, msg: M)
                where 
                    Self: ::vin::vin_core::detail::Forwarder<M> + Sized,
            {
                let msg = ::vin::vin_core::detail::WrappedMessage {
                    msg,
                    result_channel: None,
                };
                <Self as ::vin::vin_core::detail::Forwarder<M>>::forward(&self, msg).await;
            }

            async fn send_and_wait<M: ::vin::vin_core::Message>(&self, msg: M) -> ::std::result::Result<M::Result, M::Error>
                where
                    Self: Sized + ::vin::vin_core::detail::Forwarder<M>,
            {
                let (tx, rx) = ::vin::tokio::sync::oneshot::channel();
                let msg = ::vin::vin_core::detail::WrappedMessage {
                    msg,
                    result_channel: Some(tx),
                };
                <Self as ::vin::vin_core::detail::Forwarder<M>>::forward(&self, msg).await;

                rx.await.unwrap()
            }

            fn close(&self) {
                self.vin_hidden.close.notify_waiters();
            }

            fn close_future(&self) -> ::vin::tokio::sync::futures::Notified<'_> {
                self.vin_hidden.close.notified()
            }

            fn state(&self) -> ::vin::vin_core::State {
                self.vin_hidden.state.load()
            }

            fn id(&self) -> ::vin::vin_core::ActorId {
                self.vin_hidden.id.clone()
            }
        }
    }
}
