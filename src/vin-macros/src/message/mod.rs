mod attr;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use syn::{parse_macro_input, Ident, DeriveInput, ImplGenerics, TypeGenerics, WhereClause};
use quote::{quote};
use attr::*;

pub fn message_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Common vars for building the final output
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let attr = match syn::parse::<Attr>(args) {
		Ok(attr) => attr,
		Err(err) => return err.to_compile_error().into(),
	};

    // Message trait impl
    let message_trait = form_message_trait_impl(attr, name, &impl_generics, &ty_generics, where_clause);

    quote! {
        #input

        #message_trait
    }.into()
}

fn form_message_trait_impl(
	attr: Attr,
    name: &Ident,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
	where_clause: Option<&WhereClause>,
) -> TokenStream2 {
	let result = attr.result.map(|x| { quote! { #x } }).unwrap_or(quote! { () });
	let error = attr.error.map(|x| { quote! { #x } }).unwrap_or(quote! { () });

	quote! {
		impl #impl_generics ::vin::vin_core::Message for #name #ty_generics #where_clause {
			type Result = #result;
			type Error = #error;
        }
	}
}