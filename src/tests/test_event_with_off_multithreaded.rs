use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use std::sync::Arc;
use std::thread;

use super::*;

#[test]
fn test_event_with_off_multithreaded() {
    let emitter = Arc::new(EventEmitter::new());
    let _listener = emitter.start_listening();

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    let handler_id = emitter.on("increment", move |(value,): (i32,)| {
        counter_clone.fetch_add(value, Ordering::SeqCst);
    });

    let total_events = 100; // 5 threads * 20 events each
    let events_before_off = 50; // We'll remove the handler halfway through

    // Spawn multiple threads to emit events
    let mut handles = vec![];
    for _ in 0..5 {
        let emitter_clone = Arc::clone(&emitter);
        let handle = thread::spawn(move || {
            for i in 0..20 {
                emit!(emitter_clone, "increment", 1);
                if i == 9 { // Pause halfway through to allow for handler removal
                    thread::sleep(Duration::from_millis(50));
                }
                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // Wait a bit, then remove the handler
    thread::sleep(Duration::from_millis(50));
    emitter.off("increment", handler_id);

    // Wait for all threads to finishhandler
    for handle in handles {
        handle.join().unwrap();
    }

    // Give some time for any remaining events to be processed
    thread::sleep(Duration::from_millis(100));

    let final_count = counter.load(Ordering::SeqCst);
    println!("Final count: {}", final_count);

    // The final count should be greater than events_before_off but less than total_events
    assert!(
        final_count >= events_before_off && final_count < total_events,
        "Final count was {}, expected between {} and {}",
        final_count,
        events_before_off,
        total_events
    );

        // Emit more events after removing the handler
    for _ in 0..20 {
        emit!(emitter, "increment", 1);
    }

    thread::sleep(Duration::from_millis(100));

    // The count should not have changed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        final_count,
        "Counter changed after handler was removed"
    );
}
