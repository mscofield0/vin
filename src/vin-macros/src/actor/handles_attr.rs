use syn::{Ident, parse::Parse, Error, LitInt, TypePath, Token, token};

pub struct HandlesAttr {
    pub msg_type: TypePath,
    pub max_messages_at_once: usize,
}

impl Parse for HandlesAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let msg_type = input.parse::<TypePath>()?;

        if input.is_empty() { return Ok(HandlesAttr { msg_type, max_messages_at_once: 100_000 }) }

        input.parse::<token::Comma>()?;

        let arg = input.parse::<Ident>()?;
        let max_messages_at_once = match arg.to_string().as_str() {
            "max" => {
                input.parse::<Token![=]>()?;
                let size = input.parse::<LitInt>()?.base10_parse::<usize>()?;
                size
            },
            _ => return Err(Error::new(arg.span(), "unknown argument name; valid arguments are: 'max'")),
        };

        Ok(HandlesAttr {
            msg_type,
            max_messages_at_once,
        })
    }
}