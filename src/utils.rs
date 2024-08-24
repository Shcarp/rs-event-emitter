use std::error::Error;
use std::{any::Any, sync::Arc};

use futures::future::BoxFuture;
use uuid::Uuid;

use crate::{
    from_args::FromArgs,
    types::{ArcAny, BoxedAsyncHandler, BoxedHandler},
};

pub fn create_handler<F, Args>(handler: F) -> BoxedHandler
where
    F: Fn(Args) + Send + Sync + 'static,
    Args: FromArgs + 'static,
{
    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedHandler = (
        handler_id,
        Arc::new(move |args: &[ArcAny]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args);
            }
        }),
    );

    boxed_handler
}

pub fn create_async_handler<F, Args>(handler: F) -> BoxedAsyncHandler
where
    F: Fn(Args) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    Args: FromArgs + 'static,
{
    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedAsyncHandler = (
        handler_id,
        Arc::new(move |args: &[ArcAny]| {
            if let Some(typed_args) = Args::from_args(args) {
                handler(typed_args)
            } else {
                Box::pin(async {})
            }
        }),
    );

    boxed_handler
}

pub fn format_panic_message(panic_error: Box<dyn Any + Send>) -> String {
    panic_error
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| panic_error.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Unknown panic message".to_string())
}
