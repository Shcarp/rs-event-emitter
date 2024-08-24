use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::*;

#[test]
fn test_event_with_multiple_arguments() {
    let emitter = EventEmitter::new();
    let _listener = emitter.start_listening();

    let result = Arc::new(Mutex::new(String::new()));
    let result_clone = Arc::clone(&result);

    emitter.on(
        "person_info",
        move |(name, age, city, is_student, gpa): (String, i32, String, bool, f32)| {
            let mut result = result_clone.lock().unwrap();
            *result = format!(
                "{} is {} years old, lives in {}, student status: {}, GPA: {:.2}",
                name,
                age,
                city,
                if is_student { "Yes" } else { "No" },
                gpa
            );
        },
    );

    emit!(
        emitter,
        "person_info",
        "Alice".to_string(),
        30,
        "New York".to_string(),
        true,
        3.75f32
    );

    // 给一些时间让事件被处理
    thread::sleep(Duration::from_millis(100));

    assert_eq!(
        *result.lock().unwrap(),
        "Alice is 30 years old, lives in New York, student status: Yes, GPA: 3.75"
    );
}
