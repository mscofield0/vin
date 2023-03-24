use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Generics, ImplGenerics, TypeGenerics, WhereClause};

use crate::actor::names::*;
use crate::actor::handles_attr::*;
use crate::actor::closing_attr::*;

pub fn form_actor_trait(
    closing_strategy: ClosingStrategy,
    name: &Ident,
    handles_attrs: &Vec<HandlesAttribute>,
    generics: &Generics,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let (msg_names, msg_short_names) = form_message_names(&handles_attrs);
    let context_name = form_context_struct_name(name);
    let hidden_name = form_hidden_struct_name(name);

    let closing_strategy = match closing_strategy {
        ClosingStrategy::Awaiting => quote! {
            while let Some(res) = handler_join_set.join_next().await {
                match res {
                    Ok(handler_res) => if let Err(err) = handler_res {
                        ::vin::log::error!("vin | actor '{}' handling of '{}' failed with error: {:#?}", id, err.msg_name(), err);
                    },
                    Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                        ::std::panic::resume_unwind(reason);
                    },
                }
            }
        },
        ClosingStrategy::NoAwaiting => quote! {
            use ::core::time::Duration;
            let _ = ::vin::tokio::time::timeout(Duration::from_secs(1), async {
                while let Some(res) = handler_join_set.join_next().await {
                    match res {
                        Ok(handler_res) => if let Err(err) = handler_res {
                            ::vin::log::error!("vin | actor '{}' handling of '{}' failed with error: {:#?}", id, err.msg_name(), err);
                        },
                        Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                            ::std::panic::resume_unwind(reason);
                        },
                    }
                }
            }).await;
        },
    };

    let (sem_inits, msg_recv_impls): (Vec<_>, Vec<_>) = handles_attrs
        .iter()
        .zip(msg_short_names.iter().zip(msg_names.iter()))
        .map(|(attr, (short_name, long_name))| {
            let sem_name = quote::format_ident!("sem_{}", short_name);
            let wrap_name = quote::format_ident!("wrap_fn_{}", short_name);
            let (init, usage) = match &attr.bounded {
                Some(bounded) => {
                    match bounded.mode {
                        BoundedMode::Wait => {
                            let size = bounded.size;
                            (
                                quote! {
                                    let #sem_name = ::std::sync::Arc::new(::vin::tokio::sync::Semaphore::new(#size));
                                    let #wrap_name = || {
                                        let sem = ::std::sync::Arc::clone(&#sem_name);
                                        let actor = ::std::sync::Arc::clone(&actor);
                                        async move { ::vin::tokio::join!(sem.acquire_owned(), actor.vin_hidden.mailbox.#short_name.1.recv()) }
                                    };
                                },
                                quote! {
                                    (permit, msg) = #wrap_name() => {
                                        let permit = permit.expect("vin | semaphore shouldn't be closed while the actor is running");
                                        let msg = msg.expect("vin | channel should never be closed while the actor is running");
                                        ::vin::log::debug!("vin | actor '{}' handling '{}'", id, stringify!(#long_name));
        
                                        let actor = ::std::sync::Arc::clone(&actor);
                                        handler_join_set.spawn(async move {
                                            let _permit = permit;
                                            if let Some(result_channel) = msg.result_channel {
                                                let res = <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await;
                                                result_channel.send(res).unwrap(); // shouldn't ever fail
                                                Ok(())
                                            } else {
                                                match <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await {
                                                    Ok(_) => Ok(()),
                                                    Err(err) => Err(::vin::vin_core::detail::HandlerError::new(stringify!(#long_name), err)),
                                                }
                                            }
                                        });
                                    }
                                }
                            )
                        },
                        BoundedMode::Silent => (
                            quote! {},
                            quote! {
                                msg = actor.vin_hidden.mailbox.#short_name.1.recv() => {
                                    let msg = msg.expect("vin | channel should never be closed while the actor is running");
                                    ::vin::log::debug!("vin | actor '{}' handling '{}'", id, stringify!(#long_name));
    
                                    let actor = ::std::sync::Arc::clone(&actor);
                                    handler_join_set.spawn(async move {
                                        if let Some(result_channel) = msg.result_channel {
                                            let res = <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await;
                                            result_channel.send(res).unwrap(); // shouldn't ever fail
                                            Ok(())
                                        } else {
                                            match <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await {
                                                Ok(_) => Ok(()),
                                                Err(err) => Err(::vin::vin_core::detail::HandlerError::new(stringify!(#long_name), err)),
                                            }
                                        }
                                    });
                                }
                            },
                        ),
                        BoundedMode::Report => (
                            quote! {},
                            quote! {
                                msg = actor.vin_hidden.mailbox.#short_name.1.recv() => {
                                    let msg = msg.expect("vin | channel should never be closed while the actor is running");
                                    ::vin::log::debug!("vin | actor '{}' handling '{}'", id, stringify!(#long_name));
    
                                    let actor = ::std::sync::Arc::clone(&actor);
                                    handler_join_set.spawn(async move {
                                        if let Some(result_channel) = msg.result_channel {
                                            let res = <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await;
                                            result_channel.send(res).unwrap(); // shouldn't ever fail
                                            Ok(())
                                        } else {
                                            match <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await {
                                                Ok(_) => Ok(()),
                                                Err(err) => Err(::vin::vin_core::detail::HandlerError::new(stringify!(#long_name), err)),
                                            }
                                        }
                                    });
                                }
                            },
                        ),
                    }
                    
                },
                None => (
                    quote! {
                        let #sem_name = ::std::sync::Arc::new(::vin::tokio::sync::Semaphore::new(100_000));
                        let #wrap_name = || {
                            let sem = ::std::sync::Arc::clone(&#sem_name);
                            let actor = ::std::sync::Arc::clone(&actor);
                            async move { ::vin::tokio::join!(sem.acquire_owned(), actor.vin_hidden.mailbox.#short_name.1.recv()) }
                        };
                    },
                    quote! {
                        (permit, msg) = #wrap_name() => {
                            let permit = permit.expect("vin | semaphore shouldn't be closed while the actor is running");
                            let msg = msg.expect("vin | channel should never be closed while the actor is running");
                            ::vin::log::debug!("vin | actor '{}' handling '{}'", id, stringify!(#long_name));

                            let actor = ::std::sync::Arc::clone(&actor);
                            handler_join_set.spawn(async move {
                                let _permit = permit;
                                if let Some(result_channel) = msg.result_channel {
                                    let res = <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await;
                                    result_channel.send(res).unwrap(); // shouldn't ever fail
                                    Ok(())
                                } else {
                                    match <Self as ::vin::vin_core::Handler<#long_name>>::handle(actor.borrow(), msg.msg).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => Err(::vin::vin_core::detail::HandlerError::new(stringify!(#long_name), err)),
                                    }
                                }
                            });
                        }
                    }
                ),
            };

            (init, usage)
        })
        .unzip();

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

            async fn start<Id: Into<::vin::vin_core::ActorId> + Send>(id: Id, ctx: Self::Context) -> ::vin::anyhow::Result<::vin::vin_core::StrongAddr<Self>> {
                let id = id.into();

                // Add actor to global registry
                let ret = {
                    let mut reg = ::vin::vin_core::detail::REGISTRY.lock().await;
                    if reg.contains_key(&id) {
                        return Err(::vin::anyhow::anyhow!("id '{}' already taken", id));
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

                    let mut handler_join_set = ::vin::tokio::task::JoinSet::<Result<(), ::vin::vin_core::detail::HandlerError>>::new();
                    let shutdown = ::vin::vin_core::detail::SHUTDOWN_SIGNAL.notified();
                    let close = actor.vin_hidden.close.notified();
                    ::vin::tokio::pin!(shutdown);
                    ::vin::tokio::pin!(close);

                    #(#sem_inits)*

                    ::vin::log::debug!("vin | actor '{}' started", id);
                    actor.vin_hidden.state.store(::vin::vin_core::State::Running);
                    <Self as ::vin::vin_core::Hooks>::on_started(actor.borrow()).await;
                    loop {
                        ::vin::tokio::select! {
                            _ = &mut close => {
                                ::vin::log::debug!("vin | actor '{}' received close signal", id);
                                break;
                            },
                            _ = &mut shutdown => {
                                ::vin::log::debug!("vin | actor '{}' received shutdown signal", id);
                                actor.vin_hidden.close.notify_waiters();
                                break;
                            },
                            Some(res) = handler_join_set.join_next() => match res {
                                Ok(handler_res) => if let Err(err) = handler_res {
                                    ::vin::log::error!("vin | actor '{}' handling of '{}' failed with error: {:#?}", id, err.msg_name(), err);
                                    let actor = ::std::sync::Arc::clone(&actor);
                                    handler_join_set.spawn(async move {
                                        <Self as ::vin::vin_core::Hooks>::on_error(actor.borrow(), err.inner).await;
                                        Ok(())
                                    });
                                },
                                Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                    ::std::panic::resume_unwind(reason);
                                },
                            },
                            #(#msg_recv_impls),*
                        };
                    }

                    // Give some time for the existing handlers to gracefully end
                    actor.vin_hidden.state.store(::vin::vin_core::State::Closing);
                    ::vin::log::debug!("vin | actor '{}' is closing...", id);

                    // Either await until completion or just wait for 1 more second and then close
                    #closing_strategy

                    // After aborting, the join set still needs to be drained since tasks only get cancelled
                    // on an 'await' point
                    handler_join_set.abort_all();
                    while handler_join_set.join_next().await.is_some() {}

                    // Run the lifecycle on_closed hook
                    <Self as ::vin::vin_core::Hooks>::on_closed(actor.borrow()).await;
                    actor.vin_hidden.state.store(::vin::vin_core::State::Closed);
                    ::vin::log::debug!("vin | actor '{}' is closed", id);

                    // Remove the actor from the registry
                    {
                        let mut reg = ::vin::vin_core::detail::REGISTRY.lock().await;
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
                    M::Result: Send,
            {
                let msg = ::vin::vin_core::detail::WrappedMessage {
                    msg,
                    result_channel: None,
                };
                <Self as ::vin::vin_core::detail::Forwarder<M>>::forward(&self, msg).await;
            }

            async fn send_and_wait<M: ::vin::vin_core::Message>(&self, msg: M) -> ::vin::anyhow::Result<M::Result>
                where
                    Self: Sized + ::vin::vin_core::detail::Forwarder<M>,
                    M::Result: Send,
            {
                let (tx, rx) = ::vin::tokio::sync::oneshot::channel();
                let msg = ::vin::vin_core::detail::WrappedMessage {
                    msg,
                    result_channel: Some(tx),
                };
                <Self as ::vin::vin_core::detail::Forwarder<M>>::forward(&self, msg).await;

                if let Ok(val) = rx.await {
                    val
                } else {
                    Err(anyhow::anyhow!("mailbox is full"))
                }
            }

            async fn erased_send(&self, msg: ::vin::vin_core::BoxedMessage<()>) {
                let tx = self.vin_hidden.mailbox.erased_mailboxes
                    .get(&msg.as_ref().as_any().type_id())
                    .expect("vin | sent a message that the actor does not handle");
                tx.erased_send(msg).await;
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

            fn id(&self) -> String {
                self.vin_hidden.id.to_string()
            }
        }
    }
}
