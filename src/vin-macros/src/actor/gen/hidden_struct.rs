use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;
use quote::quote;

use crate::actor::names::*;
use crate::actor::handles_attr::*;

pub fn form_vin_hidden_struct(
    name: &Ident, 
    handles_attrs: &Vec<HandlesAttr>
) -> TokenStream2 {
    // ===== Form mailbox
    // Mailbox fields to inject into the actor
    let (fulls, shorts) = form_message_names(handles_attrs);
    let mailbox_fields = fulls.iter().zip(shorts.iter()).map(|(full, short)| {
        quote! { 
            #short: (
                ::vin::async_channel::Sender<::vin::vin_core::detail::WrappedMessage<#full>>, 
                ::vin::async_channel::Receiver<::vin::vin_core::detail::WrappedMessage<#full>>
            )
        }
    }).collect::<Vec<_>>();

    let mailbox_inits = handles_attrs.iter().zip(shorts.iter()).map(|(attr, short)| {
        let max = attr.max_messages_at_once;
        quote! { let #short = ::vin::async_channel::bounded(#max) }
    }).collect::<Vec<_>>();

    // Mailbox struct to inject into the actor
    let mailbox_struct_name = form_mailbox_name(name);
    let mailbox_struct = quote! {
        struct #mailbox_struct_name {
            #(#mailbox_fields),*
        }

        impl Default for #mailbox_struct_name {
            fn default() -> Self {
                #(#mailbox_inits;)*

                Self {
                    #(#shorts),*
                }
            }
        }
    };

    // ===== Form final output
    let hidden_struct_name = form_hidden_struct_name(name);
    quote! {
        #mailbox_struct

        struct #hidden_struct_name {
            mailbox: #mailbox_struct_name,
            state: ::vin::crossbeam::atomic::AtomicCell<::vin::vin_core::State>,
            close: ::vin::tokio::sync::Notify,
            id: ::vin::vin_core::ActorId,
        }

        impl Default for #hidden_struct_name {
            fn default() -> Self {
                Self {
                    mailbox: Default::default(),
                    state: Default::default(),
                    close: Default::default(),
                    id: "none".into(),
                }
            }
        }
    }
}