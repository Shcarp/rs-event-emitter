use std::panic::{self, AssertUnwindSafe};
use std::{collections::HashMap, sync::Arc}; // Add this line

use futures::future::join_all; // Add this line

use futures::FutureExt;
use emitter_runtime::{AsyncRuntime, AsyncRwLock, AsyncRwLockReadGuard, AsyncRwLockWriteGuard};

use crate::{
    from_args::FromArgs,
    types::{Param, BoxedAsyncHandler, HandlerId},
    utils,
};

pub struct AsyncEventEmitter<R: AsyncRuntime> {
    handlers: Arc<R::RwLock<HashMap<String, Vec<BoxedAsyncHandler>>>>,
    runtime: Arc<R>,
}

impl<R: AsyncRuntime> AsyncEventEmitter<R> {
    pub fn new(rt: Arc<R>) -> Self {
        AsyncEventEmitter {
            handlers: Arc::new(rt.new_rwlock(HashMap::new())),
            runtime: rt,
        }
    }
}

impl<R: AsyncRuntime> Clone for AsyncEventEmitter<R> {
    fn clone(&self) -> Self {
        AsyncEventEmitter {
            handlers: Arc::clone(&self.handlers),
            runtime: Arc::clone(&self.runtime),
        }
    }
}

impl<R: AsyncRuntime> AsyncEventEmitter<R> {
    pub async fn on<F, Args>(&self, event: &str, handler: F) -> HandlerId
    where
        F: Fn(Args) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
        Args: FromArgs + 'static,
    {
        let boxed_handler: BoxedAsyncHandler = utils::create_async_handler(handler);
        let cloned_handler = boxed_handler.clone();

        let mut handlers_guard = self.handlers.write().await;
        let handlers = handlers_guard.deref_mut();
        handlers
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(boxed_handler);

        drop(handlers_guard);
        cloned_handler.0
    }

    pub async fn off(&self, event: &str, handler_id: HandlerId) {
        let mut handlers_guard = self.handlers.write().await;
        let handlers = handlers_guard.deref_mut();
        if let Some(event_handlers) = handlers.get_mut(event) {
            event_handlers.retain(|(id, _)| *id != handler_id);
        }
        drop(handlers_guard);
    }

    pub async fn emit(&self, event: &str, args: Vec<Param>) {
        let handlers_guard = self.handlers.read().await;
        let handlers = handlers_guard.deref();
        if let Some(event_handlers) = handlers.get(event) {
            let futures: Vec<_> = event_handlers
                .iter()
                .map(|(_, handler)| {
                    let handler = handler.clone();
                    let args = args.clone();
                    let runtime = self.runtime.clone();

                    runtime.spawn(async move {
                        let result = panic::catch_unwind(AssertUnwindSafe(|| async {
                            handler(&args).await
                        }));

                        match result {
                            Ok(future) => {
                                let future_result = AssertUnwindSafe(future).catch_unwind().await;

                                match future_result {
                                    Err(panic_error) => {
                                        let panic_message = match panic_error.downcast_ref::<&str>()
                                        {
                                            Some(s) => s.to_string(),
                                            None => match panic_error.downcast_ref::<String>() {
                                                Some(s) => s.clone(),
                                                None => "Unknown panic message".to_string(),
                                            },
                                        };
                                        println!("Panic occurred during await: {}", panic_message);
                                    }
                                    _ => {}
                                }
                            }
                            Err(panic_error) => {
                                let panic_message = match panic_error.downcast_ref::<&str>() {
                                    Some(s) => s.to_string(),
                                    None => match panic_error.downcast_ref::<String>() {
                                        Some(s) => s.clone(),
                                        None => "Unknown panic message".to_string(),
                                    },
                                };
                                println!("Panic occurred: {}", panic_message);
                            }
                        }
                    })
                })
                .collect();

            drop(handlers_guard);
            let _ = join_all(futures).await;
        }
    }
}
