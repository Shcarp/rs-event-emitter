use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn test_multiple_handlers() {
    let emitter = EventEmitter::new();
    let _listener = emitter.start_listening();

    let counter1 = Arc::new(AtomicI32::new(0));
    let counter2 = Arc::new(AtomicI32::new(0));
    let counter1_clone = Arc::clone(&counter1);
    let counter2_clone = Arc::clone(&counter2);

    emitter.on("increment", move |(_,): (i32,)| {
        counter1_clone.fetch_add(1, Ordering::SeqCst);
    });

    emitter.on("increment", move |(_,): (i32,)| {
        counter2_clone.fetch_add(2, Ordering::SeqCst);
    });

    emit!(emitter, "increment", 1);

    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter1.load(Ordering::SeqCst), 1);
    assert_eq!(counter2.load(Ordering::SeqCst), 2);
}
