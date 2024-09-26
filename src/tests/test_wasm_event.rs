use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use crate::rc_emit;
use crate::wasm_emitter::EventEmitter;
use std::rc::Rc;
use std::cell::RefCell;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_on_and_emit() {
    let emitter = EventEmitter::new();
    
    let called = Rc::new(RefCell::new(false));
    let called_clone = called.clone();

    emitter.on("test", move |()| {
        let mut ref_ = called_clone.borrow_mut();
        *ref_ = true
    });

    emitter.emit("test", vec![]);
    assert!(*called.borrow());
}

#[wasm_bindgen_test]
fn test_off() {
    let emitter = EventEmitter::new();
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();

    let handler_id = emitter.on("increment", move |()| {
        *counter_clone.borrow_mut() += 1;
    });

    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 1);

    emitter.off("increment", handler_id);
    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 1); // Counter should not increase after removing the handler
}

#[wasm_bindgen_test]
fn test_once() {
    let emitter = EventEmitter::new();
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();

    emitter.once("increment", move |()| {
        let mut ref_ = counter_clone.borrow_mut();
        *ref_ += 1;
    });

    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 1);

    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 1); // Counter should not increase on second emit
}

#[wasm_bindgen_test]
fn test_remove_all_listeners() {
    let emitter = EventEmitter::new();
    let counter = Rc::new(RefCell::new(0));

    emitter.on("increment", {
        let counter = counter.clone();
        move |()| {
            *counter.borrow_mut() += 1;
        }
    });

    emitter.on("increment", {
        let counter = counter.clone();
        move |()| {
            *counter.borrow_mut() += 1;
        }
    });

    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 2);

    emitter.remove_all_listeners("increment");
    emitter.emit("increment", vec![]);
    assert_eq!(*counter.borrow(), 2); // Counter should not increase after removing all listeners
}

#[wasm_bindgen_test]
fn test_listener_count_and_event_names() {
    let emitter = EventEmitter::new();

    emitter.on("event1", |()| {});
    emitter.on("event1", |()| {});
    emitter.on("event2", |()| {});

    assert_eq!(emitter.listener_count("event1"), 2);
    assert_eq!(emitter.listener_count("event2"), 1);
    assert_eq!(emitter.listener_count("event3"), 0);

    let event_names = emitter.event_names();
    assert!(event_names.contains(&"event1".to_string()));
    assert!(event_names.contains(&"event2".to_string()));
    assert_eq!(event_names.len(), 2);
}

#[wasm_bindgen_test]
fn test_emit_with_parameters() {
    let emitter = EventEmitter::new();
    let result = Rc::new(RefCell::new(String::new()));
    let result_clone = result.clone();

    emitter.on("greet", move |args: (String, i32)| {
        *result_clone.borrow_mut() = format!("Hello, {}! You are {} years old.", args.0, args.1)
    });

    rc_emit!(emitter, "greet", "Alice".to_string(), 30);
    assert_eq!(*result.borrow(), "Hello, Alice! You are 30 years old.");
}
