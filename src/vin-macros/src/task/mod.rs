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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Actor trait impl
    let actor_trait = form_task_actor_trait(name, &impl_generics, &ty_generics, where_clause);

    // Other attributes on the actor struct
    let other_attrs = input.attrs.iter().collect::<Vec<_>>();

    // All handled message names
    let vin_hidden_struct = form_vin_hidden_struct(name);

    // Thread-safe, wrapped actor context
    let vin_context_struct = form_vin_context_struct(name, &data.fields, &other_attrs);
    
    // Modify struct fields
    let hidden_struct_name = form_hidden_struct_name(name);
    
    // Remove all other attributes since it's likely not what the user wanted (they are reapplied onto the context struct)
    input.attrs.clear();
    let attrs = &input.attrs;

    let bruh = quote! {
        #(#attrs)*
        pub struct #name {
            vin_hidden: #hidden_struct_name,
        }

        #vin_context_struct

        #vin_hidden_struct

        #actor_trait
    }.into();

    bruh
}
