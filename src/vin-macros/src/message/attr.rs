use syn::{Ident, parse::Parse, Error, Token, Type, token};

pub struct Attr {
	pub result: Option<Type>,
	pub error: Option<Type>,
}

impl Parse for Attr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.is_empty() { return Ok(Self { result: None, error: None }) }

		let mut result = None;
		let mut error = None;

		let mut read_arg = || {
			let ident = input.parse::<Ident>()?;
			input.parse::<Token![=]>()?;
			let value = input.parse::<Type>()?;
			match ident.to_string().as_str() {
				"result" => if result.is_none() {
					result = Some(value);
				} else {
					return Err(Error::new(ident.span(), "duplicate argument name"));
				},
				"error" => if error.is_none() {
					error = Some(value)
				} else {
					return Err(Error::new(ident.span(), "duplicate argument name"));
				},
				_ => return Err(Error::new(ident.span(), "unknown argument name; valid names are: 'result' and 'error'")),
			}

			Ok(())
		};

		read_arg()?;
		if input.is_empty() { return Ok(Self { result, error }) }
		input.parse::<token::Comma>()?;
		read_arg()?;

		Ok(Self {
			result,
			error,
		})
	}
}