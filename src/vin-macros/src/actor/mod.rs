use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error, Data};
use quote::quote;

mod handles_attr;
mod closing_attr;
mod gen;
mod names;

use handles_attr::*;
use gen::*;
use names::*;
use closing_attr::*;


pub fn actor_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let data = match input.data {
        Data::Struct(ref mut data) => data,
        _ => return Error::new(input.ident.span(), "vin only works on structs").into_compile_error().into(),
    };
    
    // Common vars for building the final output
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
 
	let closing_strategy = match syn::parse::<ClosingStrategy>(args) {
		Ok(closing_strategy) => closing_strategy,
		Err(err) => return err.to_compile_error().into(),
	};
    
    // Parsed handles attributes
    let handles_attrs = input.attrs.iter()
        .filter(|attr| check_if_attr_from_vin(&attr, "handles"))
        .map(|attr| attr.parse_args::<HandlesAttribute>())
        .collect::<Result<Vec<_>, _>>();

    let handles_attrs = match handles_attrs {
        Ok(attrs) => attrs,
        Err(err) => return err.to_compile_error().into(),
    };

    // Other attributes on the actor struct
    let other_attrs = input.attrs.iter()
        .filter(|attr| !check_if_attr_from_vin(&attr, "handles"))
        .collect::<Vec<_>>();

    // All handled message names
    let vin_hidden_struct = form_vin_hidden_struct(name, &handles_attrs);

    // Thread-safe, wrapped actor context
    let vin_context_struct = form_vin_context_struct(name, &data.fields, &other_attrs);

    // Forwarder trait impls
    let forwarder_traits = form_forwarder_impls(name, &handles_attrs, &impl_generics, &ty_generics, where_clause);

    // Actor trait impl
    let actor_trait = form_actor_trait(closing_strategy, name, &handles_attrs, &impl_generics, &ty_generics, where_clause);

    // Modify struct fields
    let hidden_struct_name = form_hidden_struct_name(name);
    let context_struct_name = form_context_struct_name(name);

    // Remove all other attributes since it's likely not what the user wanted (they are reapplied onto the context struct)
    input.attrs.clear();
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        pub struct #name {
            vin_ctx: ::vin::tokio::sync::RwLock<#context_struct_name>,
            vin_hidden: #hidden_struct_name,
        }

        #vin_context_struct

        #vin_hidden_struct

        #forwarder_traits
        
        #actor_trait
    }.into()
}