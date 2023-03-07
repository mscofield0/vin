use proc_macro2::{TokenStream as TokenStream2};
use syn::{Ident};
use quote::{quote};
use super::names::*;

pub fn form_vin_hidden_struct(
    name: &Ident, 
) -> TokenStream2 {
    let hidden_struct_name = form_hidden_struct_name(name);
    quote! {
        struct #hidden_struct_name {
            state: ::vin::crossbeam::atomic::AtomicCell<::vin::vin_core::State>,
            close: ::vin::tokio::sync::Notify,
            id: ::vin::vin_core::ActorId,
        }

        impl Default for #hidden_struct_name {
            fn default() -> Self {
                Self {
                    state: Default::default(),
                    close: Default::default(),
                    id: "none".into(),
                }
            }
        }
    }
}