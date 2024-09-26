use std::sync::Arc;

#[cfg(feature = "async")]
use futures::future::BoxFuture;

use uuid::Uuid;

use crate::{
    from_args::FromArgs,
    types::Param,
};

#[cfg(feature = "async")]
use crate::types::BoxedAsyncHandler;

#[cfg(feature = "sync")]
use crate::types::BoxedHandler;

#[cfg(feature = "wasm")]
use crate::types::BoxedHandler;

#[cfg(feature = "sync")]
pub fn create_handler<F, Args>(handler: F) -> BoxedHandler
where
    F: Fn(Args) + Send + Sync + 'static,
    Args: FromArgs + 'static,
{
    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedHandler = (
        handler_id,
        Arc::new(move |args: &[Param]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args);
            }
        }),
    );

    boxed_handler
}

#[cfg(feature = "async")]
pub fn create_async_handler<F, Args>(handler: F) -> BoxedAsyncHandler
where
    F: Fn(Args) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    Args: FromArgs + 'static,
{
    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedAsyncHandler = (
        handler_id,
        Arc::new(move |args: &[Param]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args)
            } else {
                Box::pin(async {})
            }
        }),
    );

    boxed_handler
}


#[cfg(feature = "wasm")]
pub fn create_wasm_handler<F, Args>(handler: F) -> BoxedHandler
where
    F: Fn(Args) + 'static,
    Args: FromArgs + 'static,
{
    use std::{cell::RefCell, rc::Rc};

    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedHandler = (
        handler_id,
        Rc::new(RefCell::new(move |args: &[Param]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args)
            }
        }))
    );

    boxed_handler
}

