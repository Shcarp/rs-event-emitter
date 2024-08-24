use std::sync::Arc;

use uuid::Uuid;

use crate::{from_args::FromArgs, types::{ArcAny, BoxedHandler, HandlerId}};

pub fn create_handler<F, Args>(handler: F) -> BoxedHandler
where
    F: Fn(Args) + Send + Sync + 'static,
    Args: FromArgs + 'static,
{
    let handler_id = Uuid::new_v4();

    let boxed_handler: BoxedHandler = (handler_id, Arc::new(move |args: &[ArcAny]| {
        if let Some(typed_args) = Args::from_args(args) {
            handler(typed_args);
        }
    }));

    boxed_handler
}

