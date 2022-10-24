#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use vin::*;
pub enum MsgA {
    Foo,
    Bar,
    Baz,
}
#[automatically_derived]
impl ::core::fmt::Debug for MsgA {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            MsgA::Foo => ::core::fmt::Formatter::write_str(f, "Foo"),
            MsgA::Bar => ::core::fmt::Formatter::write_str(f, "Bar"),
            MsgA::Baz => ::core::fmt::Formatter::write_str(f, "Baz"),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for MsgA {
    #[inline]
    fn clone(&self) -> MsgA {
        match self {
            MsgA::Foo => MsgA::Foo,
            MsgA::Bar => MsgA::Bar,
            MsgA::Baz => MsgA::Baz,
        }
    }
}
pub enum MsgB {
    Foo,
    Bar,
    Baz,
}
#[automatically_derived]
impl ::core::fmt::Debug for MsgB {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            MsgB::Foo => ::core::fmt::Formatter::write_str(f, "Foo"),
            MsgB::Bar => ::core::fmt::Formatter::write_str(f, "Bar"),
            MsgB::Baz => ::core::fmt::Formatter::write_str(f, "Baz"),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for MsgB {
    #[inline]
    fn clone(&self) -> MsgB {
        match self {
            MsgB::Foo => MsgB::Foo,
            MsgB::Bar => MsgB::Bar,
            MsgB::Baz => MsgB::Baz,
        }
    }
}
pub struct MyActor {
    vin_ctx: ::vin::vin_macros::tokio::sync::RwLock<VinContextMyActor>,
    vin_hidden: VinHiddenMyActor,
}
pub struct VinContextMyActor {
    pub number: u32,
}
struct VinMailboxMyActor {
    erased_mailboxes: ::std::collections::HashMap<
        ::core::any::TypeId,
        ::vin::vin_macros::vin_core::BoxedErasedTx,
    >,
    msga: (
        ::vin::vin_macros::async_channel::Sender<MsgA>,
        ::vin::vin_macros::async_channel::Receiver<MsgA>,
    ),
    msgb: (
        ::vin::vin_macros::async_channel::Sender<MsgB>,
        ::vin::vin_macros::async_channel::Receiver<MsgB>,
    ),
}
impl Default for VinMailboxMyActor {
    fn default() -> Self {
        let msga = ::vin::vin_macros::async_channel::unbounded();
        let msgb = ::vin::vin_macros::async_channel::unbounded();
        let erased_mailboxes = ::std::collections::HashMap::default();
        erased_mailboxes
            .insert(
                ::core::any::TypeId::of::<MsgA>(),
                VinTxWrapperMyActor(msga.0.clone()),
            );
        erased_mailboxes
            .insert(
                ::core::any::TypeId::of::<MsgB>(),
                VinTxWrapperMyActor(msgb.0.clone()),
            );
        Self {
            erased_mailboxes,
            msga,
            msgb,
        }
    }
}
struct VinHiddenMyActor {
    mailbox: VinMailboxMyActor,
    state: ::vin::vin_macros::crossbeam::atomic::AtomicCell<
        ::vin::vin_macros::vin_core::State,
    >,
    close: ::vin::vin_macros::tokio::sync::Notify,
}
impl Default for VinHiddenMyActor {
    fn default() -> Self {
        Self {
            mailbox: Default::default(),
            state: Default::default(),
            close: Default::default(),
        }
    }
}
struct VinTxWrapperMyActor<T: Message>(::vin::vin_macros::async_channel::Sender<T>);
impl ::vin::vin_macros::vin_core::ErasedTx for VinTxWrapperMyActor<MsgA> {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn send_erased<'life0, 'async_trait>(
        &'life0 self,
        msg: ::vin::vin_macros::vin_core::BoxedMessage,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                use ::vin::vin_macros::vin_core::downcast_rs::Downcast;
                let msg = msg.into_any().downcast::<MsgA>().ok();
                if let Some(msg) = msg {
                    let msg = *msg;
                    let _ = __self.0.try_send(msg);
                } else {
                    ::core::panicking::unreachable_display(
                        &"If a message is not downcastable to its correct type written \
                                in the 'erased_mailboxes' HashMap, then it's a bug. Please report at: \
                                https://github.com/mscofield0/vin/issues",
                    );
                }
            };
        })
    }
}
impl ::vin::vin_macros::vin_core::ErasedTx for VinTxWrapperMyActor<MsgB> {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn send_erased<'life0, 'async_trait>(
        &'life0 self,
        msg: ::vin::vin_macros::vin_core::BoxedMessage,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                use ::vin::vin_macros::vin_core::downcast_rs::Downcast;
                let msg = msg.into_any().downcast::<MsgB>().ok();
                if let Some(msg) = msg {
                    let msg = *msg;
                    let _ = __self.0.try_send(msg);
                } else {
                    ::core::panicking::unreachable_display(
                        &"If a message is not downcastable to its correct type written \
                                in the 'erased_mailboxes' HashMap, then it's a bug. Please report at: \
                                https://github.com/mscofield0/vin/issues",
                    );
                }
            };
        })
    }
}
impl ::vin::vin_macros::vin_core::Forwarder<MsgA> for MyActor {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn forward<'life0, 'async_trait>(
        &'life0 self,
        msg: MsgA,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                let _ = __self.vin_hidden.mailbox.msga.0.try_send(msg);
            };
        })
    }
}
impl ::vin::vin_macros::vin_core::Forwarder<MsgB> for MyActor {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn forward<'life0, 'async_trait>(
        &'life0 self,
        msg: MsgB,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                let _ = __self.vin_hidden.mailbox.msgb.0.try_send(msg);
            };
        })
    }
}
impl ::vin::vin_macros::vin_core::Actor for MyActor {
    type Context = VinContextMyActor;
    fn new(ctx: Self::Context) -> ::vin::vin_macros::vin_core::StrongAddr<Self> {
        ::vin::vin_macros::vin_core::StrongAddr::new(Self {
            vin_ctx: ::vin::vin_macros::tokio::sync::RwLock::new(ctx),
            vin_hidden: Default::default(),
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn ctx<'life0, 'async_trait>(
        &'life0 self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = ::vin::vin_macros::tokio::sync::RwLockReadGuard<Self::Context>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<
                    ::vin::vin_macros::tokio::sync::RwLockReadGuard<Self::Context>,
                > {
                return __ret;
            }
            let __self = self;
            let __ret: ::vin::vin_macros::tokio::sync::RwLockReadGuard<Self::Context> = {
                __self.vin_ctx.read().await
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn ctx_mut<'life0, 'async_trait>(
        &'life0 self,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = ::vin::vin_macros::tokio::sync::RwLockWriteGuard<Self::Context>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<
                    ::vin::vin_macros::tokio::sync::RwLockWriteGuard<Self::Context>,
                > {
                return __ret;
            }
            let __self = self;
            let __ret: ::vin::vin_macros::tokio::sync::RwLockWriteGuard<Self::Context> = {
                __self.vin_ctx.write().await
            };
            #[allow(unreachable_code)] __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn start<'life0, 'async_trait>(
        self: &'life0 ::vin::vin_macros::vin_core::StrongAddr<Self>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = ::vin::vin_macros::vin_core::StrongAddr<Self>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<
                    ::vin::vin_macros::vin_core::StrongAddr<Self>,
                > {
                return __ret;
            }
            let __self = self;
            let __ret: ::vin::vin_macros::vin_core::StrongAddr<Self> = {
                if let Err(_)
                    = __self
                        .vin_hidden
                        .state
                        .compare_exchange(
                            ::vin::vin_macros::vin_core::State::Pending,
                            ::vin::vin_macros::vin_core::State::Starting,
                        )
                {
                    return __self.clone();
                }
                ::vin::vin_macros::vin_core::add_actor();
                let ret = __self.clone();
                let actor = __self.clone();
                ::vin::vin_macros::tokio::spawn(async move {
                    use ::core::borrow::Borrow;
                    let mut handler_join_set = ::vin::vin_macros::tokio::task::JoinSet::new();
                    let shutdown = ::vin::vin_macros::vin_core::SHUTDOWN_SIGNAL
                        .notified();
                    let close = actor.vin_hidden.close.notified();
                    let mut shutdown = shutdown;
                    #[allow(unused_mut)]
                    let mut shutdown = unsafe {
                        ::tokio::macros::support::Pin::new_unchecked(&mut shutdown)
                    };
                    let mut close = close;
                    #[allow(unused_mut)]
                    let mut close = unsafe {
                        ::tokio::macros::support::Pin::new_unchecked(&mut close)
                    };
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event tests\\test1.rs:17",
                                    "test1",
                                    ::tracing::Level::DEBUG,
                                    Some("tests\\test1.rs"),
                                    Some(17u32),
                                    Some("test1"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["message"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::DEBUG
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::DEBUG
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(
                                                    &::core::fmt::Arguments::new_v1(
                                                        &["", "::on_started()"],
                                                        &[::core::fmt::ArgumentV1::new_display(&"MyActor")],
                                                    ) as &dyn Value,
                                                ),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                        }
                    };
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_started(
                            actor.borrow(),
                        )
                        .await;
                    actor
                        .vin_hidden
                        .state
                        .store(::vin::vin_macros::vin_core::State::Running);
                    loop {
                        {
                            #[doc(hidden)]
                            mod __tokio_select_util {
                                pub(super) enum Out<_0, _1, _2, _3, _4> {
                                    _0(_0),
                                    _1(_1),
                                    _2(_2),
                                    _3(_3),
                                    _4(_4),
                                    Disabled,
                                }
                                pub(super) type Mask = u8;
                            }
                            use ::tokio::macros::support::Future;
                            use ::tokio::macros::support::Pin;
                            use ::tokio::macros::support::Poll::{Ready, Pending};
                            const BRANCHES: u32 = 5;
                            let mut disabled: __tokio_select_util::Mask = Default::default();
                            if !true {
                                let mask: __tokio_select_util::Mask = 1 << 0;
                                disabled |= mask;
                            }
                            if !true {
                                let mask: __tokio_select_util::Mask = 1 << 1;
                                disabled |= mask;
                            }
                            if !true {
                                let mask: __tokio_select_util::Mask = 1 << 2;
                                disabled |= mask;
                            }
                            if !true {
                                let mask: __tokio_select_util::Mask = 1 << 3;
                                disabled |= mask;
                            }
                            if !true {
                                let mask: __tokio_select_util::Mask = 1 << 4;
                                disabled |= mask;
                            }
                            let mut output = {
                                let mut futures = (
                                    actor.vin_hidden.mailbox.msga.1.recv(),
                                    actor.vin_hidden.mailbox.msgb.1.recv(),
                                    &mut close,
                                    &mut shutdown,
                                    handler_join_set.join_next(),
                                );
                                ::tokio::macros::support::poll_fn(|cx| {
                                        let mut is_pending = false;
                                        let start = {
                                            ::tokio::macros::support::thread_rng_n(BRANCHES)
                                        };
                                        for i in 0..BRANCHES {
                                            let branch;
                                            #[allow(clippy::modulo_one)]
                                            {
                                                branch = (start + i) % BRANCHES;
                                            }
                                            match branch {
                                                #[allow(unreachable_code)]
                                                0 => {
                                                    let mask = 1 << branch;
                                                    if disabled & mask == mask {
                                                        continue;
                                                    }
                                                    let (fut, ..) = &mut futures;
                                                    let mut fut = unsafe { Pin::new_unchecked(fut) };
                                                    let out = match Future::poll(fut, cx) {
                                                        Ready(out) => out,
                                                        Pending => {
                                                            is_pending = true;
                                                            continue;
                                                        }
                                                    };
                                                    disabled |= mask;
                                                    #[allow(unused_variables)] #[allow(unused_mut)]
                                                    match &out {
                                                        msga => {}
                                                        _ => continue,
                                                    }
                                                    return Ready(__tokio_select_util::Out::_0(out));
                                                }
                                                #[allow(unreachable_code)]
                                                1 => {
                                                    let mask = 1 << branch;
                                                    if disabled & mask == mask {
                                                        continue;
                                                    }
                                                    let (_, fut, ..) = &mut futures;
                                                    let mut fut = unsafe { Pin::new_unchecked(fut) };
                                                    let out = match Future::poll(fut, cx) {
                                                        Ready(out) => out,
                                                        Pending => {
                                                            is_pending = true;
                                                            continue;
                                                        }
                                                    };
                                                    disabled |= mask;
                                                    #[allow(unused_variables)] #[allow(unused_mut)]
                                                    match &out {
                                                        msgb => {}
                                                        _ => continue,
                                                    }
                                                    return Ready(__tokio_select_util::Out::_1(out));
                                                }
                                                #[allow(unreachable_code)]
                                                2 => {
                                                    let mask = 1 << branch;
                                                    if disabled & mask == mask {
                                                        continue;
                                                    }
                                                    let (_, _, fut, ..) = &mut futures;
                                                    let mut fut = unsafe { Pin::new_unchecked(fut) };
                                                    let out = match Future::poll(fut, cx) {
                                                        Ready(out) => out,
                                                        Pending => {
                                                            is_pending = true;
                                                            continue;
                                                        }
                                                    };
                                                    disabled |= mask;
                                                    #[allow(unused_variables)] #[allow(unused_mut)]
                                                    match &out {
                                                        _ => {}
                                                        _ => continue,
                                                    }
                                                    return Ready(__tokio_select_util::Out::_2(out));
                                                }
                                                #[allow(unreachable_code)]
                                                3 => {
                                                    let mask = 1 << branch;
                                                    if disabled & mask == mask {
                                                        continue;
                                                    }
                                                    let (_, _, _, fut, ..) = &mut futures;
                                                    let mut fut = unsafe { Pin::new_unchecked(fut) };
                                                    let out = match Future::poll(fut, cx) {
                                                        Ready(out) => out,
                                                        Pending => {
                                                            is_pending = true;
                                                            continue;
                                                        }
                                                    };
                                                    disabled |= mask;
                                                    #[allow(unused_variables)] #[allow(unused_mut)]
                                                    match &out {
                                                        _ => {}
                                                        _ => continue,
                                                    }
                                                    return Ready(__tokio_select_util::Out::_3(out));
                                                }
                                                #[allow(unreachable_code)]
                                                4 => {
                                                    let mask = 1 << branch;
                                                    if disabled & mask == mask {
                                                        continue;
                                                    }
                                                    let (_, _, _, _, fut, ..) = &mut futures;
                                                    let mut fut = unsafe { Pin::new_unchecked(fut) };
                                                    let out = match Future::poll(fut, cx) {
                                                        Ready(out) => out,
                                                        Pending => {
                                                            is_pending = true;
                                                            continue;
                                                        }
                                                    };
                                                    disabled |= mask;
                                                    #[allow(unused_variables)] #[allow(unused_mut)]
                                                    match &out {
                                                        Some(res) => {}
                                                        _ => continue,
                                                    }
                                                    return Ready(__tokio_select_util::Out::_4(out));
                                                }
                                                _ => {
                                                    ::core::panicking::unreachable_display(
                                                        &"reaching this means there probably is an off by one bug",
                                                    )
                                                }
                                            }
                                        }
                                        if is_pending {
                                            Pending
                                        } else {
                                            Ready(__tokio_select_util::Out::Disabled)
                                        }
                                    })
                                    .await
                            };
                            match output {
                                __tokio_select_util::Out::_0(msga) => {
                                    let msga = msga
                                        .expect(
                                            "channel should never be closed while the actor is running",
                                        );
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "event tests\\test1.rs:17",
                                                    "test1",
                                                    ::tracing::Level::DEBUG,
                                                    Some("tests\\test1.rs"),
                                                    Some(17u32),
                                                    Some("test1"),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &["message"],
                                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::DEBUG
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::DEBUG
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                let mut iter = CALLSITE.metadata().fields().iter();
                                                CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set(
                                                        &[
                                                            (
                                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                Some(
                                                                    &::core::fmt::Arguments::new_v1(
                                                                        &["", "::handle::<", ">()"],
                                                                        &match (&"MyActor", &"MsgA") {
                                                                            args => {
                                                                                [
                                                                                    ::core::fmt::ArgumentV1::new_display(args.0),
                                                                                    ::core::fmt::ArgumentV1::new_display(args.1),
                                                                                ]
                                                                            }
                                                                        },
                                                                    ) as &dyn Value,
                                                                ),
                                                            ),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                    let self2 = actor.clone();
                                    handler_join_set
                                        .spawn(async move {
                                            match <Self as ::vin::vin_macros::vin_core::Handler<
                                                MsgA,
                                            >>::handle(self2.borrow(), msga)
                                                .await
                                            {
                                                Ok(_) => Ok(()),
                                                Err(err) => {
                                                    Err(
                                                        ::vin::vin_macros::vin_core::HandlerError::new("MsgA", err),
                                                    )
                                                }
                                            }
                                        });
                                }
                                __tokio_select_util::Out::_1(msgb) => {
                                    let msgb = msgb
                                        .expect(
                                            "channel should never be closed while the actor is running",
                                        );
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "event tests\\test1.rs:17",
                                                    "test1",
                                                    ::tracing::Level::DEBUG,
                                                    Some("tests\\test1.rs"),
                                                    Some(17u32),
                                                    Some("test1"),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &["message"],
                                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::DEBUG
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::DEBUG
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                let mut iter = CALLSITE.metadata().fields().iter();
                                                CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set(
                                                        &[
                                                            (
                                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                Some(
                                                                    &::core::fmt::Arguments::new_v1(
                                                                        &["", "::handle::<", ">()"],
                                                                        &match (&"MyActor", &"MsgB") {
                                                                            args => {
                                                                                [
                                                                                    ::core::fmt::ArgumentV1::new_display(args.0),
                                                                                    ::core::fmt::ArgumentV1::new_display(args.1),
                                                                                ]
                                                                            }
                                                                        },
                                                                    ) as &dyn Value,
                                                                ),
                                                            ),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                    let self2 = actor.clone();
                                    handler_join_set
                                        .spawn(async move {
                                            match <Self as ::vin::vin_macros::vin_core::Handler<
                                                MsgB,
                                            >>::handle(self2.borrow(), msgb)
                                                .await
                                            {
                                                Ok(_) => Ok(()),
                                                Err(err) => {
                                                    Err(
                                                        ::vin::vin_macros::vin_core::HandlerError::new("MsgB", err),
                                                    )
                                                }
                                            }
                                        });
                                }
                                __tokio_select_util::Out::_2(_) => {
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "event tests\\test1.rs:17",
                                                    "test1",
                                                    ::tracing::Level::DEBUG,
                                                    Some("tests\\test1.rs"),
                                                    Some(17u32),
                                                    Some("test1"),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &["message"],
                                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::DEBUG
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::DEBUG
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                let mut iter = CALLSITE.metadata().fields().iter();
                                                CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set(
                                                        &[
                                                            (
                                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                Some(
                                                                    &::core::fmt::Arguments::new_v1(
                                                                        &["", " received close signal"],
                                                                        &[::core::fmt::ArgumentV1::new_display(&"MyActor")],
                                                                    ) as &dyn Value,
                                                                ),
                                                            ),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                    actor
                                        .vin_hidden
                                        .state
                                        .store(::vin::vin_macros::vin_core::State::Closing);
                                    break;
                                }
                                __tokio_select_util::Out::_3(_) => {
                                    {
                                        use ::tracing::__macro_support::Callsite as _;
                                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                            static META: ::tracing::Metadata<'static> = {
                                                ::tracing_core::metadata::Metadata::new(
                                                    "event tests\\test1.rs:17",
                                                    "test1",
                                                    ::tracing::Level::DEBUG,
                                                    Some("tests\\test1.rs"),
                                                    Some(17u32),
                                                    Some("test1"),
                                                    ::tracing_core::field::FieldSet::new(
                                                        &["message"],
                                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                                    ),
                                                    ::tracing::metadata::Kind::EVENT,
                                                )
                                            };
                                            ::tracing::callsite::DefaultCallsite::new(&META)
                                        };
                                        let enabled = ::tracing::Level::DEBUG
                                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                            && ::tracing::Level::DEBUG
                                                <= ::tracing::level_filters::LevelFilter::current()
                                            && {
                                                let interest = CALLSITE.interest();
                                                !interest.is_never()
                                                    && ::tracing::__macro_support::__is_enabled(
                                                        CALLSITE.metadata(),
                                                        interest,
                                                    )
                                            };
                                        if enabled {
                                            (|value_set: ::tracing::field::ValueSet| {
                                                let meta = CALLSITE.metadata();
                                                ::tracing::Event::dispatch(meta, &value_set);
                                            })({
                                                #[allow(unused_imports)]
                                                use ::tracing::field::{debug, display, Value};
                                                let mut iter = CALLSITE.metadata().fields().iter();
                                                CALLSITE
                                                    .metadata()
                                                    .fields()
                                                    .value_set(
                                                        &[
                                                            (
                                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                Some(
                                                                    &::core::fmt::Arguments::new_v1(
                                                                        &["", " received shutdown signal"],
                                                                        &[::core::fmt::ArgumentV1::new_display(&"MyActor")],
                                                                    ) as &dyn Value,
                                                                ),
                                                            ),
                                                        ],
                                                    )
                                            });
                                        } else {
                                        }
                                    };
                                    actor
                                        .vin_hidden
                                        .state
                                        .store(::vin::vin_macros::vin_core::State::Closing);
                                    break;
                                }
                                __tokio_select_util::Out::_4(Some(res)) => {
                                    match res {
                                        Ok(handler_res) => {
                                            if let Err(err) = handler_res {
                                                {
                                                    use ::tracing::__macro_support::Callsite as _;
                                                    static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                                        static META: ::tracing::Metadata<'static> = {
                                                            ::tracing_core::metadata::Metadata::new(
                                                                "event tests\\test1.rs:17",
                                                                "test1",
                                                                ::tracing::Level::ERROR,
                                                                Some("tests\\test1.rs"),
                                                                Some(17u32),
                                                                Some("test1"),
                                                                ::tracing_core::field::FieldSet::new(
                                                                    &["message"],
                                                                    ::tracing_core::callsite::Identifier(&CALLSITE),
                                                                ),
                                                                ::tracing::metadata::Kind::EVENT,
                                                            )
                                                        };
                                                        ::tracing::callsite::DefaultCallsite::new(&META)
                                                    };
                                                    let enabled = ::tracing::Level::ERROR
                                                        <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                                        && ::tracing::Level::ERROR
                                                            <= ::tracing::level_filters::LevelFilter::current()
                                                        && {
                                                            let interest = CALLSITE.interest();
                                                            !interest.is_never()
                                                                && ::tracing::__macro_support::__is_enabled(
                                                                    CALLSITE.metadata(),
                                                                    interest,
                                                                )
                                                        };
                                                    if enabled {
                                                        (|value_set: ::tracing::field::ValueSet| {
                                                            let meta = CALLSITE.metadata();
                                                            ::tracing::Event::dispatch(meta, &value_set);
                                                        })({
                                                            #[allow(unused_imports)]
                                                            use ::tracing::field::{debug, display, Value};
                                                            let mut iter = CALLSITE.metadata().fields().iter();
                                                            CALLSITE
                                                                .metadata()
                                                                .fields()
                                                                .value_set(
                                                                    &[
                                                                        (
                                                                            &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                            Some(
                                                                                &::core::fmt::Arguments::new_v1_formatted(
                                                                                    &["", "::handle::<", ">() failed with error: "],
                                                                                    &[
                                                                                        ::core::fmt::ArgumentV1::new_display(&"MyActor"),
                                                                                        ::core::fmt::ArgumentV1::new_display(&err.msg_name()),
                                                                                        ::core::fmt::ArgumentV1::new_debug(&err),
                                                                                    ],
                                                                                    &[
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 0usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 0u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 1usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 0u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 2usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 4u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                    ],
                                                                                    unsafe { ::core::fmt::UnsafeArg::new() },
                                                                                ) as &dyn Value,
                                                                            ),
                                                                        ),
                                                                    ],
                                                                )
                                                        });
                                                    } else {
                                                    }
                                                };
                                            }
                                        }
                                        Err(join_err) => {
                                            if let Ok(reason) = join_err.try_into_panic() {
                                                ::std::panic::resume_unwind(reason);
                                            }
                                        }
                                    }
                                }
                                __tokio_select_util::Out::Disabled => {
                                    ::std::rt::begin_panic(
                                        "all branches are disabled and there is no else branch",
                                    )
                                }
                                _ => {
                                    ::core::panicking::unreachable_display(
                                        &"failed to match bind",
                                    )
                                }
                            }
                        };
                    }
                    use ::core::time::Duration;
                    let _ = ::vin::vin_macros::tokio::time::timeout(
                            Duration::from_secs(1),
                            async {
                                while let Some(res) = handler_join_set.join_next().await {
                                    match res {
                                        Ok(handler_res) => {
                                            if let Err(err) = handler_res {
                                                {
                                                    use ::tracing::__macro_support::Callsite as _;
                                                    static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                                        static META: ::tracing::Metadata<'static> = {
                                                            ::tracing_core::metadata::Metadata::new(
                                                                "event tests\\test1.rs:17",
                                                                "test1",
                                                                ::tracing::Level::ERROR,
                                                                Some("tests\\test1.rs"),
                                                                Some(17u32),
                                                                Some("test1"),
                                                                ::tracing_core::field::FieldSet::new(
                                                                    &["message"],
                                                                    ::tracing_core::callsite::Identifier(&CALLSITE),
                                                                ),
                                                                ::tracing::metadata::Kind::EVENT,
                                                            )
                                                        };
                                                        ::tracing::callsite::DefaultCallsite::new(&META)
                                                    };
                                                    let enabled = ::tracing::Level::ERROR
                                                        <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                                        && ::tracing::Level::ERROR
                                                            <= ::tracing::level_filters::LevelFilter::current()
                                                        && {
                                                            let interest = CALLSITE.interest();
                                                            !interest.is_never()
                                                                && ::tracing::__macro_support::__is_enabled(
                                                                    CALLSITE.metadata(),
                                                                    interest,
                                                                )
                                                        };
                                                    if enabled {
                                                        (|value_set: ::tracing::field::ValueSet| {
                                                            let meta = CALLSITE.metadata();
                                                            ::tracing::Event::dispatch(meta, &value_set);
                                                        })({
                                                            #[allow(unused_imports)]
                                                            use ::tracing::field::{debug, display, Value};
                                                            let mut iter = CALLSITE.metadata().fields().iter();
                                                            CALLSITE
                                                                .metadata()
                                                                .fields()
                                                                .value_set(
                                                                    &[
                                                                        (
                                                                            &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                                            Some(
                                                                                &::core::fmt::Arguments::new_v1_formatted(
                                                                                    &["", "::handle::<", ">() failed with error: "],
                                                                                    &[
                                                                                        ::core::fmt::ArgumentV1::new_display(&"MyActor"),
                                                                                        ::core::fmt::ArgumentV1::new_display(&err.msg_name()),
                                                                                        ::core::fmt::ArgumentV1::new_debug(&err),
                                                                                    ],
                                                                                    &[
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 0usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 0u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 1usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 0u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                        ::core::fmt::rt::v1::Argument {
                                                                                            position: 2usize,
                                                                                            format: ::core::fmt::rt::v1::FormatSpec {
                                                                                                fill: ' ',
                                                                                                align: ::core::fmt::rt::v1::Alignment::Unknown,
                                                                                                flags: 4u32,
                                                                                                precision: ::core::fmt::rt::v1::Count::Implied,
                                                                                                width: ::core::fmt::rt::v1::Count::Implied,
                                                                                            },
                                                                                        },
                                                                                    ],
                                                                                    unsafe { ::core::fmt::UnsafeArg::new() },
                                                                                ) as &dyn Value,
                                                                            ),
                                                                        ),
                                                                    ],
                                                                )
                                                        });
                                                    } else {
                                                    }
                                                };
                                            }
                                        }
                                        Err(join_err) => {
                                            if let Ok(reason) = join_err.try_into_panic() {
                                                ::std::panic::resume_unwind(reason);
                                            }
                                        }
                                    }
                                }
                            },
                        )
                        .await;
                    handler_join_set.abort_all();
                    while handler_join_set.join_next().await.is_some() {}
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event tests\\test1.rs:17",
                                    "test1",
                                    ::tracing::Level::DEBUG,
                                    Some("tests\\test1.rs"),
                                    Some(17u32),
                                    Some("test1"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["message"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::DEBUG
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::DEBUG
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(
                                                    &::core::fmt::Arguments::new_v1(
                                                        &["", "::on_closing()"],
                                                        &[::core::fmt::ArgumentV1::new_display(&"MyActor")],
                                                    ) as &dyn Value,
                                                ),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                        }
                    };
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_closing(
                            actor.borrow(),
                        )
                        .await;
                    {
                        use ::tracing::__macro_support::Callsite as _;
                        static CALLSITE: ::tracing::callsite::DefaultCallsite = {
                            static META: ::tracing::Metadata<'static> = {
                                ::tracing_core::metadata::Metadata::new(
                                    "event tests\\test1.rs:17",
                                    "test1",
                                    ::tracing::Level::DEBUG,
                                    Some("tests\\test1.rs"),
                                    Some(17u32),
                                    Some("test1"),
                                    ::tracing_core::field::FieldSet::new(
                                        &["message"],
                                        ::tracing_core::callsite::Identifier(&CALLSITE),
                                    ),
                                    ::tracing::metadata::Kind::EVENT,
                                )
                            };
                            ::tracing::callsite::DefaultCallsite::new(&META)
                        };
                        let enabled = ::tracing::Level::DEBUG
                            <= ::tracing::level_filters::STATIC_MAX_LEVEL
                            && ::tracing::Level::DEBUG
                                <= ::tracing::level_filters::LevelFilter::current()
                            && {
                                let interest = CALLSITE.interest();
                                !interest.is_never()
                                    && ::tracing::__macro_support::__is_enabled(
                                        CALLSITE.metadata(),
                                        interest,
                                    )
                            };
                        if enabled {
                            (|value_set: ::tracing::field::ValueSet| {
                                let meta = CALLSITE.metadata();
                                ::tracing::Event::dispatch(meta, &value_set);
                            })({
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                let mut iter = CALLSITE.metadata().fields().iter();
                                CALLSITE
                                    .metadata()
                                    .fields()
                                    .value_set(
                                        &[
                                            (
                                                &iter.next().expect("FieldSet corrupted (this is a bug)"),
                                                Some(
                                                    &::core::fmt::Arguments::new_v1(
                                                        &["", "::on_closed()"],
                                                        &[::core::fmt::ArgumentV1::new_display(&"MyActor")],
                                                    ) as &dyn Value,
                                                ),
                                            ),
                                        ],
                                    )
                            });
                        } else {
                        }
                    };
                    actor
                        .vin_hidden
                        .state
                        .store(::vin::vin_macros::vin_core::State::Closed);
                    <Self as ::vin::vin_macros::vin_core::LifecycleHook>::on_closed(
                            actor.borrow(),
                        )
                        .await;
                    ::vin::vin_macros::vin_core::remove_actor();
                });
                ret
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
impl ::vin::vin_macros::vin_core::Addr for MyActor {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn send<'life0, 'async_trait, M>(
        &'life0 self,
        msg: M,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        Self: ::vin::vin_macros::vin_core::Forwarder<M> + Sized,
        M: 'async_trait + ::vin::vin_macros::vin_core::Message + Send,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                <Self as ::vin::vin_macros::vin_core::Forwarder<M>>::forward(__self, msg)
                    .await;
            };
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn send_erased<'life0, 'async_trait>(
        &'life0 self,
        msg: ::vin::vin_macros::vin_core::BoxedMessage,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let __self = self;
            let msg = msg;
            let _: () = {
                let tx = __self
                    .vin_hidden
                    .mailbox
                    .erased_mailboxes
                    .get(&msg.id())
                    .expect("sent a message that the actor does not handle");
                tx.send_erased(msg);
            };
        })
    }
    fn close(&self) {
        self.vin_hidden.state.store(::vin::vin_macros::vin_core::State::Closing);
        self.vin_hidden.close.notify_waiters();
    }
    fn state(&self) -> ::vin::vin_macros::vin_core::State {
        self.vin_hidden.state.load()
    }
}
impl vin::LifecycleHook for MyActor {}
impl vin::Handler<MsgA> for MyActor {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn handle<'life0, 'async_trait>(
        &'life0 self,
        msg: MsgA,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = anyhow::Result<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<anyhow::Result<()>> {
                return __ret;
            }
            let __self = self;
            let msg = msg;
            let __ret: anyhow::Result<()> = {
                let ctx = __self.ctx().await;
                {
                    ::std::io::_print(
                        ::core::fmt::Arguments::new_v1(
                            &["The message is: ", " and the number is ", "\n"],
                            &[
                                ::core::fmt::ArgumentV1::new_debug(&msg),
                                ::core::fmt::ArgumentV1::new_display(&ctx.number),
                            ],
                        ),
                    );
                };
                Err(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            ::core::fmt::Arguments::new_v1(&["hi, i am error"], &[]),
                        );
                        error
                    }),
                )
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
impl vin::Handler<MsgB> for MyActor {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn handle<'life0, 'async_trait>(
        &'life0 self,
        msg: MsgB,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = anyhow::Result<()>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<anyhow::Result<()>> {
                return __ret;
            }
            let __self = self;
            let msg = msg;
            let __ret: anyhow::Result<()> = {
                let ctx = __self.ctx().await;
                {
                    ::std::io::_print(
                        ::core::fmt::Arguments::new_v1(
                            &["The message is: ", " and the number is ", "\n"],
                            &[
                                ::core::fmt::ArgumentV1::new_debug(&msg),
                                ::core::fmt::ArgumentV1::new_display(&ctx.number),
                            ],
                        ),
                    );
                };
                Err(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            ::core::fmt::Arguments::new_v1(&["hi, i am error"], &[]),
                        );
                        error
                    }),
                )
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
#[cfg(test)]
mod tests {
    use std::time::Duration;
    use tracing::Level;
    use super::*;
    extern crate test;
    #[cfg(test)]
    #[rustc_test_marker]
    pub const test1: test::TestDescAndFn = test::TestDescAndFn {
        desc: test::TestDesc {
            name: test::StaticTestName("tests::test1"),
            ignore: false,
            ignore_message: ::core::option::Option::None,
            compile_fail: false,
            no_run: false,
            should_panic: test::ShouldPanic::No,
            test_type: test::TestType::IntegrationTest,
        },
        testfn: test::StaticTestFn(|| test::assert_test_result(test1())),
    };
    fn test1() {
        let body = async {
            tracing_subscriber::fmt().with_max_level(Level::TRACE).init();
            let ctx = VinContextMyActor { number: 42 };
            let actor = MyActor::new(ctx).start().await;
            actor.send(MsgA::Bar).await;
            actor.send(MsgB::Bar).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            vin::shutdown();
            vin::wait_for_shutdowns().await;
        };
        #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
        {
            return tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed building the Runtime")
                .block_on(body);
        }
    }
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&test1])
}
