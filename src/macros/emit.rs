#[macro_export]
macro_rules! emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Arc::new($arg) as $crate::types::Param),*])
    };
}

#[macro_export]
macro_rules! async_emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Arc::new($arg) as $crate::types::Param),*]).await
    };
}

#[macro_export]
macro_rules! rc_emit {
    ($emitter:expr, $event:expr, $($arg:expr),*) => {
        $emitter.emit($event, vec![$(Rc::new($arg) as $crate::types::Param),*])
    };
}