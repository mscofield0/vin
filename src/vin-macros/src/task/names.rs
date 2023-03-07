use syn::Ident;

pub fn form_hidden_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinHidden{}", name)
}

pub fn form_context_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinContext{}", name)
}