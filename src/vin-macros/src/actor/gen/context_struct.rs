use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, Fields, Attribute, Generics};
use quote::quote;

use crate::actor::names::*;

pub fn form_vin_context_struct(
    name: &Ident, 
    fields: &Fields, 
    other_attrs: &Vec<&Attribute>,
    generics: &Generics,
) -> TokenStream2 {
    let context_struct_name = form_context_struct_name(name);

    match fields {
        Fields::Named(fields) => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #generics #fields
        },
        Fields::Unnamed(fields) => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #generics #fields;
        },
        Fields::Unit => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #generics;
        }
    }
}