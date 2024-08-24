use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn test_event_with_off_listening() {
    let emitter = Arc::new(EventEmitter::new());
    let listener = emitter.start_listening();

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    let handler_id = emitter.on("increment", move |(value,): (i32,)| {
        counter_clone.fetch_add(value, Ordering::SeqCst);
    });

    // Emit some events
    for _ in 0..20 {
        emit!(emitter, "increment", 1);
    }

    // Wait a bit, then stop listening
    thread::sleep(Duration::from_millis(50));

    println!("Stopping listening {}", counter.load(Ordering::SeqCst));

    emitter.stop_listening();

    // Give some time for the listener thread to stop
    thread::sleep(Duration::from_millis(10));

    // Emit more events after stopping listening
    for _ in 0..20 {
        emit!(emitter, "increment", 1);
    }

    // Give some time for any remaining events to be processed
    thread::sleep(Duration::from_millis(100));

    // The count should not have changed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        20,
        "Counter changed after stopping listening"
    );

    // Emit more events after stopping listening
    for _ in 0..20 {
        emit!(emitter, "increment", 1);
    }

    // The count should not have changed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        20,
        "Counter changed after stopping listening"
    );

    // Remove the handler
    emitter.off("increment", handler_id);

    // Emit more events after removing the handler
    for _ in 0..20 {
        emit!(emitter, "increment", 1);
    }

    // The count should not have changed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        20,
        "Counter changed after handler was removed"
    );

    // Wait for the listener thread to finish
    listener.join().unwrap();
}
