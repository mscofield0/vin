mod result_attr;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use syn::{parse_macro_input, Ident, DeriveInput, ImplGenerics, TypeGenerics, WhereClause};
use quote::{quote};
use result_attr::*;

pub fn message_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Common vars for building the final output
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let result_arg = match syn::parse::<ResultArg>(args) {
		Ok(result_arg) => result_arg,
		Err(err) => return err.to_compile_error().into(),
	};

    // Message trait impl
    let message_trait = form_message_trait_impl(result_arg, name, &impl_generics, &ty_generics, where_clause);

    quote! {
        #input

        #message_trait
    }.into()
}

fn form_message_trait_impl(
	result_arg: ResultArg,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
	where_clause: Option<&WhereClause>,
) -> TokenStream2 {
	let result_type = match result_arg {
		ResultArg::NoResult => quote! { () },
		ResultArg::WithResult(result) => quote! { #result },
	};
	
	quote! {
		impl #impl_generics ::vin::vin_core::Message for #name #ty_generics #where_clause {
			type Result = #result_type;
        }
	}
}