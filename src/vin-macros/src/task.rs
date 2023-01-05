use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use syn::{parse_macro_input, Ident, DeriveInput, ImplGenerics, TypeGenerics, WhereClause};
use quote::{quote};

pub fn task_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Common vars for building the final output
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Actor trait impl
    let actor_trait = form_task_actor_trait(name, &impl_generics, &ty_generics, where_clause);

    let bruh = quote! {
        #input

        #actor_trait
    }.into();

    bruh
}

fn form_task_actor_trait(
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    quote! {
        #[::vin::vin_core::async_trait::async_trait]
        impl #impl_generics ::vin::vin_core::TaskActor for #name #ty_generics #where_clause {
            async fn start<Id: Into<::vin::ActorId> + Send>(mut self, id: Id) -> ::std::sync::Arc<::vin::vin_core::TaskCloseHandle> {
                ::vin::vin_core::add_actor();
                let id = id.into();
                let close_handle = ::std::sync::Arc::new(TaskCloseHandle::default());
                let ret = ::std::sync::Arc::clone(&close_handle);

                ::vin::tokio::spawn(async move {
                    let shutdown = ::vin::vin_core::SHUTDOWN_SIGNAL.notified();
                    let close = close_handle.close_future();
                    ::vin::tokio::pin!(shutdown);
                    ::vin::tokio::pin!(close);

                    ::vin::tracing::debug!("task actor '{}' started", id);
                    let mut task_join_set = ::vin::tokio::task::JoinSet::new();
                    task_join_set.spawn(<Self as ::vin::vin_core::Task>::task(self));

                    loop {
                        ::vin::tokio::select! {
                            _ = &mut shutdown => {
                                ::vin::tracing::debug!("task actor '{}' received shutdown signal", id);
                                break;
                            },
                            _ = &mut close => {
                                ::vin::tracing::debug!("task actor '{}' received close signal", id);
                                break;
                            },
                            Some(res) = task_join_set.join_next() => match res {
                                Ok(task_res) => match task_res {
                                    Ok(_) => {
                                        ::vin::tracing::debug!("task actor '{}' completed gracefully", id);
                                        break;
                                    },
                                    Err(err) => {
                                        ::vin::tracing::error!("task actor '{}' failed with error: {:#?}", id, err);
                                        break;
                                    }
                                },
                                Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                    ::std::panic::resume_unwind(reason);
                                },
                            },
                        };
                    }

                    // After aborting, the join set still needs to be drained since tasks only get cancelled
                    // on an 'await' point
                    task_join_set.abort_all();
                    while task_join_set.join_next().await.is_some() {}

                    ::vin::vin_core::remove_actor();
                });

                ret
            }
        }
    }
}
