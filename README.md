### Event System
A simple rs-event-emitter implementation in Rust.

The rs-event-emitter allows you to register event handlers for specific events and emit events with associated data. It provides a way to decouple components and enable communication through events.

Currently, the parameters only support types that have implemented cloning. Improvements will be made in the future.

### Usage
To use the event system, follow these steps:

1. Add the following to your `Cargo.toml` file:
```toml
[dependencies]
rs-event-emitter = "0.0.1"
```
2. Import the crate in your `main.rs` or `lib.rs` file:
```rust
extern crate rs_event_emitter::*;
```
3. Create an event handler function:
```rust
// Define the event handlers
let handler1 = EventHandler::new(|(a, b, c): (i32, &str, f64)| {
    assert_eq!(a, 42);
    assert_eq!(b, "hello");
    assert_eq!(c, 3.14);
});

let handler2 = EventHandler::new(|(name, age): (String, u32)| {
    assert_eq!(name, "John");
    assert_eq!(age, 25);
});
```
4. Register the event handler for a specific event:
```rust
let emitter = EventEmitter::new();

emitter.on("event1".to_string(), handler1.clone());
emitter.on("event2".to_string(), handler2.clone());

```
5. Emit an event:
```rust
emitter.emit::<(i32, &str, f64)>("event1", (42, "hello", 3.14));
emitter.emit::<(String, u32)>("event2", ("John".to_string(), 25));
```
6. Unregister event handlers using the EventEmitter::off method:
```rust
emitter.off("event1", &handler1);
emitter.off("event2", &handler2);
```

### API
`EventEmitter`
+ `new() -> Self`: Creates a new `EventEmitter` instance.

+ `on<T>(&self, event: String, handler: EventHandler<T>)`: listen an event handler for a specific event.

+ `off<T>(&self, event: &str, handler: &EventHandler<T>)`: unListen an event handler for a specific event.

+ `emit<T: 'static + Clone>(&self, event: &str, data: T)`: Emits an event with associated data, triggering the registered event handlers for that event.

`EventHandler`
+ `new<F>(handler: F) -> Self`: Creates a new event handler with the provided closure.

### Examples
```rust
use rs_event_emitter::*;

fn main() {
    // Create a new EventEmitter
    let emitter = EventEmitter::new();

    // Define the event handlers
    let handler1 = EventHandler::new(|(a, b, c): (i32, &str, f64)| {
        println!("Event handler 1 called with data: {}, {}, {}", a, b, c);
    });

    let handler2 = EventHandler::new(|(name, age): (String, u32)| {
        println!("Event handler 2 called with data: {}, {}", name, age);
    });

    // Register the event handlers
    emitter.on("event1".to_string(), handler1.clone());
    emitter.on("event2".to_string(), handler2.clone());

    // Emit events
    emitter.emit::<(i32, &str, f64)>("event1", (42, "hello", 3.14));
    emitter.emit::<(String, u32)>("event2", ("John".to_string(), 25));

    // Unregister event handlers
    emitter.off("event1", &handler1);
    emitter.off("event2", &handler2);

    // Emit events again, but handlers should not be called
    emitter.emit::<(i32, &str, f64)>("event1", (42, "hello", 3.14));
    emitter.emit::<(String, u32)>("event2", ("John".to_string(), 25));
}
```
When you run the above code, you should see the event handlers being called for the emitted events.

### Contributing
If you would like to contribute to this project, please open an issue or submit a pull request.

### License
This project is licensed under the MIT License
