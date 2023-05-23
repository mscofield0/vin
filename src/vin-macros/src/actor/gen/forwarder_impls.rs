use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, ImplGenerics, TypeGenerics, WhereClause};
use quote::quote;

use crate::actor::{handles_attr::*, names::*};

pub fn form_forwarder_impls(
    name: &Ident,
    handles_attrs: &Vec<HandlesAttr>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let (fulls, shorts) = form_message_names(handles_attrs);

    let mailbox_sends = shorts.iter().map(|short| {
        quote! {
            self.vin_hidden.mailbox.#short.0
                .send(msg)
                .await
                .expect("vin | mailbox channel should never be closed during the actor's lifetime")
        }
    }).collect::<Vec<_>>();

    let forwarder_impls = fulls.iter().zip(mailbox_sends.iter()).map(|(full, send)| {
        quote! {
            #[::vin::async_trait::async_trait]
            impl #impl_generics ::vin::vin_core::detail::Forwarder<#full> for #name #ty_generics #where_clause {
                async fn forward(&self, msg: ::vin::vin_core::detail::WrappedMessage<#full>) {
                    #send
                }
            }
        }
    }).collect::<Vec<_>>();

    quote! {
        #(#forwarder_impls)*
    }
}
