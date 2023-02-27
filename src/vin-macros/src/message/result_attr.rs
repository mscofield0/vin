use syn::{Ident, parse::Parse, Error, TypePath, Token};

pub enum ResultArg {
	NoResult,
	WithResult(TypePath),
}

impl Parse for ResultArg {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		if input.is_empty() {
            return Ok(ResultArg::NoResult);
        }

		let ident = input.parse::<Ident>()?;
		let arg = match ident.to_string().as_str() {
			"result" => {
				input.parse::<Token![=]>()?;
				let result_type = input.parse::<TypePath>()?;
				
				ResultArg::WithResult(result_type)
			}
			_ => return Err(Error::new(ident.span(), "unknown argument name; valid names are: 'result'")),
		};

		Ok(arg)
	}
}