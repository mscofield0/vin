use itertools::Itertools;
use syn::{Ident, TypePath, Attribute, PathSegment};
use super::handles_attr::HandlesAttr;

pub fn form_mailbox_name(name: &Ident) -> Ident {
    quote::format_ident!("VinMailbox{}", name)
}

pub fn form_hidden_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinHidden{}", name)
}

pub fn form_context_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinContext{}", name)
}

pub fn form_message_names(handles_attrs: &Vec<HandlesAttr>) -> (Vec<TypePath>, Vec<Ident>) {
    let msg_names = handles_attrs.iter()
        .map(|attr| attr.msg_type.clone())
        .collect::<Vec<_>>();

    let msg_short_names = handles_attrs.iter()
        .map(|attr| {
            let ident_name = attr.msg_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            quote::format_ident!("{}", ident_name)
        })
        .collect::<Vec<_>>();

    (msg_names, msg_short_names)
}

pub fn check_if_attr_from_vin(attr: &Attribute, item: &str) -> bool {
    attr.path.segments.iter().contains(&syn::parse_str::<PathSegment>("vin").unwrap()) && 
    attr.path.segments.iter().contains(&syn::parse_str::<PathSegment>(item).unwrap())
}