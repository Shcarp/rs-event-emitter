use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use super::*;


#[test]
pub fn test_multiple_event_types() {
    let emitter = EventEmitter::new();
    let _listener = emitter.start_listening();

    let result = Arc::new(Mutex::new(String::new()));
    let result_clone1 = Arc::clone(&result);
    let result_clone2 = Arc::clone(&result);

    emitter.on("greet", move |(name,): (String,)| {
        let mut result = result_clone1.lock().unwrap();
        *result = format!("Hello, {}!", name);
    });

    emitter.on("farewell", move |(name,): (String,)| {
        let mut result = result_clone2.lock().unwrap();
        *result = format!("Goodbye, {}!", name);
    });

    emit!(emitter, "greet", "Alice".to_string());
    thread::sleep(Duration::from_millis(50));
    assert_eq!(*result.lock().unwrap(), "Hello, Alice!");

    emit!(emitter, "farewell", "Bob".to_string());
    thread::sleep(Duration::from_millis(50));
    assert_eq!(*result.lock().unwrap(), "Goodbye, Bob!");
}