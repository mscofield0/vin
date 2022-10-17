use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use rassert_rs::rassert;
use syn::{parse_macro_input, Ident, DeriveInput, parse::Parse, Error, LitInt, TypePath, Token, Data, Fields, parenthesized, ImplGenerics, TypeGenerics, WhereClause};
use quote::quote;

struct HandlesAttribute {
    message_type: TypePath,
    bounded: Option<Bounded>,
}

enum BoundedMode {
    Overwrite,
    Report,
    Silent,
}

type BoundSize = usize;

struct Bounded {
    size: BoundSize,
    mode: BoundedMode,
}

enum AttrArg {
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
                    "overwrite" => BoundedMode::Overwrite,
                    "report" => BoundedMode::Report,
                    "silent" => BoundedMode::Silent,
                    _ => return Err(Error::new(mode.span(), "unknown bounded mode; valid modes are: 'overwrite', 'report', 'silent'")),
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

#[proc_macro_attribute]
pub fn handles(_args: TokenStream, _input: TokenStream) -> TokenStream {
    quote! {}.into()
}

/// Main macro for vin.
///
/// Generates the actor impls and forms necessary fields.
#[proc_macro_attribute]
pub fn actor(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let data = match input.data {
        Data::Struct(ref mut data) => data,
        _ => return Error::new(input.ident.span(), "vin only works on structs").into_compile_error().into(),
    };

    // Common vars for building the final output
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parsed handles attributes
    let handles_attrs = input.attrs.iter()
        .filter(|attr| !attr.path.is_ident("handles"))
        .map(|attr| attr.parse_args::<HandlesAttribute>())
        .collect::<Result<Vec<_>, _>>();

    let handles_attrs = match handles_attrs {
        Ok(attrs) => attrs,
        Err(err) => return err.to_compile_error().into(),
    };

    if handles_attrs.is_empty() {
        return Error::new(name.span(), "no message specified for handling").to_compile_error().into();
    }

    // All handled message names
    let vin_hidden_struct = form_vin_hidden_struct(name, &handles_attrs);

    // Thread-safe, wrapped actor context
    let vin_context_struct = form_vin_context_struct(name, &data.fields);

    // Forwarder trait impls
    let forwarder_traits = form_forwarder_impls(name, &handles_attrs, &impl_generics, &ty_generics, where_clause);

    // Actor trait impl
    let actor_trait = form_actor_trait(name, &handles_attrs, &impl_generics, &ty_generics, where_clause);

    // Modify struct fields
    let hidden_struct_name = form_hidden_struct_name(name);
    let context_struct_name = form_context_struct_name(name);

    // Remove handles attributes
    input.attrs.retain(|attr| attr.path.segments.last().unwrap().ident != "handles");
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        pub struct #name {
            vin_ctx: ::vin_macros::tokio::sync::RwLock<#context_struct_name>,
            vin_hidden: #hidden_struct_name,
        }

        #vin_context_struct

        #vin_hidden_struct

        #forwarder_traits
        
        #actor_trait
    }.into()
}

fn form_mailbox_name(name: &Ident) -> Ident {
    quote::format_ident!("VinMailbox{}", name)
}

fn form_hidden_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinHidden{}", name)
}

fn form_context_struct_name(name: &Ident) -> Ident {
    quote::format_ident!("VinContext{}", name)
}

fn form_tracing_report(msg: String) -> TokenStream2 {
    quote! { ::vin_macros::tracing::debug!("{:?}", #msg) }
}

fn form_message_names(handles_attrs: &Vec<HandlesAttribute>) -> (Vec<TypePath>, Vec<Ident>, Vec<Ident>) {
    let msg_names = handles_attrs.iter()
        .map(|attr| attr.message_type.clone())
        .collect::<Vec<_>>();

    let msg_short_names = handles_attrs.iter()
        .map(|attr| {
            let ident_name = attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            quote::format_ident!("{}", ident_name)
        })
        .collect::<Vec<_>>();

    let msg_fut_names = msg_short_names.iter()
        .map(|name| quote::format_ident!("{}_fut", name))
        .collect::<Vec<_>>();

    (msg_names, msg_short_names, msg_fut_names)
}

fn form_vin_context_struct(name: &Ident, fields: &Fields) -> TokenStream2 {
    let context_struct_name = form_context_struct_name(name);

    match fields {
        Fields::Named(fields) => quote! {
            pub struct #context_struct_name #fields
        },
        Fields::Unnamed(fields) => quote! {
            pub struct #context_struct_name #fields;
        },
        Fields::Unit => quote! {
            pub struct #context_struct_name;
        }
    }
}

fn form_vin_hidden_struct(name: &Ident, handles_attrs: &Vec<HandlesAttribute>) -> TokenStream2 {
    // ===== Form mailbox
    // Mailbox fields to inject into the actor
    let mailbox_fields = handles_attrs.iter()
        .map(|attr| {
            let type_path = &attr.message_type;
            let short_name = &type_path.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            if attr.bounded.is_some() {
                quote! { #short_name: ::vin_macros::crossbeam::queue::ArrayQueue<#type_path> }
            } else {
                quote! { #short_name: ::vin_macros::crossbeam::queue::SegQueue<#type_path> }
            }
        })
        .collect::<Vec<_>>();
    let mailbox_field_inits = handles_attrs.iter()
        .map(|attr| {
            let short_name = &attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            if let Some(Bounded { size, mode: _ }) = &attr.bounded {
                quote! { #short_name: ::vin_macros::crossbeam::queue::ArrayQueue::new(#size) }
            } else {
                quote! { #short_name: ::vin_macros::crossbeam::queue::SegQueue::new() }
            }
        })
        .collect::<Vec<_>>();

    // Mailbox struct to inject into the actor
    let mailbox_struct_name = form_mailbox_name(name);
    let mailbox_struct = quote! {
        struct #mailbox_struct_name {
            #(#mailbox_fields),*
        }

        impl Default for #mailbox_struct_name {
            fn default() -> Self {
                Self {
                    #(#mailbox_field_inits),*
                }
            }
        }
    };

    // ===== Form final output
    let hidden_struct_name = form_hidden_struct_name(name);
    quote! {
        #mailbox_struct

        struct #hidden_struct_name {
            mailbox: #mailbox_struct_name,
            state: ::vin_macros::crossbeam::atomic::AtomicCell<::vin_macros::vin_core::State>,
        }

        impl Default for #hidden_struct_name {
            fn default() -> Self {
                Self {
                    mailbox: Default::default(),
                    state: Default::default(),
                }
            }
        }
    }
}

fn form_actor_trait(
    name: &Ident,
    handles_attrs: &Vec<HandlesAttribute>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let (msg_names, msg_short_names, msg_fut_names) = form_message_names(&handles_attrs);
    let context_name = form_context_struct_name(name);

    let on_started = form_tracing_report(format!("{}::on_started", name.to_string()));
    let on_closing = form_tracing_report(format!("{}::on_closing", name.to_string()));
    let on_closed = form_tracing_report(format!("{}::on_closed", name.to_string()));
    let handling = form_tracing_report(format!("{}::handle", name.to_string()));

    quote! {
        #[::vin_macros::async_trait::async_trait]
        impl #impl_generics ::vin_macros::vin_core::Actor for #name #ty_generics #where_clause {
            type Context = #context_name;

            fn new(ctx: Self::Context) -> Self {
                Self {
                    vin_ctx: ::vin_macros::tokio::sync::RwLock::new(ctx),
                    vin_hidden: Default::default(),
                }
            }

            fn send<M: ::vin_macros::vin_core::Message>(&self, msg: M)
                where Self: ::vin_macros::vin_core::Forwarder<M> 
            {
                <Self as ::vin_macros::vin_core::Forwarder<M>>::forward(self, msg);
            }

            fn state(&self) -> ::vin_macros::vin_core::State {
                self.vin_hidden.state.load()
            }

            async fn ctx(&self) -> ::vin_macros::tokio::sync::RwLockReadGuard<Self::Context> {
                self.vin_ctx.read().await
            }

            async fn ctx_mut(&self) -> ::vin_macros::tokio::sync::RwLockWriteGuard<Self::Context> {
                self.vin_ctx.write().await
            }

            fn close(&self) {
                self.vin_hidden.state.store(::vin_macros::vin_core::State::Closing);
            }

            async fn start(self) -> ::vin_macros::vin_core::Addr<Self> {
                let ret = ::vin_macros::vin_core::Addr::new(self);
                let self = ret.clone();

                ::vin_macros::tokio::spawn(async move {
                    println!("State: {:?}", self.vin_hidden.state);
                    #(
                    let mut #msg_fut_names = ::vin_macros::futures::future::poll_fn(|_| {
                        if let Some(val) = self.vin_hidden.mailbox.#msg_short_names.pop() {
                            ::core::task::Poll::Ready(val)
                        } else {
                            ::core::task::Poll::Pending
                        }
                    });

                    )*

                    #on_started;
                    self.vin_hidden.state.store(::vin_macros::vin_core::State::Starting);
                    self.on_started().await;
                    self.vin_hidden.state.store(::vin_macros::vin_core::State::Running);
                    loop {
                        match self.vin_hidden.state.load() {
                            ::vin_macros::vin_core::State::Running => {},
                            ::vin_macros::vin_core::State::Closing => break,
                            _ => unreachable!("vin: invalid state found in task loop; shouldn't happen ever"),
                        }

                        use ::core::borrow::Borrow;
                        ::vin_macros::tokio::select! {#(
                            #msg_short_names = &mut #msg_fut_names => {
                                #handling;
                                <Self as ::vin_macros::vin_core::Handler<#msg_names>>::handle(self.borrow(), #msg_short_names).await
                            }
                        ),*};
                    }
                    #on_closing;
                    self.on_closing().await;
                    #on_closed;
                    self.vin_hidden.state.store(::vin_macros::vin_core::State::Closed);
                    self.on_closed().await;
                });
                
                ret
            }
        }
    }
}

fn form_forwarder_impls(
    name: &Ident,
    handles_attrs: &Vec<HandlesAttribute>,
    impl_generics: &ImplGenerics,
    ty_generics: &TypeGenerics,
    where_clause: Option<&WhereClause>,
) -> TokenStream2 {
    let streams = handles_attrs.iter()
        .map(|attr| {
            let short_name = attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            let quoted = if let Some(Bounded { size: _, mode }) = &attr.bounded {
                match mode {
                    BoundedMode::Overwrite => {
                        quote! {
                            let _ = self.vin_hidden.mailbox.#short_name.push(msg);
                        }
                    },
                    BoundedMode::Report => {
                        quote! { 
                            if let Err(err) = self.vin_hidden.mailbox.#short_name.push(msg) {
                                tracing::error!("{:?}", err);
                            }
                        }
                    },
                    BoundedMode::Silent => {
                        quote! {
                            let _ = self.vin_hidden.mailbox.#short_name.push(msg);
                        }
                    },
                }
            } else {
                quote! {
                    self.vin_hidden.mailbox.#short_name.push(msg);
                }
            };

            (attr.message_type.clone(), quoted)
        })
        .map(|(msg_name, body)| {
            quote! {
                impl #impl_generics ::vin_macros::vin_core::Forwarder<#msg_name> for #name #ty_generics #where_clause {
                    fn forward(&self, msg: #msg_name) {
                        #body
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let mut stream = TokenStream2::new();
    streams.into_iter().for_each(|x| stream.extend(x) );

    stream
}



