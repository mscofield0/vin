use proc_macro2::{TokenStream as TokenStream2};
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};
use quote::{quote};

use crate::task::names::{
    form_hidden_struct_name,
    form_context_struct_name,
};

pub fn form_task_actor_trait(
    name: &Ident,
    phantom_field_names: &Vec<Ident>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let context_name = form_context_struct_name(name);
    let hidden_name = form_hidden_struct_name(name);

    quote! {
        #[::vin::async_trait::async_trait]
        impl #impl_generics ::vin::vin_core::TaskActor for #name #ty_generics #where_clause {
            async fn start<Id: Into<::vin::vin_core::ActorId> + Send>(id: Id, mut ctx: Self::Context) -> ::vin::vin_core::StrongAddr<Self> {
                ::vin::vin_core::add_actor();
                let id = id.into();
                let ret = ::std::sync::Arc::new(Self {
                    vin_hidden: #hidden_name {
                        id: id.clone(),
                        ..Default::default()
                    },
                    #(#phantom_field_names: ::core::marker::PhantomData),*
                });
                let actor = ::std::sync::Arc::clone(&ret);

                ::vin::tokio::spawn(async move {
                    use ::core::borrow::Borrow;

                    let shutdown = ::vin::vin_core::detail::SHUTDOWN_SIGNAL.notified();
                    let close = actor.vin_hidden.close.notified();
                    ::vin::tokio::pin!(shutdown);
                    ::vin::tokio::pin!(close);

                    ::vin::log::debug!("vin | task actor '{}' started", id);
                    actor.vin_hidden.state.store(::vin::vin_core::State::Running);

                    let mut task_join_set = ::vin::tokio::task::JoinSet::new();
                    {
                        let actor = ::std::sync::Arc::clone(&actor);
                        task_join_set.spawn(async move {
                            <Self as ::vin::vin_core::Task>::task(actor.borrow(), ctx).await
                        });
                    }

                    loop {
                        ::vin::tokio::select! {
                            _ = &mut close => {
                                ::vin::log::debug!("vin | task actor '{}' received close signal", id);
                                break;
                            },
                            _ = &mut shutdown => {
                                ::vin::log::debug!("vin | task actor '{}' received shutdown signal", id);
                                actor.vin_hidden.close.notify_waiters();
                                break;
                            },
                            Some(res) = task_join_set.join_next() => match res {
                                Ok(task_res) => match task_res {
                                    Ok(_) => {
                                        ::vin::log::debug!("vin | task actor '{}' completed gracefully", id);
                                        break;
                                    },
                                    Err(err) => {
                                        ::vin::log::error!("vin | task actor '{}' failed with error: {:#?}", id, err);
                                        break;
                                    }
                                },
                                Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                    ::std::panic::resume_unwind(reason);
                                },
                            },
                        };
                    }

                    actor.vin_hidden.state.store(::vin::vin_core::State::Closing);
                    ::vin::log::debug!("vin | task actor '{}' is closing...", id);

                    if let Some(res) = task_join_set.join_next().await {
                        match res {
                            Ok(task_res) => if let Err(err) = task_res {
                                ::vin::log::error!("vin | task actor '{}' failed with error: {:#?}", id, err);
                            },
                            Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                ::std::panic::resume_unwind(reason);
                            },
                        }
                    }

                    actor.vin_hidden.state.store(::vin::vin_core::State::Closed);
                    ::vin::log::debug!("vin | task actor '{}' is closed", id);
                    ::vin::vin_core::remove_actor();
                });

                ret
            }
        }

        impl #impl_generics ::vin::vin_core::TaskAddr for #name #ty_generics #where_clause {
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

        impl #impl_generics ::vin::vin_core::TaskContextTrait for #name #ty_generics #where_clause {
            type Context = #context_name #ty_generics;
        }
    }
}
