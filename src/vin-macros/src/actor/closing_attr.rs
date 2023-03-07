use syn::{Ident, parse::Parse, Error};

pub enum ClosingStrategy {
	Awaiting,
	NoAwaiting,
}

impl Parse for ClosingStrategy {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.is_empty() {
            return Ok(Self::Awaiting);
        }

		let ident = input.parse::<Ident>()?;
		let arg = match ident.to_string().as_str() {
			"awaiting" => {
				Self::Awaiting
			},
			"no_awaiting" => {
				Self::NoAwaiting
			},
			_ => return Err(Error::new(ident.span(), "unknown argument name; valid names are: 'awaiting', 'no_awaiting'")),
		};

		Ok(arg)
	}
}