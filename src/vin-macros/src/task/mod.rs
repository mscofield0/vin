mod task_actor_impl;
mod hidden_struct;
mod context_struct;
pub mod names;

use task_actor_impl::*;
use hidden_struct::*;
use context_struct::*;
use names::*;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Error};
use quote::{quote};

pub fn task_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    
    let data = match input.data {
        Data::Struct(ref mut data) => data,
        _ => return Error::new(input.ident.span(), "vin only works on structs").into_compile_error().into(),
    };
    
    // Common vars for building the final output
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Creates phantom data for all unused type generics
    let (phantom_field_names, phantom_fields): (Vec<_>, Vec<_>) = generics
        .type_params().map(Some).chain(core::iter::repeat(None))
        .zip(generics.lifetimes().map(Some).chain(core::iter::repeat(None)))
        .take_while(|(ty, lt)| ty.is_some() || lt.is_some())
        .map(|(ty, lt)| {
            match (ty, lt) {
                (Some(ty), Some(lt)) => {
                    let field_name = quote::format_ident!("_phantom_{}_{}", ty.ident.to_string().to_lowercase(), lt.lifetime.ident.to_string().to_lowercase());
                    let ident = &ty.ident;
                    let field = quote! { #field_name: ::core::marker::PhantomData<& #lt #ident> };
                    (field_name, field)
                },
                (Some(ty), None) => {
                    let field_name = quote::format_ident!("_phantom_{}", ty.ident.to_string().to_lowercase());
                    let ident = &ty.ident;
                    let field = quote! { #field_name: ::core::marker::PhantomData<#ident> };
                    (field_name, field)
                },
                (None, Some(lt)) => {
                    let field_name = quote::format_ident!("_phantom_{}", lt.lifetime.ident.to_string().to_lowercase());
                    let field = quote! { #field_name: ::core::marker::PhantomData<& #lt u32> };
                    (field_name, field)
                },
                _ => unreachable!(),
            }
        })
        .unzip();

    // Actor trait impl
    let actor_trait = form_task_actor_trait(name, &phantom_field_names, &impl_generics, &ty_generics, where_clause);

    // Other attributes on the actor struct
    let other_attrs = input.attrs.iter().collect::<Vec<_>>();

    // All handled message names
    let vin_hidden_struct = form_vin_hidden_struct(name);

    // Thread-safe, wrapped actor context
    let vin_context_struct = form_vin_context_struct(name, &data.fields, &other_attrs, &generics);
    
    // Modify struct fields
    let hidden_struct_name = form_hidden_struct_name(name);
    
    // Remove all other attributes since it's likely not what the user wanted (they are reapplied onto the context struct)
    input.attrs.clear();
    let attrs = &input.attrs;

    let bruh = quote! {
        #(#attrs)*
        pub struct #name #generics {
            vin_hidden: #hidden_struct_name,
            #(#phantom_fields),*
        }

        #vin_context_struct

        #vin_hidden_struct

        #actor_trait
    }.into();

    bruh
}
