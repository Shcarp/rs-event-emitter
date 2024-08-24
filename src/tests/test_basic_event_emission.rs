
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::*;

#[test]
pub fn test_basic_event_emission() {
    let emitter = EventEmitter::new();
    let _listener = emitter.start_listening();

    let counter = Arc::new(AtomicI32::new(0));
    let counter_clone = Arc::clone(&counter);

    emitter.on("increment", move |(_,): (i32,)| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    emit!(emitter, "increment", 1);
    emit!(emitter, "increment", 1);

    // 给一些时间让事件被处理
    thread::sleep(Duration::from_millis(100));

    assert_eq!(counter.load(Ordering::SeqCst), 2);
}