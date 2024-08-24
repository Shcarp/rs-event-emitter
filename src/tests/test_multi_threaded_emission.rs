use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use super::*;

#[test]
fn test_multi_threaded_emission() {
    let emitter = EventEmitter::with_thread_pool_size(4);
    let _listener = emitter.start_listening();

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    emitter.on("increment", move |(_,): (i32,)| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    let emitter_clone = emitter.clone();
    let handle = thread::spawn(move || {
        for _ in 0..50 {
            emit!(emitter_clone, "increment", 1);
        }
    });

    for _ in 0..50 {
        emit!(emitter, "increment", 1);
    }

    handle.join().unwrap();

    thread::sleep(Duration::from_millis(200));

    assert_eq!(counter.load(Ordering::SeqCst), 100);
}
