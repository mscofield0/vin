use proc_macro2::TokenStream as TokenStream2;
use syn::{Ident, Fields, Attribute, TypeGenerics, WhereClause};
use quote::quote;

use super::names::*;

pub fn form_vin_context_struct(
    name: &Ident, 
    fields: &Fields, 
    other_attrs: &Vec<&Attribute>,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let context_struct_name = form_context_struct_name(name);

    match fields {
        Fields::Named(fields) => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #ty_generics #where_clause #fields
        },
        Fields::Unnamed(fields) => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #ty_generics #where_clause #fields;
        },
        Fields::Unit => quote! {
            #(#other_attrs)*
            pub struct #context_struct_name #ty_generics #where_clause;
        }
    }
}