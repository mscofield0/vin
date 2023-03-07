use proc_macro2::TokenStream as TokenStream2;
use syn::Ident;
use quote::quote;

use crate::actor::names::*;
use crate::actor::handles_attr::*;

pub fn form_vin_hidden_struct(
    name: &Ident, 
    handles_attrs: &Vec<HandlesAttribute>
) -> TokenStream2 {
    // ===== Form mailbox
    // Mailbox fields to inject into the actor
    let (msg_names, msg_short_names) = form_message_names(handles_attrs);
    let mailbox_fields = handles_attrs.iter()
        .map(|attr| {
            let type_path = &attr.message_type;
            let short_name = &type_path.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            quote! { 
                #short_name: (
                    ::vin::async_channel::Sender<::vin::vin_core::detail::WrappedMessage<#type_path>>, 
                    ::vin::async_channel::Receiver<::vin::vin_core::detail::WrappedMessage<#type_path>>
                )
            }
        })
        .collect::<Vec<_>>();
    let mailbox_inits = handles_attrs.iter()
        .map(|attr| {
            let short_name = &attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            if let Some(Bounded { size, mode: _ }) = &attr.bounded {
                quote! { let #short_name = ::vin::async_channel::bounded(#size) }
            } else {
                quote! { let #short_name = ::vin::async_channel::unbounded() }
            }
        })
        .collect::<Vec<_>>();

    // Mailbox struct to inject into the actor
    let mailbox_struct_name = form_mailbox_name(name);
    let tx_wrapper_name = quote::format_ident!("VinTxWrapper{}", name);
    let mailbox_struct = quote! {
        struct #mailbox_struct_name {
            erased_mailboxes: ::std::collections::HashMap<::core::any::TypeId, ::vin::vin_core::detail::BoxedErasedTx>,
            #(#mailbox_fields),*
        }

        impl Default for #mailbox_struct_name {
            fn default() -> Self {
                #(#mailbox_inits;)*

                let mut erased_mailboxes = ::std::collections::HashMap::default();
                #(erased_mailboxes.insert(::core::any::TypeId::of::<#msg_names>(), Box::new(#tx_wrapper_name(#msg_short_names.0.clone())) as ::vin::vin_core::detail::BoxedErasedTx);)*

                Self {
                    erased_mailboxes,
                    #(#msg_short_names),*
                }
            }
        }
    };

    let mut erased_tx_stream = TokenStream2::new();
    erased_tx_stream.extend(quote! {
        struct #tx_wrapper_name<M: ::vin::vin_core::Message>(
            ::vin::async_channel::Sender<::vin::vin_core::detail::WrappedMessage<M>>
        );
    });

    handles_attrs.iter()
        .map(|attr| {
            let type_path = &attr.message_type.path;
            let msg = quote! {
                ::vin::vin_core::detail::WrappedMessage {
                    msg,
                    result_channel: None,
                }
            };

            let quoted = if let Some(Bounded { size: _, mode }) = &attr.bounded {
                match mode {
                    BoundedMode::Wait => {
                        quote! {
                            self.0.send(#msg).await.expect("vin | mailbox channel should never be closed during the actor's lifetime");
                        }
                    },
                    BoundedMode::Report => {
                        quote! { 
                            if let Err(err) = self.0.try_send(#msg) {
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
                            let _ = self.0.try_send(#msg);
                        }
                    },
                }
            } else {
                quote! {
                    let _ = self.0.try_send(#msg);
                }
            };

            (type_path, quoted)
        })
        .map(|(msg_name, body)| {
            let tx_wrapper_name = quote::format_ident!("VinTxWrapper{}", name);
            quote! {
                #[::vin::async_trait::async_trait]
                impl ::vin::vin_core::detail::ErasedTx for #tx_wrapper_name<#msg_name> {
                    async fn erased_send(&self, msg: ::vin::vin_core::BoxedMessage<()>) {
                        let msg = msg.into_any().downcast::<#msg_name>().ok();
                        if let Some(msg) = msg {
                            let msg = *msg;
                            #body
                        } else {
                            unreachable!(
                                "vin | If a message is not downcastable to its correct type written \
                                in the 'erased_mailboxes' HashMap, then it's a bug. Please report at: \
                                https://github.com/mscofield0/vin/issues"
                            );
                        }
                    }
                }
            }
        })
        .for_each(|x| erased_tx_stream.extend(x));

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

        #erased_tx_stream
    }
}