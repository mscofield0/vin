use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};
use quote::quote;

use crate::actor::handles_attr::*;

pub fn form_forwarder_impls(
    name: &Ident,
    handles_attrs: &Vec<HandlesAttribute>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let streams = handles_attrs.iter()
        .map(|attr| {
            let type_path = &attr.message_type.path;
            let short_name = attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            let quoted = if let Some(Bounded { size: _, mode }) = &attr.bounded {
                match mode {
                    BoundedMode::Wait => {
                        quote! {
                            self.vin_hidden.mailbox.#short_name.0
                                .send(msg)
                                .await
                                .expect("vin | mailbox channel should never be closed during the actor's lifetime");
                        }
                    },
                    BoundedMode::Report => {
                        quote! { 
                            if let Err(err) = self.vin_hidden.mailbox.#short_name.0.try_send(msg) {
                                match err {
                                    ::vin::async_channel::TrySendError::Full(_) => {
                                        ::vin::log::error!("vin | mailbox for {:?} is full", stringify!(#type_path));
                                    },
                                    ::vin::async_channel::TrySendError::Closed(_) => {
                                        unreachable!("vin | mailbox channel should never be closed during the actor's lifetime");
                                    },
                                }
                            }
                        }
                    },
                    BoundedMode::Silent => {
                        quote! {
                            let _ = self.vin_hidden.mailbox.#short_name.0.try_send(msg);
                        }
                    },
                }
            } else {
                quote! {
                    let _ = self.vin_hidden.mailbox.#short_name.0.try_send(msg);
                }
            };

            (attr.message_type.clone(), quoted)
        })
        .map(|(msg_name, body)| {
            quote! {
                #[::vin::async_trait::async_trait]
                impl #impl_generics ::vin::vin_core::detail::Forwarder<#msg_name> for #name #ty_generics #where_clause {
                    async fn forward(&self, msg: ::vin::vin_core::detail::WrappedMessage<#msg_name>) {
                        #body
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let mut stream = TokenStream2::new();
    streams.into_iter().for_each(|x| stream.extend(x) );

    stream
}
