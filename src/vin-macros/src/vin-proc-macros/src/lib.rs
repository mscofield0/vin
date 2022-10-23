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
    Wait,
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

/// A noop macro attribute. Check `vin::actor` for more information.
#[proc_macro_attribute]
pub fn handles(_args: TokenStream, _input: TokenStream) -> TokenStream {
    quote! {}.into()
}

/// Generates the actor impls and forms necessary fields.
/// 
/// ## Additional arguments
/// Currently there is only one additional argument and it's 'bounded'.
/// 
/// ### `bounded`
/// The `bounded` argument allows you to set an upper limit to the amount of messages a mailbox 
/// can take in. It also allows you to set a strategy for handling a full mailbox. Current 
/// available strategies are: 'wait' (awaits until the mailbox is available), 'report' (reports a failure) 
/// and 'silent' (silently discards the message).
/// 
/// ## Example
/// ```rust
/// #[vin::actor]
/// #[vin::handles(Message, bounded(size = 1024, report))]
/// struct MyActor;
/// ```
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
            vin_ctx: ::vin::vin_macros::tokio::sync::RwLock<#context_struct_name>,
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

fn form_message_names(handles_attrs: &Vec<HandlesAttribute>) -> (Vec<TypePath>, Vec<Ident>) {
    let msg_names = handles_attrs.iter()
        .map(|attr| attr.message_type.clone())
        .collect::<Vec<_>>();

    let msg_short_names = handles_attrs.iter()
        .map(|attr| {
            let ident_name = attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            quote::format_ident!("{}", ident_name)
        })
        .collect::<Vec<_>>();

    (msg_names, msg_short_names)
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
            quote! { #short_name: (::vin::vin_macros::async_channel::Sender<#type_path>, ::vin::vin_macros::async_channel::Receiver<#type_path>) }
        })
        .collect::<Vec<_>>();
    let mailbox_field_inits = handles_attrs.iter()
        .map(|attr| {
            let short_name = &attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            if let Some(Bounded { size, mode: _ }) = &attr.bounded {
                quote! { #short_name: ::vin::vin_macros::async_channel::bounded(#size) }
            } else {
                quote! { #short_name: ::vin::vin_macros::async_channel::unbounded() }
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
            state: ::vin::vin_macros::crossbeam::atomic::AtomicCell<::vin::vin_macros::vin_core::State>,
            close: ::vin::vin_macros::tokio::sync::Notify,
        }

        impl Default for #hidden_struct_name {
            fn default() -> Self {
                Self {
                    mailbox: Default::default(),
                    state: Default::default(),
                    close: Default::default(),
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
    let (msg_names, msg_short_names) = form_message_names(&handles_attrs);
    let context_name = form_context_struct_name(name);

    quote! {
        #[::vin::vin_macros::async_trait::async_trait]
        impl #impl_generics ::vin::vin_macros::vin_core::Actor for #name #ty_generics #where_clause {
            type Context = #context_name;

            fn new(ctx: Self::Context) -> Self {
                Self {
                    vin_ctx: ::vin::vin_macros::tokio::sync::RwLock::new(ctx),
                    vin_hidden: Default::default(),
                }
            }

            async fn send<M: ::vin::vin_macros::vin_core::Message + Send>(&self, msg: M)
                where Self: ::vin::vin_macros::vin_core::Forwarder<M> 
            {
                <Self as ::vin::vin_macros::vin_core::Forwarder<M>>::forward(self, msg).await;
            }

            fn state(&self) -> ::vin::vin_macros::vin_core::State {
                self.vin_hidden.state.load()
            }

            async fn ctx(&self) -> ::vin::vin_macros::tokio::sync::RwLockReadGuard<Self::Context> {
                self.vin_ctx.read().await
            }

            async fn ctx_mut(&self) -> ::vin::vin_macros::tokio::sync::RwLockWriteGuard<Self::Context> {
                self.vin_ctx.write().await
            }

            fn close(&self) {
                self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Closing);
                self.vin_hidden.close.notify_waiters();
            }

            async fn start(self) -> ::vin::vin_macros::vin_core::Addr<Self>
                where Self: LifecycleHook 
            {
                ::vin::vin_macros::vin_core::add_actor();

                let ret = ::vin::vin_macros::vin_core::Addr::new(self);
                let self = ret.clone();

                ::vin::vin_macros::tokio::spawn(async move {
                    use ::core::borrow::Borrow;

                    let mut handler_join_set = ::vin::vin_macros::tokio::task::JoinSet::new();
                    let shutdown = ::vin::vin_macros::vin_core::SHUTDOWN_SIGNAL.notified();
                    let close = self.vin_hidden.close.notified();
                    ::vin::vin_macros::tokio::pin!(shutdown);
                    ::vin::vin_macros::tokio::pin!(close);

                    ::vin::vin_macros::tracing::debug!("{}::on_started()", stringify!(#name));
                    self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Starting);
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_started(self.borrow()).await;
                    self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Running);
                    loop {
                        ::vin::vin_macros::tokio::select! {
                            #(#msg_short_names = self.vin_hidden.mailbox.#msg_short_names.1.recv() => {
                                let #msg_short_names = #msg_short_names.expect("channel should never be closed while the actor is running");
                                ::vin::vin_macros::tracing::debug!("{}::handle::<{}>()", stringify!(#name), stringify!(#msg_names));

                                let self2 = self.clone();
                                handler_join_set.spawn(async move {
                                    match <Self as ::vin::vin_macros::vin_core::Handler<#msg_names>>::handle(self2.borrow(), #msg_short_names).await {
                                        Ok(_) => Ok(()),
                                        Err(err) => Err(::vin::vin_macros::vin_core::HandlerError::new(stringify!(#msg_names), err)),
                                    }
                                });
                            }),*
                            _ = &mut close => {
                                ::vin::vin_macros::tracing::debug!("{} received close signal", stringify!(#name));
                                self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Closing);
                                break;
                            },
                            _ = &mut shutdown => {
                                ::vin::vin_macros::tracing::debug!("{} received shutdown signal", stringify!(#name));
                                self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Closing);
                                break;
                            },
                            Some(res) = handler_join_set.join_next() => match res {
                                Ok(handler_res) => if let Err(err) = handler_res {
                                    ::vin::vin_macros::tracing::error!("{}::handle::<{}>() failed with error: {:#?}", stringify!(#name), err.msg_name(), err);
                                },
                                Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                    ::std::panic::resume_unwind(reason);
                                },
                            },
                        };
                    }

                    // Give some time for the existing handlers to gracefully end
                    use ::core::time::Duration;
                    let _ = ::vin::vin_macros::tokio::time::timeout(Duration::from_secs(1), async {
                        while let Some(res) = handler_join_set.join_next().await {
                            match res {
                                Ok(handler_res) => if let Err(err) = handler_res {
                                    ::vin::vin_macros::tracing::error!("{}::handle::<{}>() failed with error: {:#?}", stringify!(#name), err.msg_name(), err);
                                },
                                Err(join_err) => if let Ok(reason) = join_err.try_into_panic() {
                                    ::std::panic::resume_unwind(reason);
                                },
                            }
                        }
                    }).await;

                    // After aborting, the join set still needs to be drained since tasks only get cancelled
                    // on an 'await' point
                    handler_join_set.abort_all();
                    while handler_join_set.join_next().await.is_some() {}

                    ::vin::vin_macros::tracing::debug!("{}::on_closing()", stringify!(#name));
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_closing(self.borrow()).await;
                    ::vin::vin_macros::tracing::debug!("{}::on_closed()", stringify!(#name));
                    self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Closed);
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_closed(self.borrow()).await;
                    ::vin::vin_macros::vin_core::remove_actor();
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
            let type_path = &attr.message_type.path;
            let short_name = attr.message_type.path.segments.last().unwrap().ident.to_string().to_lowercase();
            let short_name = quote::format_ident!("{}", short_name);
            let quoted = if let Some(Bounded { size: _, mode }) = &attr.bounded {
                match mode {
                    BoundedMode::Wait => {
                        quote! {
                            self.vin_hidden.mailbox.#short_name.0.send(msg).await
                                .expect("mailbox channel should never be closed during the actor's lifetime");
                        }
                    },
                    BoundedMode::Report => {
                        quote! { 
                            if let Err(err) = self.vin_hidden.mailbox.#short_name.0.try_send(msg) {
                                match err {
                                    ::vin::vin_macros::async_channel::TrySendError::Full(_) => {
                                        ::vin::vin_macros::tracing::error!("mailbox for {:?} is full", stringify!(#type_path));
                                    },
                                    ::vin::vin_macros::async_channel::TrySendError::Closed(_) => {
                                        unreachable!("mailbox channel should never be closed during the actor's lifetime");
                                    },
                                }
                            }
                        }
                    },
                    BoundedMode::Silent => {
                        quote! {
                            let _ = self.vin_hidden.mailbox.#short_name.0.try_send(msg);
                        }
                    },
                }
            } else {
                quote! {
                    let _ = self.vin_hidden.mailbox.#short_name.0.try_send(msg);
                }
            };

            (attr.message_type.clone(), quoted)
        })
        .map(|(msg_name, body)| {
            quote! {
                #[::vin::vin_macros::async_trait::async_trait]
                impl #impl_generics ::vin::vin_macros::vin_core::Forwarder<#msg_name> for #name #ty_generics #where_clause {
                    async fn forward(&self, msg: #msg_name) {
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



