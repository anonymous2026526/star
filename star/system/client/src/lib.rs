use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use log::debug;
use pingora::{apps::ServerApp, protocols::Stream, server::ShutdownWatch};

use crate::proxy::StarReverseProxy;

pub const STAR_TLS_EXTENSION_TYPE: u16 = 65280;
const MAX_HEADER_SIZE: usize = 64 * 1024;

type BoxError = Box<dyn Error + Send + Sync>;


#[async_trait]
impl ServerApp for StarReverseProxy {
    async fn process_new(
        self: &Arc<Self>,
        mut io: Stream,
        _shutdown: &ShutdownWatch,
    ) -> Option<Stream> {
        if let Err(err) = self.handle(&mut io).await {
            debug!("STAR reverse proxy connection closed: {err}");
        }

        None
    }
    
    #[doc = " This callback will be called once after the service stops listening to its endpoints."]
    #[allow(elided_named_lifetimes,clippy::async_yields_async,clippy::diverging_sub_expression,clippy::let_unit_value,clippy::needless_arbitrary_self_type,clippy::no_effect_underscore_binding,clippy::shadow_same,clippy::type_complexity,clippy::type_repetition_in_bounds,clippy::used_underscore_binding)]
    fn cleanup<'life0,'async_trait>(&'life0 self) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send+'async_trait> >where 'life0: 'async_trait,Self: ::core::marker::Sync+'async_trait{
        Box::pin(async move {
            let __self = self;
            let _: () = {};
        })
    }
}


pub mod utils;
pub mod config;
pub mod auth;
pub mod proxy;

