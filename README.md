### Event System

A simple rs-event-emitter implementation in Rust.

The rs-event-emitter allows you to register event handlers for specific events and emit events with associated data. It provides a way to decouple components and enable communication through events.

Currently, the parameters only support types that have implemented cloning. Improvements will be made in the future.

### Usage

To use the event system, follow these steps:

1. Add the following to your `Cargo.toml` file:

```toml
[dependencies]
rs-event-emitter = "0.0.5"
```

2. Import the crate in your `main.rs` or `lib.rs` file:

```rust
extern crate rs_event_emitter::*;
```

3. Create an event handler function:

```rust
// Define the event handlers
let handler1 = EventHandler::new(Box::new(|(a, b, c): (i32, &str, f64)| {
    println!("event1: {}, {}, {}", a, b, c);
    assert_eq!(a, 42);
    assert_eq!(b, "hello");
    assert_eq!(c, 3.14);
}));

let handler2 = EventHandler::new(Box::new(|(name, age): (String, u32)| {
    println!("event2: {}, {}", name, age);
    assert_eq!(name, "John");
    assert_eq!(age, 25);
}));
```

4. Register the event handler for a specific event:

```rust
let emitter = EventEmitter::new();

emitter.on("event1", Arc::new(handler1.clone()));
emitter.on("event2", Arc::new(handler2.clone()));

```

5. Emit an event:

```rust
emitter.emit("event1", Box::new((42, "hello", 3.14)));
emitter.emit("event2", Box::new(("John".to_string(), 25 as u32)));
```

6. Unregister event handlers using the EventEmitter::off method:

```rust
emitter.off("event1", Arc::new(handler1));
emitter.off("event2", Arc::new(handler2));
```

### API

`EventEmitter`

-   `fn new() -> Self`: Creates a new `EventEmitter` instance.

-   `fn on(&self, event: &'static str, handler: Arc<dyn Handle>)`: listen an event handler for a specific event.

-   `fn off(&self, event: &str, handler: Arc<dyn Handle>)`: unListen an event handler for a specific event.

-   `fn emit(&self, event: &str, data: Box<dyn Any>)`: Emits an event with associated data, triggering the registered event handlers for that event.

`EventHandler`

-   `fn new(handler: fn(T)) -> Self`: Creates a new event handler with the provided closure.

### Examples

```rust
use rs_event_emitter::*;

fn main() {
    // Create a new EventEmitter
        let emitter = EventEmitter::new();

        // Define the event handlers
        let handler1 = EventHandler::new(Box::new(|(a, b, c): (i32, &str, f64)| {
            println!("event1: {}, {}, {}", a, b, c);
            assert_eq!(a, 42);
            assert_eq!(b, "hello");
            assert_eq!(c, 3.14);
        }));

        let handler2 = EventHandler::new(Box::new(|(name, age): (String, u32)| {
            println!("event2: {}, {}", name, age);
            assert_eq!(name, "John");
            assert_eq!(age, 25);
        }));

        // Register the event handlers
        emitter.on("event1", Arc::new(handler1.clone()));
        emitter.on("event2", Arc::new(handler2.clone()));
        // Emit events
        emitter.emit("event1", Box::new((42, "hello", 3.14)));
        emitter.emit("event2", Box::new(("John".to_string(), 25 as u32)));

        // Unregister event handlers
        emitter.off("event1", Arc::new(handler1));
        emitter.off("event2", Arc::new(handler2));

        // Emit events again, but handlers should not be called
        emitter.emit("event1", Box::new((42, "hello", 3.14)));
        emitter.emit("event2", Box::new(("John".to_string(), 26 as u32)));
}
```

When you run the above code, you should see the event handlers being called for the emitted events.

### Contributing

If you would like to contribute to this project, please open an issue or submit a pull request.

### License

This project is licensed under the MIT License
