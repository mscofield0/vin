use rassert_rs::rassert;
use syn::{Ident, parse::Parse, Error, LitInt, TypePath, Token, parenthesized};

pub struct HandlesAttribute {
    pub message_type: TypePath,
    pub bounded: Option<Bounded>,
}

pub enum BoundedMode {
    Wait,
    Report,
    Silent,
}

pub type BoundSize = usize;

pub struct Bounded {
    pub size: BoundSize,
    pub mode: BoundedMode,
}

pub enum AttrArg {
    Bounded(BoundSize, BoundedMode),
}

impl Parse for AttrArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let attr = match ident.to_string().as_str() {
            "bounded" => {
                let paren_content;
                parenthesized!(paren_content in input);
                let size = paren_content.parse::<Ident>()?;
                rassert!(size == "size", Error::new(size.span(), "unknown attribute argument type; expected: 'size'"));
                paren_content.parse::<Token![=]>()?;
                let size = paren_content.parse::<LitInt>()?.base10_parse::<usize>()?;
                paren_content.parse::<Token![,]>()?;
                let mode = paren_content.parse::<Ident>()?;
                let mode = match mode.to_string().as_str() {
                    "wait" => BoundedMode::Wait,
                    "report" => BoundedMode::Report,
                    "silent" => BoundedMode::Silent,
                    _ => return Err(Error::new(mode.span(), "unknown bounded mode; valid modes are: 'wait', 'report', 'silent'")),
                };

                AttrArg::Bounded(size, mode)
            },
            _ => return Err(Error::new(ident.span(), "unknown argument name; valid names are: 'bounded'")),
        };

        Ok(attr)
    }
}

impl Parse for HandlesAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let message_type = input.parse::<TypePath>()?;
        
        let attr = if !input.is_empty() {
            input.parse::<Token![,]>()?;
            Some(input.parse::<AttrArg>()?)
        } else {
            None
        };

        let bounded = if let Some(AttrArg::Bounded(size, mode)) = attr {
            Some(Bounded { size, mode })
        } else {
            None
        };

        Ok(HandlesAttribute {
            message_type,
            bounded,
        })
    }
}