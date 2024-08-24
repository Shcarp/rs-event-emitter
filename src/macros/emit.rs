#[macro_export]
macro_rules! emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Arc::new($arg) as $crate::types::ArcAny),*])
    };
}

#[macro_export]
macro_rules! async_emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Arc::new($arg) as $crate::types::ArcAny),*]).await
    };
}